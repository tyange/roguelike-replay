use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::action::Action;
use crate::ai;
use crate::combat;
use crate::comps::{Effect, InBackpack};
use crate::defs::Defs;
use crate::entity::EntityId;
use crate::error::DataError;
use crate::event::Event;
use crate::floor::{Floor, generate_floor};
use crate::fov;
use crate::geom::{Dir, Point};
use crate::map::{Map, Tile};
use crate::rng::Rng;
use crate::state::{Input, RunState};
use crate::world::World;

const SAVE_VERSION: u32 = 1;

/// 저장할 때의 모양. 게임을 복사하지 않도록 빌려서 감싼다
#[derive(Serialize)]
struct SaveRef<'a> {
    version: u32,
    game: &'a Game,
}

/// 본문보다 먼저 버전만 읽는다. 나머지 필드는 무시된다
#[derive(Deserialize)]
struct SaveHeader {
    version: u32,
}

/// 읽을 때의 모양. 읽은 게임은 소유한 값이어야 한다
#[derive(Deserialize)]
struct SaveOwned {
    game: Game,
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    map: Map,
    world: World,
    player: EntityId,
    events: Vec<Event>,
    log: Vec<String>,
    state: RunState,
    rng: Rng,
    depth: u32,
    turns: u32,
    kills: u32,
    /// 정의 목록은 저장하지 않는다. 불러올 때 데이터 파일에서 다시 받는다
    #[serde(skip)]
    defs: Arc<Defs>,
    /// 다른 스레드가 만들고 있는 다음 층을 받을 곳. 저장할 수 없는 값이다
    #[serde(skip)]
    pending: Option<Receiver<Floor>>,
    /// 층 생성에 일부러 넣는 지연. 테스트와 눈으로 확인하는 용도
    #[serde(skip)]
    generation_delay: Duration,
}

impl Game {
    /// 시드 하나로 던전 한 판이 정해진다. 같은 시드는 항상 같은 판이다
    pub fn new(seed: u64, defs: Arc<Defs>) -> Game {
        let mut rng = Rng::new(seed);
        let floor = generate_floor(rng.next_u64(), 0, &defs);
        let mut world = World::default();
        let player = world.spawn_player(floor.start);
        let mut game = Game {
            map: Map::solid(0, 0), // 바로 아래 apply_floor 가 첫 층으로 바꾼다
            world,
            player,
            events: Vec::new(),
            log: Vec::new(),
            state: RunState::AwaitingInput,
            rng,
            depth: 0,
            turns: 0,
            kills: 0,
            defs,
            pending: None,
            generation_delay: Duration::ZERO,
        };
        game.apply_floor(floor);
        game
    }

    /// 3 단계까지 쓰던 손으로 만든 방. 기존 테스트를 그대로 돌리기 위해 남긴다
    #[cfg(test)]
    pub fn hand_made() -> Game {
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
            depth: 0,
            turns: 0,
            kills: 0,
            defs: crate::defs::builtin_arc(),
            pending: None,
            generation_delay: Duration::ZERO,
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

    /// 지금 몇 번째 층인가. 처음은 0
    pub fn depth(&self) -> u32 {
        self.depth
    }

    /// 플레이어가 쓴 턴 수
    pub fn turns(&self) -> u32 {
        self.turns
    }

    /// 플레이어가 죽인 몬스터 수
    pub fn kills(&self) -> u32 {
        self.kills
    }

    /// 층 생성 스레드에 넣을 지연. 생성 중에도 창이 반응하는지 눈으로 확인할 때 쓴다
    pub fn set_generation_delay(&mut self, delay: Duration) {
        self.generation_delay = delay;
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

    /// 게임 전체를 문자열로. 시야처럼 다시 계산할 수 있는 값은 빠진다
    pub fn to_save_string(&self) -> Result<String, DataError> {
        Ok(ron::to_string(&SaveRef {
            version: SAVE_VERSION,
            game: self,
        })?)
    }

    /// 저장 문자열에서 게임을 되살린다. 망가졌거나 버전이 다르면 에러
    pub fn from_save_string(text: &str, defs: Arc<Defs>) -> Result<Game, DataError> {
        // 버전부터 본다: 형식이 바뀐 옛 파일은 본문 해석에서 먼저 실패할 수 있기 때문이다
        let header: SaveHeader = ron::from_str(text)?;
        if header.version != SAVE_VERSION {
            return Err(DataError::Version {
                found: header.version,
                expected: SAVE_VERSION,
            });
        }
        let SaveOwned { mut game } = ron::from_str(text)?;
        if game.world.positions.get(game.player).is_none() {
            return Err(DataError::Invalid {
                what: "save file".to_string(),
                reason: "the player entity is missing".to_string(),
            });
        }
        game.defs = defs;
        if game.state == RunState::Generating {
            // 받을 곳이 사라졌으니, 계단 위에서 다시 내려가게 한다
            game.state = RunState::AwaitingInput;
        }
        // 저장하지 않은 시야를 다시 계산하게 한다
        for (_, view) in game.world.viewsheds.iter_mut() {
            view.dirty = true;
        }
        game.update_views();
        Ok(game)
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
            RunState::Generating => self.poll_generation(),
            RunState::GameOver { cause } => match input {
                Some(Input::Restart) => {
                    // 같은 수열 위에서 다음 판을 만든다. core 는 시계를 모른다
                    let seed = self.rng.next_u64();
                    // 새 판에도 정의가 필요하다. 목록을 복사하지 않고 공유한다
                    *self = Game::new(seed, Arc::clone(&self.defs));
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
        if self.pending.is_some() {
            // 계단을 내려갔다. 다음 층이 올 때까지 몬스터도 움직이지 않는다
            self.turns += 1;
            return RunState::Generating;
        }
        self.finish_player_turn()
    }

    fn finish_player_turn(&mut self) -> RunState {
        self.turns += 1;
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
            Action::Descend => {
                if actor == self.player {
                    self.begin_descent();
                }
            }
        }
    }

    /// 다음 층 생성을 다른 스레드에 맡긴다. 필요한 값은 모두 스레드로 옮긴다(move)
    fn begin_descent(&mut self) {
        let on_stairs = self
            .world
            .positions
            .get(self.player)
            .is_some_and(|p| self.map.tile(p.0) == Some(Tile::Stairs));
        if !on_stairs {
            self.push_log("There are no stairs here.".to_string());
            return;
        }
        let seed = self.rng.next_u64();
        let depth = self.depth + 1;
        let defs = Arc::clone(&self.defs); // 큰 정의 목록은 복사하지 않고 공유한다
        let delay = self.generation_delay;
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            thread::sleep(delay);
            let floor = generate_floor(seed, depth, &defs);
            let _ = tx.send(floor); // 받는 쪽이 사라졌으면 보낼 곳이 없을 뿐이다
        });
        self.pending = Some(rx);
        self.push_log("You descend...".to_string());
    }

    /// 막지 않고 한 번만 확인한다. 아직이면 계속 기다리는 상태로 남는다
    fn poll_generation(&mut self) -> RunState {
        let Some(rx) = &self.pending else {
            return RunState::AwaitingInput;
        };
        let result = rx.try_recv();
        match result {
            Ok(floor) => {
                self.pending = None;
                self.apply_floor(floor);
                self.push_log(format!("Depth {}.", self.depth));
                RunState::AwaitingInput
            }
            Err(TryRecvError::Empty) => RunState::Generating,
            Err(TryRecvError::Disconnected) => {
                self.pending = None;
                self.push_log("The stairs collapsed (floor generation failed).".to_string());
                RunState::AwaitingInput
            }
        }
    }

    fn apply_floor(&mut self, floor: Floor) {
        // 위치가 있는 엔티티는 바닥의 몬스터와 아이템이다. 가방 속 아이템에는 위치가 없다
        let doomed: Vec<EntityId> = self
            .world
            .positions
            .iter()
            .map(|(id, _)| id)
            .filter(|&id| id != self.player)
            .collect();
        for id in doomed {
            self.world.despawn(id);
        }
        if let Some(pos) = self.world.positions.get_mut(self.player) {
            pos.0 = floor.start;
        }
        if let Some(view) = self.world.viewsheds.get_mut(self.player) {
            view.dirty = true;
        }
        for (index, p) in floor.monsters {
            if let Some(def) = self.defs.monsters.get(index) {
                self.world.spawn_monster(def, p);
            }
        }
        for (index, p) in floor.items {
            if let Some(def) = self.defs.items.get(index) {
                self.world.spawn_item(def, p);
            }
        }
        self.map = floor.map;
        self.depth = floor.depth;
        self.update_views();
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
                            if source == self.player {
                                self.kills += 1;
                            }
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

    /// 화면 아래 메시지 줄에 한 줄을 남긴다 (실행 파일이 불러오기 실패를 알리는 데도 쓴다)
    pub fn push_log(&mut self, line: String) {
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
        let a = Game::new(42, crate::defs::builtin_arc());
        let b = Game::new(42, crate::defs::builtin_arc());
        assert_eq!(a.map(), b.map());
        assert_eq!(layout(&a), layout(&b));
    }

    #[test]
    fn different_seed_different_game() {
        let a = Game::new(1, crate::defs::builtin_arc());
        let b = Game::new(2, crate::defs::builtin_arc());
        assert_ne!(a.map(), b.map());
    }

    #[test]
    fn player_starts_on_floor_with_nothing_on_top() {
        for seed in 0..50 {
            let game = Game::new(seed, crate::defs::builtin_arc());
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
        let mut a = Game::new(9, crate::defs::builtin_arc());
        let mut b = Game::new(9, crate::defs::builtin_arc());
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

    fn play_script(game: &mut Game, steps: usize) {
        let moves = [Dir::Right, Dir::Down, Dir::Left, Dir::Up];
        for i in 0..steps {
            act(game, Action::Move(moves[i % 4]));
            if i % 5 == 0 {
                act(game, Action::Wait);
            }
        }
    }

    #[test]
    fn save_and_load_roundtrip() {
        let mut game = Game::new(11, crate::defs::builtin_arc());
        play_script(&mut game, 12);
        let text = game.to_save_string().unwrap();
        let loaded = Game::from_save_string(&text, crate::defs::builtin_arc()).unwrap();
        assert_eq!(loaded.to_save_string().unwrap(), text);
        assert_eq!(player_pos(&loaded), player_pos(&game));
        // 시야는 저장하지 않았지만 불러오면서 다시 계산된다
        let here = player_pos(&loaded).unwrap();
        assert!(loaded.can_see(here));
    }

    #[test]
    fn loaded_game_continues_exactly_like_the_original() {
        let mut original = Game::new(21, crate::defs::builtin_arc());
        play_script(&mut original, 8);
        let text = original.to_save_string().unwrap();
        let mut loaded = Game::from_save_string(&text, crate::defs::builtin_arc()).unwrap();
        play_script(&mut original, 40);
        play_script(&mut loaded, 40);
        assert_eq!(
            original.to_save_string().unwrap(),
            loaded.to_save_string().unwrap()
        );
    }

    #[test]
    fn garbage_save_is_an_error_not_a_panic() {
        for text in ["", "hello", "(version: 1)", "(version: 1, game: 5)"] {
            assert!(Game::from_save_string(text, crate::defs::builtin_arc()).is_err());
        }
    }

    #[test]
    fn wrong_version_is_reported_as_version_error() {
        let game = Game::new(3, crate::defs::builtin_arc());
        let text = game.to_save_string().unwrap();
        let old = text.replacen(&format!("version:{SAVE_VERSION}"), "version:99", 1);
        assert_ne!(old, text);
        match Game::from_save_string(&old, crate::defs::builtin_arc()) {
            Err(DataError::Version { found, expected }) => {
                assert_eq!((found, expected), (99, 1));
            }
            Err(other) => panic!("expected Version, got {other:?}"),
            Ok(_) => panic!("expected Version, got a game"),
        }
    }

    #[test]
    fn save_without_a_player_is_invalid() {
        let mut game = Game::new(3, crate::defs::builtin_arc());
        game.world.despawn(game.player);
        let text = game.to_save_string().unwrap();
        let err = Game::from_save_string(&text, crate::defs::builtin_arc()).err();
        assert!(matches!(err, Some(DataError::Invalid { .. })));
    }

    fn teleport_to_stairs(game: &mut Game) {
        let stairs = game.map.find(Tile::Stairs).expect("every floor has stairs");
        game.world.positions.get_mut(game.player).unwrap().0 = stairs;
    }

    /// 결과가 올 때까지 update(None)을 반복한다 (테스트 전용: 실제 게임은 프레임마다 한 번씩 부른다)
    fn wait_until_generated(game: &mut Game) {
        for _ in 0..2000 {
            if game.state() != &RunState::Generating {
                return;
            }
            game.update(None);
            std::thread::sleep(Duration::from_millis(1));
        }
        panic!("floor generation never finished");
    }

    #[test]
    fn descending_builds_the_next_floor_in_the_background() {
        let mut game = Game::new(5, crate::defs::builtin_arc());
        game.set_generation_delay(Duration::from_millis(150));
        let player = game.player();
        teleport_to_stairs(&mut game);
        let before = game.map().clone();

        let started = std::time::Instant::now();
        game.update(Some(Input::Act(Action::Descend)));
        assert!(
            started.elapsed() < Duration::from_millis(100),
            "update must not block"
        );
        assert_eq!(game.state(), &RunState::Generating);

        game.update(None); // 아직 도착 전: 계속 기다린다
        assert_eq!(game.state(), &RunState::Generating);

        wait_until_generated(&mut game);
        assert_eq!(game.state(), &RunState::AwaitingInput);
        assert_eq!(game.depth(), 1);
        assert_ne!(game.map(), &before);
        assert_eq!(game.player(), player); // 같은 플레이어 엔티티가 이어진다
        assert!(game.can_see(player_pos(&game).unwrap()));
    }

    #[test]
    fn same_seed_gives_the_same_floors_regardless_of_timing() {
        let run = |delay_ms: u64| {
            let mut game = Game::new(77, crate::defs::builtin_arc());
            game.set_generation_delay(Duration::from_millis(delay_ms));
            for _ in 0..3 {
                teleport_to_stairs(&mut game);
                game.update(Some(Input::Act(Action::Descend)));
                wait_until_generated(&mut game);
            }
            (game.depth(), game.map().clone())
        };
        assert_eq!(run(0), run(20));
    }

    #[test]
    fn monsters_do_not_act_while_generating() {
        let mut game = Game::new(5, crate::defs::builtin_arc());
        game.set_generation_delay(Duration::from_millis(100));
        teleport_to_stairs(&mut game);
        game.update(Some(Input::Act(Action::Descend)));
        assert_eq!(game.player_hp(), 30);
        game.update(Some(Input::Act(Action::Wait))); // 생성 중에는 입력이 무시된다
        assert_eq!(game.state(), &RunState::Generating);
    }

    #[test]
    fn descend_off_the_stairs_only_logs() {
        let mut game = Game::new(5, crate::defs::builtin_arc());
        clear_monsters(&mut game);
        act(&mut game, Action::Descend);
        assert_eq!(game.state(), &RunState::AwaitingInput);
        assert!(game.log().iter().any(|line| line.contains("no stairs")));
        assert_eq!(game.depth(), 0);
    }

    #[test]
    fn inventory_survives_the_trip() {
        let mut game = Game::new(5, crate::defs::builtin_arc());
        clear_monsters(&mut game);
        let potion = game.world.spawn_potion(Point::new(0, 0));
        put_under_player(&mut game, potion);
        act(&mut game, Action::Pickup);
        teleport_to_stairs(&mut game);
        game.update(Some(Input::Act(Action::Descend)));
        wait_until_generated(&mut game);
        assert_eq!(game.depth(), 1);
        assert_eq!(game.inventory().collect::<Vec<_>>(), vec![potion]);
    }
}
