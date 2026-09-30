use macroquad::prelude::*;

#[macroquad::main("Rogue")]
async fn main() {
    loop {
        clear_background(BLACK);
        draw_text(rogue_core::hello(), 20.0, 40.0, 32.0, WHITE);
        next_frame().await;
    }
}
