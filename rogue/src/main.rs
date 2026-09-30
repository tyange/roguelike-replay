use macroquad::prelude::*;
use rogue_core::{Action, Dir, Game, Input, Point, Rgb, RunState, Tile};

const TILE: f32 = 24.0;

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

/// 이번 프레임에 눌린 키를 core의 입력으로 바꾼다
fn read_input() -> Option<Input> {
    if is_key_pressed(KeyCode::Up) {
        Some(Input::Act(Action::Move(Dir::Up)))
    } else if is_key_pressed(KeyCode::Down) {
        Some(Input::Act(Action::Move(Dir::Down)))
    } else if is_key_pressed(KeyCode::Left) {
        Some(Input::Act(Action::Move(Dir::Left)))
    } else if is_key_pressed(KeyCode::Right) {
        Some(Input::Act(Action::Move(Dir::Right)))
    } else if is_key_pressed(KeyCode::Space) {
        Some(Input::Act(Action::Wait))
    } else if is_key_pressed(KeyCode::R) {
        Some(Input::Restart)
    } else {
        None
    }
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
    let mut game = Game::new();

    loop {
        game.update(read_input());

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
