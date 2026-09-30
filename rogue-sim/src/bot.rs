use rogue_core::{Action, Dir, Effect, Game, Input, Point, RunState, Tile, astar};

/// 봇의 정책. 월드를 읽기만 하고 입력 하나를 고른다 (3-2 의 몬스터 AI 와 같은 모양).
/// 입력이 필요 없는 상태(몬스터 차례, 층 생성 중, 게임오버)에서는 None
pub fn choose(game: &Game) -> Option<Input> {
    match game.state() {
        RunState::AwaitingInput => Some(on_map(game)),
        RunState::ShowInventory => Some(match heal_slot(game) {
            Some(slot) => Input::Choose(slot),
            None => Input::Cancel,
        }),
        RunState::Targeting { .. } => Some(Input::Cancel),
        RunState::MonsterTurn | RunState::Generating | RunState::GameOver { .. } => None,
    }
}

fn on_map(game: &Game) -> Input {
    let world = game.world();
    let Some(here) = world.positions.get(game.player()).map(|p| p.0) else {
        return Input::Act(Action::Wait);
    };
    let hp = game.player_hp();
    let max = world.healths.get(game.player()).map_or(1, |h| h.max);

    // 1. 체력이 바닥나면 물약
    if hp * 3 < max && heal_slot(game).is_some() {
        return Input::OpenInventory;
    }
    // 2. 보이는 몬스터 중 가장 가까운 것과 싸운다
    let nearest = world
        .monsters
        .iter()
        .filter_map(|(id, _)| world.positions.get(id))
        .map(|p| p.0)
        .filter(|&p| game.can_see(p))
        .min_by_key(|&p| p.dist2(here));
    if let Some(target) = nearest
        && let Some(step) = next_step(game, here, target)
    {
        return Input::Act(Action::Move(step));
    }
    // 3. 발밑의 아이템 줍기
    let item_here = world
        .items
        .iter()
        .any(|(id, _)| world.positions.get(id).is_some_and(|p| p.0 == here));
    if item_here {
        return Input::Act(Action::Pickup);
    }
    // 4. 계단으로
    if game.map().tile(here) == Some(Tile::Stairs) {
        return Input::Act(Action::Descend);
    }
    if let Some(stairs) = game.map().find(Tile::Stairs)
        && let Some(step) = next_step(game, here, stairs)
    {
        return Input::Act(Action::Move(step));
    }
    Input::Act(Action::Wait)
}

/// to 로 가는 최단 경로의 첫 걸음 방향
fn next_step(game: &Game, from: Point, to: Point) -> Option<Dir> {
    let path = astar(game.map(), from, to)?;
    Dir::between(from, *path.first()?)
}

/// 인벤토리에서 회복 물약이 몇 번째 칸인가
fn heal_slot(game: &Game) -> Option<usize> {
    game.inventory().position(|id| {
        matches!(
            game.world().items.get(id).map(|item| item.0),
            Some(Effect::Heal(_))
        )
    })
}
