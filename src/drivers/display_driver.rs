extern crate sdl2;

use sdl2::pixels;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::processor::CHIP8_WIDTH;
use crate::processor::CHIP8_HEIGHT;

const SCALE_FACTOR: u32 = 20;
const SCREEN_WIDTH: u32 = (CHIP8_WIDTH as u32) * SCALE_FACTOR;
const SCREEN_HEIGHT: u32 = (CHIP8_HEIGHT as u32) * SCALE_FACTOR;

pub struct DisplayDriver {
    canvas: Canvas<Window>,
}

impl DisplayDriver {

    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("rust-sdl2 demo: Video", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        DisplayDriver {
            canvas
        }
    }

    fn color(&self, value :u8) -> pixels::Color {
        if value == 0 {
            pixels::Color::RGB(0, 0, 0)
        } else {
            pixels::Color::RGB(0, 255, 0)
        }
    }

    pub fn draw(&mut self, pixels: &[[u8; CHIP8_WIDTH]; CHIP8_HEIGHT]) {

        for (y, &row) in pixels.iter().enumerate() {
            let sy = y as u32 * SCALE_FACTOR;

            for (x, &col) in row.iter().enumerate() {
                let sx = x as u32 * SCALE_FACTOR;
                
                self.canvas.set_draw_color(self.color(col));
                let _ = self.canvas.fill_rect(Rect::new(sx as i32, sy as i32, SCALE_FACTOR, SCALE_FACTOR));
            } 
        }

        self.canvas.present();
    }

}