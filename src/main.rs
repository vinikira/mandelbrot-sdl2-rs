extern crate num;
extern crate rayon;
extern crate sdl2;

use num::complex::Complex64;
use rayon::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::WindowCanvas;
use std::time::Duration;

struct GameState {
    canvas_size: usize,
    max_iterations: usize,
    colours: Vec<Color>,
    magnification_value: f64,
    pan_x: f64,
    pan_y: f64,
}

impl GameState {
    fn rainbow_colors(&mut self) {
        let c = self.max_iterations;

        let sin_to_dec = |i: u32, phase: f64| -> u8 {
            let s = (std::f64::consts::PI / (c as f64) * 2.0 * (i as f64) + phase).sin();
            (((s * 127.0) + 128.0).floor()) as u8
        };

        self.colours = (0..c as u32)
            .map(|i| {
                Color::RGBA(
                    sin_to_dec(i, 0.0 * std::f64::consts::PI * 2.0 / 3.0),
                    sin_to_dec(i, 2.0 * std::f64::consts::PI * 2.0 / 3.0),
                    sin_to_dec(i, 1.0 * std::f64::consts::PI * 2.0 / 3.0),
                    255,
                )
            })
            .collect()
    }

    fn mandelbrot(&self, x: f64, y: f64) -> usize {
        let a = Complex64::new(x, y);
        let mut i = 0;
        let mut z = a;

        while z.norm_sqr() < 4.0 && i < self.max_iterations {
            i += 1;
            z = z * z + a;
        }

        i
    }

    fn render(&mut self, canvas: &mut WindowCanvas) {
        let pixels: Vec<u8> = (0..self.canvas_size.pow(2))
            .into_par_iter()
            .enumerate()
            .map(|(n, _pixel)| {
                let y = (n as u32) / self.canvas_size as u32;
                let x = (n as u32) - (y * self.canvas_size as u32);
                let re = x as f64 / self.magnification_value - self.pan_x;
                let im = y as f64 / self.magnification_value - self.pan_y;

                self.mandelbrot(re, im) as u8
            })
            .collect();

        pixels.into_iter().enumerate().for_each(|(n, iterations)| {
            let y = (n as u32) / self.canvas_size as u32;
            let x = (n as u32) - (y * self.canvas_size as u32);

            let color = if iterations < self.max_iterations as u8 {
                self.colours[iterations as usize]
            } else {
                Color::RGB(0, 0, 0)
            };

            canvas.set_draw_color(color);

            canvas.draw_point(Point::new(x as i32, y as i32)).unwrap();
        });
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            canvas_size: 800,
            max_iterations: 60,
            colours: vec![],
            magnification_value: 800f64,
            pan_x: 0f64,
            pan_y: 0f64,
        }
    }
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let mut game_state = GameState::default();

    let window = video_subsystem
        .window(
            "mandelbrot rust+sdl2",
            game_state.canvas_size as u32,
            game_state.canvas_size as u32,
        )
        .position_centered()
        .build()
        .unwrap();

    let mut canvas: WindowCanvas = window.into_canvas().accelerated().build().unwrap();

    game_state.rainbow_colors();

    game_state.render(&mut canvas);

    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    game_state.pan_x += 0.05f64;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    game_state.pan_x -= 0.05f64;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    game_state.pan_y += 0.05f64;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    game_state.pan_y -= 0.05f64;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    game_state.magnification_value += 100f64;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    game_state.magnification_value -= 100f64;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => {
                    game_state.magnification_value = 800f64;
                    game_state.pan_x = 0f64;
                    game_state.pan_y = 0f64;
                }
                _ => {}
            }
        }

        game_state.render(&mut canvas);
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
