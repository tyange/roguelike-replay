use macroquad::prelude::*;
use rogue_core::{Action, Dir, Game, Point, Tile};

const TILE: f32 = 24.0;

fn tile_color(tile: Tile) -> Color {
    match tile {
        Tile::Wall => DARKGRAY,
        Tile::Floor => Color::new(0.15, 0.15, 0.2, 1.0),
    }
}

/// 이번 프레임에 눌린 키를 core의 행동으로 바꾼다
fn read_action() -> Option<Action> {
    if is_key_pressed(KeyCode::Up) {
        Some(Action::Move(Dir::Up))
    } else if is_key_pressed(KeyCode::Down) {
        Some(Action::Move(Dir::Down))
    } else if is_key_pressed(KeyCode::Left) {
        Some(Action::Move(Dir::Left))
    } else if is_key_pressed(KeyCode::Right) {
        Some(Action::Move(Dir::Right))
    } else if is_key_pressed(KeyCode::Space) {
        Some(Action::Wait)
    } else {
        None
    }
}

#[macroquad::main("Rogue")]
async fn main() {
    let mut game = Game::new();

    loop {
        if let Some(action) = read_action() {
            game.step(action);
        }

        clear_background(BLACK);

        let map = game.map();
        for y in 0..map.height() {
            for x in 0..map.width() {
                if let Some(tile) = map.tile(Point::new(x, y)) {
                    draw_rectangle(
                        x as f32 * TILE,
                        y as f32 * TILE,
                        TILE - 1.0,
                        TILE - 1.0,
                        tile_color(tile),
                    );
                }
            }
        }

        let p = game.player();
        draw_text(
            "@",
            p.x as f32 * TILE + 5.0,
            (p.y + 1) as f32 * TILE - 5.0,
            TILE,
            YELLOW,
        );

        next_frame().await;
    }
}
