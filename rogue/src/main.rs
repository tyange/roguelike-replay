use macroquad::prelude::*;
use rogue_core::{Action, Dir, Game, Input, Point, Rgb, RunState, Tile};

const TILE: f32 = 20.0;

const NUMBER_KEYS: [KeyCode; 9] = [
    KeyCode::Key1,
    KeyCode::Key2,
    KeyCode::Key3,
    KeyCode::Key4,
    KeyCode::Key5,
    KeyCode::Key6,
    KeyCode::Key7,
    KeyCode::Key8,
    KeyCode::Key9,
];

fn tile_color(tile: Tile) -> Color {
    match tile {
        Tile::Wall => DARKGRAY,
        Tile::Floor => Color::new(0.15, 0.15, 0.2, 1.0),
    }
}

fn dim(c: Color) -> Color {
    Color::new(c.r * 0.4, c.g * 0.4, c.b * 0.4, c.a)
}

/// core의 색을 macroquad의 색으로. 변환은 실행 파일 쪽에 둔다
fn to_color(c: Rgb) -> Color {
    Color::from_rgba(c.r, c.g, c.b, 255)
}

/// 이번 프레임에 눌린 방향키
fn read_direction() -> Option<Dir> {
    if is_key_pressed(KeyCode::Up) {
        Some(Dir::Up)
    } else if is_key_pressed(KeyCode::Down) {
        Some(Dir::Down)
    } else if is_key_pressed(KeyCode::Left) {
        Some(Dir::Left)
    } else if is_key_pressed(KeyCode::Right) {
        Some(Dir::Right)
    } else {
        None
    }
}

/// 같은 키라도 지금 상태에 따라 뜻이 다르다. 상태마다 키 해석을 따로 둔다
fn read_input(state: &RunState) -> Option<Input> {
    match state {
        RunState::AwaitingInput => {
            if let Some(dir) = read_direction() {
                Some(Input::Act(Action::Move(dir)))
            } else if is_key_pressed(KeyCode::Space) {
                Some(Input::Act(Action::Wait))
            } else if is_key_pressed(KeyCode::G) {
                Some(Input::Act(Action::Pickup))
            } else if is_key_pressed(KeyCode::I) {
                Some(Input::OpenInventory)
            } else {
                None
            }
        }
        RunState::ShowInventory => {
            if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::I) {
                Some(Input::Cancel)
            } else {
                NUMBER_KEYS
                    .iter()
                    .position(|&k| is_key_pressed(k))
                    .map(Input::Choose)
            }
        }
        RunState::Targeting { .. } => {
            if let Some(dir) = read_direction() {
                Some(Input::MoveCursor(dir))
            } else if is_key_pressed(KeyCode::Enter) {
                Some(Input::Confirm)
            } else if is_key_pressed(KeyCode::Escape) {
                Some(Input::Cancel)
            } else {
                None
            }
        }
        RunState::MonsterTurn => None,
        RunState::GameOver { .. } => is_key_pressed(KeyCode::R).then_some(Input::Restart),
    }
}

/// 시드는 실행 파일 쪽의 일이다: core는 시계를 모른다
fn fresh_seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as u64)
}

fn draw_glyph(p: Point, ch: char, color: Color) {
    draw_text(
        ch.to_string(),
        p.x as f32 * TILE + 5.0,
        (p.y + 1) as f32 * TILE - 5.0,
        TILE,
        color,
    );
}

#[macroquad::main("Rogue")]
async fn main() {
    let mut game = Game::new(fresh_seed());

    loop {
        let input = read_input(game.state());
        game.update(input);

        clear_background(BLACK);

        let map = game.map();
        for y in 0..map.height() {
            for x in 0..map.width() {
                let p = Point::new(x, y);
                let visible = game.can_see(p);
                // 지금 보이지도 않고 본 적도 없으면 그리지 않는다
                if !visible && !map.is_explored(p) {
                    continue;
                }
                if let Some(tile) = map.tile(p) {
                    let color = if visible {
                        tile_color(tile)
                    } else {
                        dim(tile_color(tile))
                    };
                    draw_rectangle(
                        x as f32 * TILE,
                        y as f32 * TILE,
                        TILE - 1.0,
                        TILE - 1.0,
                        color,
                    );
                }
            }
        }

        // 위치와 모양이 둘 다 있는 것은 모두 그린다. 플레이어인지 몬스터인지는 묻지 않는다
        let world = game.world();
        for (id, pos) in world.positions.iter() {
            if !game.can_see(pos.0) {
                continue;
            }
            if let Some(glyph) = world.glyphs.get(id) {
                draw_glyph(pos.0, glyph.ch, to_color(glyph.color));
            }
        }

        let base_y = map.height() as f32 * TILE;
        draw_text(
            format!("HP {}", game.player_hp()),
            10.0,
            base_y + 24.0,
            28.0,
            WHITE,
        );

        // 최근 전투 메시지 4줄
        for (i, line) in game.log().iter().rev().take(4).enumerate() {
            draw_text(line, 10.0, base_y + 50.0 + i as f32 * 20.0, 20.0, LIGHTGRAY);
        }

        // 상태에 따라 무엇을 더 그릴지 정한다. 와일드카드 없이 모든 상태를 적는다
        match game.state() {
            RunState::AwaitingInput | RunState::MonsterTurn => {}
            RunState::ShowInventory => {
                draw_rectangle(40.0, 40.0, 440.0, 320.0, Color::new(0.0, 0.0, 0.0, 0.85));
                draw_text("Inventory (1-9 use, Esc close)", 60.0, 75.0, 26.0, WHITE);
                for (i, id) in game.inventory().enumerate() {
                    let name = world.names.get(id).map_or("?", |n| n.0.as_str());
                    draw_text(
                        format!("{}) {name}", i + 1),
                        60.0,
                        110.0 + i as f32 * 26.0,
                        24.0,
                        LIGHTGRAY,
                    );
                }
            }
            RunState::Targeting { cursor, .. } => {
                draw_rectangle_lines(
                    cursor.x as f32 * TILE,
                    cursor.y as f32 * TILE,
                    TILE,
                    TILE,
                    2.0,
                    YELLOW,
                );
                draw_text(
                    "Arrows move, Enter fire, Esc back",
                    10.0,
                    20.0,
                    24.0,
                    YELLOW,
                );
            }
            RunState::GameOver { cause } => {
                let mid_y = screen_height() / 2.0;
                draw_text(
                    format!("You died. Killed by {cause}."),
                    40.0,
                    mid_y,
                    36.0,
                    RED,
                );
                draw_text("Press R to restart", 40.0, mid_y + 40.0, 28.0, WHITE);
            }
        }

        next_frame().await;
    }
}
