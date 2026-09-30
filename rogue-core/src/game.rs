use crate::action::Action;
use crate::ai;
use crate::combat;
use crate::comps::{Effect, InBackpack};
use crate::dungeon;
use crate::entity::EntityId;
use crate::event::Event;
use crate::fov;
use crate::geom::{Dir, Point};
use crate::map::Map;
use crate::rng::Rng;
use crate::state::{Input, RunState};
use crate::world::World;

pub struct Game {
    map: Map,
    world: World,
    player: EntityId,
    events: Vec<Event>,
    log: Vec<String>,
    state: RunState,
    rng: Rng,
}

impl Game {
    /// 시드 하나로 던전 한 판이 정해진다. 같은 시드는 항상 같은 판이다
    pub fn new(seed: u64) -> Game {
        let mut rng = Rng::new(seed);
        let dungeon = dungeon::generate(40, 25, &mut rng);
        let Some((first, rest)) = dungeon.rooms.split_first() else {
            unreachable!("생성기는 방을 최소 하나는 만든다");
        };
        let mut world = World::default();
        let player = world.spawn_player(first.center());
        for room in rest {
            for _ in 0..rng.range(0, 3) {
                let p = room.random_point(&mut rng);
                if world.blocker_at(p).is_some() {
                    continue; // 이미 막힌 자리
                }
                if rng.chance(60) {
                    world.spawn_goblin(p);
                } else {
                    world.spawn_kobold(p);
                }
            }
            if rng.chance(40) {
                world.spawn_potion(room.random_point(&mut rng));
            }
            if rng.chance(30) {
                world.spawn_scroll(room.random_point(&mut rng));
            }
        }
        let mut game = Game {
            map: dungeon.map,
            world,
            player,
            events: Vec::new(),
            log: Vec::new(),
            state: RunState::AwaitingInput,
            rng,
        };
        game.update_views();
        game
    }

    /// 3 단계까지 쓰던 손으로 만든 방. 기존 테스트를 그대로 돌리기 위해 남긴다
    #[cfg(test)]
    pub fn hand_made() -> Game {
        use crate::map::Tile;

        let mut world = World::default();
        let player = world.spawn_player(Point::new(15, 10));
        world.spawn_goblin(Point::new(5, 5));
        world.spawn_goblin(Point::new(24, 4));
        world.spawn_kobold(Point::new(20, 15));
        world.spawn_potion(Point::new(18, 12));
        world.spawn_scroll(Point::new(13, 8));
        let mut map = Map::room(30, 20);
        // 시야와 길찾기를 시험할 벽
        for y in 3..=12 {
            map.set(Point::new(10, y), Tile::Wall);
        }
        for x in 16..=24 {
            map.set(Point::new(x, 7), Tile::Wall);
        }
        let mut game = Game {
            map,
            world,
            player,
            events: Vec::new(),
            log: Vec::new(),
            state: RunState::AwaitingInput,
            rng: Rng::new(0),
        };
        game.update_views();
        game
    }

    pub fn map(&self) -> &Map {
        &self.map
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn player(&self) -> EntityId {
        self.player
    }

    /// 지금까지의 전투 메시지. 오래된 것이 앞에 있다
    pub fn log(&self) -> &[String] {
        &self.log
    }

    pub fn state(&self) -> &RunState {
        &self.state
    }

    pub fn is_over(&self) -> bool {
        matches!(self.state, RunState::GameOver { .. })
    }

    pub fn player_hp(&self) -> i32 {
        self.world.healths.get(self.player).map_or(0, |h| h.hp)
    }

    /// 플레이어에게 지금 보이는 좌표인가
    pub fn can_see(&self, p: Point) -> bool {
        self.world
            .viewsheds
            .get(self.player)
            .is_some_and(|v| v.visible.contains(&p))
    }

    /// 플레이어가 가방에 가진 아이템. 가방 칸 순서대로 나온다
    pub fn inventory(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.world
            .backpacks
            .iter()
            .filter(|(_, pack)| pack.owner == self.player)
            .map(|(id, _)| id)
    }

    /// 매 프레임 한 번 부른다. 현재 상태를 값으로 꺼내(소비) 다음 상태를 만들어 되돌려 넣는다
    pub fn update(&mut self, input: Option<Input>) {
        let current = std::mem::replace(&mut self.state, RunState::AwaitingInput);
        self.state = match current {
            RunState::AwaitingInput => self.on_awaiting_input(input),
            RunState::MonsterTurn => match self.monsters_act() {
                Some(cause) => RunState::GameOver { cause },
                None => RunState::AwaitingInput,
            },
            RunState::ShowInventory => self.on_inventory(input),
            RunState::Targeting { item, cursor } => self.on_targeting(item, cursor, input),
            RunState::GameOver { cause } => match input {
                Some(Input::Restart) => {
                    // 같은 수열 위에서 다음 판을 만든다. core 는 시계를 모른다
                    *self = Game::new(self.rng.next_u64());
                    return;
                }
                _ => RunState::GameOver { cause },
            },
        };
    }

    fn on_awaiting_input(&mut self, input: Option<Input>) -> RunState {
        match input {
            Some(Input::Act(action)) => self.player_turn(action),
            Some(Input::OpenInventory) => RunState::ShowInventory,
            Some(Input::Restart)
            | Some(Input::Cancel)
            | Some(Input::Choose(_))
            | Some(Input::MoveCursor(_))
            | Some(Input::Confirm)
            | None => RunState::AwaitingInput,
        }
    }

    fn on_inventory(&mut self, input: Option<Input>) -> RunState {
        match input {
            Some(Input::Cancel) | Some(Input::OpenInventory) => RunState::AwaitingInput,
            Some(Input::Choose(n)) => {
                // 빌림이 이 줄에서 끝나도록 값(EntityId, Copy)만 꺼낸다
                let chosen = self.inventory().nth(n);
                match chosen {
                    Some(item) => self.use_item(item),
                    None => RunState::ShowInventory,
                }
            }
            Some(Input::Act(_))
            | Some(Input::Restart)
            | Some(Input::MoveCursor(_))
            | Some(Input::Confirm)
            | None => RunState::ShowInventory,
        }
    }

    fn on_targeting(&mut self, item: EntityId, cursor: Point, input: Option<Input>) -> RunState {
        match input {
            Some(Input::MoveCursor(dir)) => {
                let next = cursor.step(dir);
                // 보이는 곳으로만 커서를 옮길 수 있다
                let cursor = if self.can_see(next) { next } else { cursor };
                RunState::Targeting { item, cursor }
            }
            Some(Input::Confirm) => self.cast(item, cursor),
            Some(Input::Cancel) => RunState::ShowInventory,
            Some(Input::Act(_))
            | Some(Input::Restart)
            | Some(Input::OpenInventory)
            | Some(Input::Choose(_))
            | None => RunState::Targeting { item, cursor },
        }
    }

    /// 플레이어의 행동 하나를 처리하고 다음 상태를 돌려준다
    fn player_turn(&mut self, action: Action) -> RunState {
        self.perform(self.player, action);
        self.update_views();
        self.finish_player_turn()
    }

    fn finish_player_turn(&mut self) -> RunState {
        match self.apply_events() {
            Some(cause) => RunState::GameOver { cause },
            None => RunState::MonsterTurn,
        }
    }

    fn use_item(&mut self, item: EntityId) -> RunState {
        let Some(effect) = self.world.items.get(item).map(|i| i.0) else {
            return RunState::ShowInventory;
        };
        let name = self.name_of(item);
        match effect {
            Effect::Heal(amount) => {
                self.push_log(format!("You drink the {name}."));
                self.events.push(Event::Heal {
                    target: self.player,
                    amount,
                });
                self.events.push(Event::Consume { item });
                self.finish_player_turn()
            }
            Effect::AreaDamage { .. } => {
                let cursor = self
                    .world
                    .positions
                    .get(self.player)
                    .map_or(Point::new(0, 0), |p| p.0);
                RunState::Targeting { item, cursor }
            }
        }
    }

    fn cast(&mut self, item: EntityId, center: Point) -> RunState {
        let Some(Effect::AreaDamage { amount, radius }) = self.world.items.get(item).map(|i| i.0)
        else {
            return RunState::ShowInventory;
        };
        let name = self.name_of(item);
        self.push_log(format!("You read the {name}."));
        self.events
            .extend(self.world.monsters.iter().filter_map(|(id, _)| {
                let pos = self.world.positions.get(id)?.0;
                (pos.dist2(center) <= radius * radius).then_some(Event::Damage {
                    target: id,
                    amount,
                    source: self.player,
                })
            }));
        self.events.push(Event::Consume { item });
        self.finish_player_turn()
    }

    fn perform(&mut self, actor: EntityId, action: Action) {
        match action {
            Action::Wait => {}
            Action::Move(dir) => self.try_move(actor, dir),
            Action::Pickup => self.pickup(actor),
        }
    }

    fn try_move(&mut self, actor: EntityId, dir: Dir) {
        let Some(from) = self.world.positions.get(actor).map(|p| p.0) else {
            return;
        };
        let to = from.step(dir);
        match self.world.blocker_at(to) {
            Some(other) => self.attack(actor, other),
            None if self.map.is_walkable(to) => {
                if let Some(p) = self.world.positions.get_mut(actor) {
                    p.0 = to;
                }
                if let Some(v) = self.world.viewsheds.get_mut(actor) {
                    v.dirty = true;
                }
            }
            None => {}
        }
    }

    /// 발밑의 아이템 하나를 가방에 넣는다. 바닥에서 사라지므로 Position을 지운다
    fn pickup(&mut self, actor: EntityId) {
        let Some(here) = self.world.positions.get(actor).map(|p| p.0) else {
            return;
        };
        let found = self
            .world
            .items
            .iter()
            .map(|(id, _)| id)
            .find(|&id| self.world.positions.get(id).is_some_and(|p| p.0 == here));
        match found {
            Some(item) => {
                self.world.positions.remove(item);
                self.world
                    .backpacks
                    .insert(item, InBackpack { owner: actor });
                let name = self.name_of(item);
                self.push_log(format!("You pick up the {name}."));
            }
            None => self.push_log("There is nothing here to pick up.".to_string()),
        }
    }

    fn attack(&mut self, attacker: EntityId, defender: EntityId) {
        // 플레이어와 몬스터 사이에서만 싸운다 (몬스터끼리는 서로 막기만 한다)
        if attacker != self.player && defender != self.player {
            return;
        }
        // 체력을 직접 바꾸지 않고, 일어나야 할 일을 큐에 넣는다
        if let Some(event) = combat::attack_event(&self.world, attacker, defender) {
            self.events.push(event);
        }
    }

    /// 쌓인 이벤트를 한 번에 적용한다. 플레이어가 죽었으면 사인(죽인 쪽의 이름)을 돌려준다
    fn apply_events(&mut self) -> Option<String> {
        let mut cause = None;
        let events = std::mem::take(&mut self.events);
        for event in events {
            match event {
                Event::Damage {
                    target,
                    amount,
                    source,
                } => {
                    let attacker = self.name_of(source);
                    let defender = self.name_of(target);
                    let Some(health) = self.world.healths.get_mut(target) else {
                        continue; // 같은 턴에 이미 죽은 대상
                    };
                    health.hp -= amount;
                    let dead = health.hp <= 0;
                    self.push_log(format!("{attacker} hits {defender} for {amount}."));
                    if dead {
                        self.push_log(format!("{defender} dies."));
                        if target == self.player {
                            // 플레이어는 지우지 않는다. 죽음은 상태(GameOver)로 표현한다
                            cause = Some(attacker);
                        } else {
                            self.world.despawn(target);
                        }
                    }
                }
                Event::Heal { target, amount } => {
                    let name = self.name_of(target);
                    if let Some(health) = self.world.healths.get_mut(target) {
                        health.hp = (health.hp + amount).min(health.max);
                        self.push_log(format!("{name} recovers {amount} HP."));
                    }
                }
                Event::Consume { item } => self.world.despawn(item),
            }
        }
        cause
    }

    /// 다시 계산이 필요한 시야만 계산한다. 플레이어가 본 타일은 "본 적 있음"으로 기록한다
    fn update_views(&mut self) {
        for (id, view) in self.world.viewsheds.iter_mut() {
            if !view.dirty {
                continue;
            }
            let Some(pos) = self.world.positions.get(id) else {
                continue;
            };
            view.visible = fov::compute(&self.map, pos.0, view.range);
            view.dirty = false;
            if id == self.player {
                for &p in &view.visible {
                    self.map.mark_explored(p);
                }
            }
        }
    }

    fn name_of(&self, id: EntityId) -> String {
        self.world
            .names
            .get(id)
            .map_or_else(|| "Something".to_string(), |n| n.0.clone()) // 로그에 남길 이름은 소유한 값이 필요하다
    }

    fn push_log(&mut self, line: String) {
        self.log.push(line);
        if self.log.len() > 100 {
            self.log.drain(..50);
        }
    }

    /// 모든 몬스터가 행동한다. 플레이어가 죽으면 사인을 돌려준다
    fn monsters_act(&mut self) -> Option<String> {
        // 순회하면서 perform 으로 월드를 고쳐야 하므로, ID 를 먼저 모아 빌림을 끝낸다
        let ids: Vec<EntityId> = self.world.monsters.iter().map(|(id, _)| id).collect();
        for id in ids {
            let action = ai::decide(&self.world, &self.map, id, self.player);
            self.perform(id, action);
            self.update_views();
            if let Some(cause) = self.apply_events() {
                return Some(cause);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Tile;

    /// 입력 하나를 넣고, 몬스터 차례까지 끝낸다
    fn act(game: &mut Game, action: Action) {
        game.update(Some(Input::Act(action)));
        game.update(None);
    }

    fn player_pos(game: &Game) -> Option<Point> {
        game.world.positions.get(game.player).map(|p| p.0)
    }

    /// 몬스터를 모두 치워서 플레이어만 남긴다
    fn clear_monsters(game: &mut Game) {
        let ids: Vec<EntityId> = game.world.monsters.iter().map(|(id, _)| id).collect();
        for id in ids {
            game.world.despawn(id);
        }
    }

    #[test]
    fn player_moves_and_stops_at_wall() {
        let mut game = Game::hand_made();
        clear_monsters(&mut game);
        act(&mut game, Action::Move(Dir::Right));
        assert_eq!(player_pos(&game), Some(Point::new(16, 10)));
        for _ in 0..30 {
            act(&mut game, Action::Move(Dir::Left));
        }
        assert_eq!(player_pos(&game), Some(Point::new(11, 10)));
    }

    #[test]
    fn moving_into_a_monster_attacks_it() {
        let mut game = Game::hand_made();
        clear_monsters(&mut game);
        let goblin = game.world.spawn_goblin(Point::new(16, 10));
        game.update(Some(Input::Act(Action::Move(Dir::Right))));
        // 이동하지 않고 공격한다: 공격 5 - 방어 1 = 4
        assert_eq!(player_pos(&game), Some(Point::new(15, 10)));
        assert_eq!(game.world.healths.get(goblin).map(|h| h.hp), Some(6));
        assert_eq!(
            game.log().first().map(String::as_str),
            Some("Player hits Goblin for 4.")
        );
    }

    #[test]
    fn monster_dies_and_is_removed() {
        let mut game = Game::hand_made();
        clear_monsters(&mut game);
        let goblin = game.world.spawn_goblin(Point::new(16, 10));
        if let Some(h) = game.world.healths.get_mut(goblin) {
            h.hp = 4;
        }
        game.update(Some(Input::Act(Action::Move(Dir::Right))));
        assert!(!game.world.entities.is_alive(goblin));
        assert!(game.world.blocker_at(Point::new(16, 10)).is_none());
        assert_eq!(game.log().last().map(String::as_str), Some("Goblin dies."));
    }

    #[test]
    fn monsters_approach_the_player() {
        let mut game = Game::hand_made();
        clear_monsters(&mut game);
        let goblin = game.world.spawn_goblin(Point::new(12, 10));
        act(&mut game, Action::Wait);
        assert_eq!(
            game.world.positions.get(goblin).map(|p| p.0),
            Some(Point::new(13, 10))
        );
    }

    #[test]
    fn adjacent_monster_attacks_the_player() {
        let mut game = Game::hand_made();
        clear_monsters(&mut game);
        game.world.spawn_goblin(Point::new(16, 10));
        act(&mut game, Action::Wait);
        // 공격 4 - 방어 2 = 2
        assert_eq!(game.player_hp(), 28);
        assert_eq!(game.log(), ["Goblin hits Player for 2."]);
    }

    #[test]
    fn minimum_damage_applies_in_game() {
        let mut game = Game::hand_made();
        clear_monsters(&mut game);
        let goblin = game.world.spawn_goblin(Point::new(16, 10));
        if let Some(c) = game.world.combats.get_mut(goblin) {
            c.defense = 99;
        }
        game.update(Some(Input::Act(Action::Move(Dir::Right))));
        assert_eq!(game.world.healths.get(goblin).map(|h| h.hp), Some(9));
    }

    #[test]
    fn no_input_changes_nothing() {
        let mut game = Game::hand_made();
        for _ in 0..10 {
            game.update(None);
        }
        assert_eq!(player_pos(&game), Some(Point::new(15, 10)));
        assert_eq!(game.state(), &RunState::AwaitingInput);
        assert!(game.log().is_empty());
    }

    #[test]
    fn turn_goes_through_monster_turn() {
        let mut game = Game::hand_made();
        game.update(Some(Input::Act(Action::Wait)));
        assert_eq!(game.state(), &RunState::MonsterTurn);
        game.update(None);
        assert_eq!(game.state(), &RunState::AwaitingInput);
    }

    #[test]
    fn player_death_leads_to_game_over() {
        let mut game = Game::hand_made();
        // 몬스터는 플레이어가 보여야 다가온다. 시야 안에 고블린 하나를 둔다
        game.world.spawn_goblin(Point::new(16, 10));
        for _ in 0..200 {
            act(&mut game, Action::Wait);
        }
        assert!(game.is_over());
        assert_eq!(
            game.state(),
            &RunState::GameOver {
                cause: "Goblin".to_string()
            }
        );
    }

    #[test]
    fn game_over_ignores_moves_and_restart_starts_over() {
        let mut game = Game::hand_made();
        // 몬스터는 플레이어가 보여야 다가온다. 시야 안에 고블린 하나를 둔다
        game.world.spawn_goblin(Point::new(16, 10));
        for _ in 0..200 {
            act(&mut game, Action::Wait);
        }
        let before = player_pos(&game);
        game.update(Some(Input::Act(Action::Move(Dir::Right))));
        assert!(game.is_over());
        assert_eq!(player_pos(&game), before);

        game.update(Some(Input::Restart));
        assert_eq!(game.state(), &RunState::AwaitingInput);
        assert_eq!(game.player_hp(), 30);
    }

    /// 플레이어 발밑으로 아이템을 옮겨 놓는다
    fn put_under_player(game: &mut Game, item: EntityId) {
        let here = player_pos(game).unwrap();
        game.world
            .positions
            .insert(item, crate::comps::Position(here));
    }

    #[test]
    fn pickup_moves_item_into_backpack() {
        let mut game = Game::hand_made();
        clear_monsters(&mut game);
        let potion = game.world.spawn_potion(Point::new(0, 0));
        put_under_player(&mut game, potion);
        act(&mut game, Action::Pickup);
        assert!(game.world.positions.get(potion).is_none());
        assert_eq!(game.inventory().collect::<Vec<_>>(), vec![potion]);
    }

    #[test]
    fn pickup_on_empty_floor_only_logs() {
        let mut game = Game::hand_made();
        clear_monsters(&mut game);
        act(&mut game, Action::Pickup);
        assert_eq!(
            game.log().last().map(String::as_str),
            Some("There is nothing here to pick up.")
        );
        assert_eq!(game.inventory().count(), 0);
    }

    #[test]
    fn potion_heals_up_to_max_and_is_consumed() {
        let mut game = Game::hand_made();
        clear_monsters(&mut game);
        let potion = game.world.spawn_potion(Point::new(0, 0));
        put_under_player(&mut game, potion);
        act(&mut game, Action::Pickup);
        game.world.healths.get_mut(game.player).unwrap().hp = 25;
        game.update(Some(Input::OpenInventory));
        game.update(Some(Input::Choose(0)));
        assert_eq!(game.state(), &RunState::MonsterTurn);
        assert_eq!(game.player_hp(), 30); // 25 + 12가 아니라 최대 체력까지만
        assert!(game.world.items.get(potion).is_none());
        assert_eq!(game.inventory().count(), 0);
    }

    #[test]
    fn opening_inventory_costs_no_turn() {
        let mut game = Game::hand_made();
        game.world.spawn_goblin(Point::new(17, 10));
        let before: Vec<_> = game
            .world
            .positions
            .iter()
            .map(|(id, p)| (id, *p))
            .collect();
        game.update(Some(Input::OpenInventory));
        assert_eq!(game.state(), &RunState::ShowInventory);
        game.update(Some(Input::Cancel));
        assert_eq!(game.state(), &RunState::AwaitingInput);
        // 몬스터 차례를 거치지 않았으므로 아무도 움직이지 않았다
        let after: Vec<_> = game
            .world
            .positions
            .iter()
            .map(|(id, p)| (id, *p))
            .collect();
        assert_eq!(before, after);
    }

    #[test]
    fn choosing_a_missing_slot_changes_nothing() {
        let mut game = Game::hand_made();
        clear_monsters(&mut game);
        game.update(Some(Input::OpenInventory));
        game.update(Some(Input::Choose(3)));
        assert_eq!(game.state(), &RunState::ShowInventory);
        assert_eq!(game.player_hp(), 30);
        assert!(game.log().is_empty());
    }

    #[test]
    fn scroll_goes_through_targeting_and_hits_area() {
        let mut game = Game::hand_made();
        clear_monsters(&mut game);
        let near = game.world.spawn_goblin(Point::new(17, 10));
        let far = game.world.spawn_goblin(Point::new(15, 15));
        let scroll = game.world.spawn_scroll(Point::new(0, 0));
        put_under_player(&mut game, scroll);
        act(&mut game, Action::Pickup);

        game.update(Some(Input::OpenInventory));
        game.update(Some(Input::Choose(0)));
        assert!(matches!(game.state(), RunState::Targeting { .. }));
        game.update(Some(Input::MoveCursor(Dir::Right)));
        game.update(Some(Input::MoveCursor(Dir::Right)));
        game.update(Some(Input::Confirm));
        assert_eq!(game.state(), &RunState::MonsterTurn);
        // 고블린 HP 10, 피해 8: 범위 안은 살아남고(2), 범위 밖은 그대로
        assert_eq!(game.world.healths.get(near).map(|h| h.hp), Some(2));
        assert_eq!(game.world.healths.get(far).map(|h| h.hp), Some(10));
        assert!(game.world.items.get(scroll).is_none());
    }

    #[test]
    fn cancelling_targeting_keeps_the_scroll() {
        let mut game = Game::hand_made();
        clear_monsters(&mut game);
        let scroll = game.world.spawn_scroll(Point::new(0, 0));
        put_under_player(&mut game, scroll);
        act(&mut game, Action::Pickup);
        game.update(Some(Input::OpenInventory));
        game.update(Some(Input::Choose(0)));
        assert!(matches!(game.state(), RunState::Targeting { .. }));
        game.update(Some(Input::Cancel));
        assert_eq!(game.state(), &RunState::ShowInventory);
        assert_eq!(game.inventory().collect::<Vec<_>>(), vec![scroll]);
    }

    #[test]
    fn cursor_cannot_leave_the_field_of_view() {
        let mut game = Game::hand_made();
        clear_monsters(&mut game);
        let scroll = game.world.spawn_scroll(Point::new(0, 0));
        put_under_player(&mut game, scroll);
        act(&mut game, Action::Pickup);
        game.update(Some(Input::OpenInventory));
        game.update(Some(Input::Choose(0)));
        for _ in 0..30 {
            game.update(Some(Input::MoveCursor(Dir::Left)));
        }
        let RunState::Targeting { cursor, .. } = game.state() else {
            panic!("should still be targeting");
        };
        assert!(game.can_see(*cursor));
    }

    /// 화면에 놓인 모든 것의 (위치, 글자). 두 판이 같은지 비교할 때 쓴다
    fn layout(game: &Game) -> Vec<(Point, char)> {
        game.world
            .positions
            .iter()
            .filter_map(|(id, p)| Some((p.0, game.world.glyphs.get(id)?.ch)))
            .collect()
    }

    #[test]
    fn same_seed_same_game() {
        let a = Game::new(42);
        let b = Game::new(42);
        assert_eq!(a.map(), b.map());
        assert_eq!(layout(&a), layout(&b));
    }

    #[test]
    fn different_seed_different_game() {
        let a = Game::new(1);
        let b = Game::new(2);
        assert_ne!(a.map(), b.map());
    }

    #[test]
    fn player_starts_on_floor_with_nothing_on_top() {
        for seed in 0..50 {
            let game = Game::new(seed);
            let here = player_pos(&game).expect("player has a position");
            assert_eq!(game.map().tile(here), Some(Tile::Floor), "seed {seed}");
            let others = game
                .world
                .positions
                .iter()
                .filter(|&(id, p)| id != game.player && p.0 == here)
                .count();
            assert_eq!(others, 0, "seed {seed}");
        }
    }

    #[test]
    fn restart_gives_a_new_dungeon_but_is_deterministic() {
        let mut a = Game::new(9);
        let mut b = Game::new(9);
        let first = a.map().clone();
        for game in [&mut a, &mut b] {
            game.state = RunState::GameOver {
                cause: "test".to_string(),
            };
            game.update(Some(Input::Restart));
        }
        assert_eq!(a.state(), &RunState::AwaitingInput);
        assert_ne!(a.map(), &first);
        assert_eq!(a.map(), b.map());
    }
}
