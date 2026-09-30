use crate::action::Action;
use crate::combat;
use crate::entity::EntityId;
use crate::event::Event;
use crate::geom::{Dir, Point};
use crate::map::Map;
use crate::state::{Input, RunState};
use crate::world::World;

pub struct Game {
    map: Map,
    world: World,
    player: EntityId,
    events: Vec<Event>,
    log: Vec<String>,
    state: RunState,
}

impl Game {
    pub fn new() -> Game {
        let mut world = World::default();
        let player = world.spawn_player(Point::new(15, 10));
        for pos in [Point::new(5, 5), Point::new(24, 4), Point::new(20, 15)] {
            world.spawn_goblin(pos, player);
        }
        Game {
            map: Map::room(30, 20),
            world,
            player,
            events: Vec::new(),
            log: Vec::new(),
            state: RunState::AwaitingInput,
        }
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

    /// 매 프레임 한 번 부른다. 현재 상태를 값으로 꺼내(소비) 다음 상태를 만들어 되돌려 넣는다
    pub fn update(&mut self, input: Option<Input>) {
        let current = std::mem::replace(&mut self.state, RunState::AwaitingInput);
        self.state = match current {
            RunState::AwaitingInput => match input {
                Some(Input::Act(action)) => {
                    self.perform(self.player, action);
                    match self.apply_events() {
                        Some(cause) => RunState::GameOver { cause },
                        None => RunState::MonsterTurn,
                    }
                }
                Some(Input::Restart) | None => RunState::AwaitingInput,
            },
            RunState::MonsterTurn => match self.monsters_act() {
                Some(cause) => RunState::GameOver { cause },
                None => RunState::AwaitingInput,
            },
            RunState::GameOver { cause } => match input {
                Some(Input::Restart) => {
                    *self = Game::new();
                    return;
                }
                Some(Input::Act(_)) | None => RunState::GameOver { cause },
            },
        };
    }

    fn perform(&mut self, actor: EntityId, action: Action) {
        let Action::Move(dir) = action else {
            return; // Wait
        };
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
            }
            None => {}
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
            }
        }
        cause
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
            let Some(target) = self.world.targets.get(id).map(|t| t.0) else {
                continue;
            };
            // 대상이 이미 사라졌으면 None이 나온다
            let Some(goal) = self.world.positions.get(target).map(|p| p.0) else {
                continue;
            };
            let Some(from) = self.world.positions.get(id).map(|p| p.0) else {
                continue;
            };
            self.perform(id, Action::Move(step_toward(from, goal)));
            if let Some(cause) = self.apply_events() {
                return Some(cause);
            }
        }
        None
    }
}

impl Default for Game {
    fn default() -> Self {
        Game::new()
    }
}

/// 목표 쪽으로 한 칸 (거리가 먼 축 우선)
fn step_toward(from: Point, to: Point) -> Dir {
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    if dx.abs() >= dy.abs() {
        if dx > 0 { Dir::Right } else { Dir::Left }
    } else if dy > 0 {
        Dir::Down
    } else {
        Dir::Up
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
        let mut game = Game::new();
        clear_monsters(&mut game);
        act(&mut game, Action::Move(Dir::Right));
        assert_eq!(player_pos(&game), Some(Point::new(16, 10)));
        for _ in 0..30 {
            act(&mut game, Action::Move(Dir::Left));
        }
        assert_eq!(player_pos(&game), Some(Point::new(1, 10)));
    }

    #[test]
    fn moving_into_a_monster_attacks_it() {
        let mut game = Game::new();
        clear_monsters(&mut game);
        let goblin = game.world.spawn_goblin(Point::new(16, 10), game.player);
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
        let mut game = Game::new();
        clear_monsters(&mut game);
        let goblin = game.world.spawn_goblin(Point::new(16, 10), game.player);
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
        let mut game = Game::new();
        clear_monsters(&mut game);
        let goblin = game.world.spawn_goblin(Point::new(5, 10), game.player);
        act(&mut game, Action::Wait);
        assert_eq!(
            game.world.positions.get(goblin).map(|p| p.0),
            Some(Point::new(6, 10))
        );
    }

    #[test]
    fn adjacent_monster_attacks_the_player() {
        let mut game = Game::new();
        clear_monsters(&mut game);
        game.world.spawn_goblin(Point::new(16, 10), game.player);
        act(&mut game, Action::Wait);
        // 공격 4 - 방어 2 = 2
        assert_eq!(game.player_hp(), 28);
        assert_eq!(game.log(), ["Goblin hits Player for 2."]);
    }

    #[test]
    fn minimum_damage_applies_in_game() {
        let mut game = Game::new();
        clear_monsters(&mut game);
        let goblin = game.world.spawn_goblin(Point::new(16, 10), game.player);
        if let Some(c) = game.world.combats.get_mut(goblin) {
            c.defense = 99;
        }
        game.update(Some(Input::Act(Action::Move(Dir::Right))));
        assert_eq!(game.world.healths.get(goblin).map(|h| h.hp), Some(9));
    }

    #[test]
    fn no_input_changes_nothing() {
        let mut game = Game::new();
        for _ in 0..10 {
            game.update(None);
        }
        assert_eq!(player_pos(&game), Some(Point::new(15, 10)));
        assert_eq!(game.state(), &RunState::AwaitingInput);
        assert!(game.log().is_empty());
    }

    #[test]
    fn turn_goes_through_monster_turn() {
        let mut game = Game::new();
        game.update(Some(Input::Act(Action::Wait)));
        assert_eq!(game.state(), &RunState::MonsterTurn);
        game.update(None);
        assert_eq!(game.state(), &RunState::AwaitingInput);
    }

    #[test]
    fn player_death_leads_to_game_over() {
        let mut game = Game::new();
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
        let mut game = Game::new();
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
}
