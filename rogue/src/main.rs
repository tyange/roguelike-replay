use macroquad::prelude::*;
use rogue_core::{Map, Point, Tile};

const TILE: f32 = 24.0;

fn tile_color(tile: Tile) -> Color {
    match tile {
        Tile::Wall => DARKGRAY,
        Tile::Floor => Color::new(0.15, 0.15, 0.2, 1.0),
    }
}

#[macroquad::main("Rogue")]
async fn main() {
    let map = Map::room(30, 20);

    loop {
        clear_background(BLACK);

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

        next_frame().await;
    }
}
