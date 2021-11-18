use std::thread;
use std::time::{Duration, Instant};
use sdl2::video::Window;
use sdl2::render::Canvas;
use sdl2::rect::Rect;
use sdl2::pixels::Color;

use crate::GameOfLife;

const DEFAULT_FRAMERATE: u128 = 24;
const MAX_FRAMERATE: u128 = 120;

const DEFAULT_STEPS_PER_FRAME: usize = 1;
const MAX_STEPS_PER_FRAME: usize = 50;


/// Struct to render a GameOfLife using SDL.
pub struct SdlRender {
    game: GameOfLife,  // game to render
    canvas: Canvas<Window>,  // SDL canvas to draw on
    cell_size: usize,  // side length of square cell, in pixels
    play: bool,  // whether calling self.render() causes game steps
    framerate: u128,  // maximum framerate of render
    min_render_nanos: u128,  // minimum time per render step based on framerate
    steps_per_frame: usize,  // how many game steps to take on each frame
    step_count: u128,  // number of steps taken so far
}

impl SdlRender {
    /// Create a new instance of a renderer with the given game to render,
    /// canvas to draw on, and size to draw cells at.
    pub fn new(game: GameOfLife, canvas: Canvas<Window>, cell_size: usize) -> SdlRender {
        SdlRender {
            game, canvas, cell_size,
            play: false,
            framerate: DEFAULT_FRAMERATE,
            min_render_nanos: 1_000_000_000 / DEFAULT_FRAMERATE,
            steps_per_frame: DEFAULT_STEPS_PER_FRAME,
            step_count: 0,
        }
    }

    /// Render the game state on the canvas, and advance the game state if the
    /// renderer is currently playing.
    pub fn render(&mut self) {
        let time = Instant::now();

        // Render the game.
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();
        self.canvas.set_draw_color(Color::WHITE);
        for cell in self.game.live_cells() {
            let (x, y) = (cell.c * self.cell_size, cell.r * self.cell_size);
            let cell = Rect::new(
                x.try_into().unwrap(), 
                y.try_into().unwrap(),
                self.cell_size.try_into().unwrap(),
                self.cell_size.try_into().unwrap(),
            );
            if let Err(message) = self.canvas.fill_rect(cell) {
                eprintln!("failed to draw rect {:?}: {}", cell, message);
            }
        }
        self.canvas.present();

        // Advance the game state.
        if self.play {
            for _ in 0..self.steps_per_frame {
                self.game.step();
            }
            self.step_count += u128::try_from(self.steps_per_frame).unwrap();
        }
        let steps = self.step_count;

        // Update the canvas window title to reflect current render settings.
        let framerate = if self.framerate == MAX_FRAMERATE + 1 {
            String::from("max")
        } else {
            format!("{}", self.framerate)
        };
        let iters = self.steps_per_frame;
        if let Err(message) = self.canvas.window_mut().set_title(&format!(
            "Gol | {} | FPS: {} | Evolutions Per Frame: {}",
            steps,
            framerate,
            iters)
        ) {
            eprintln!("failed to change window title: `{}`", message);
        }

        // Block to achieve desired framerate.
        let elapsed = time.elapsed().as_nanos();
        if self.play && elapsed < self.min_render_nanos {
            thread::sleep(Duration::from_nanos((self.min_render_nanos - elapsed).try_into().unwrap()));
        }
    }
    
    /// Tell the renderer to advance the game state after a render.
    pub fn play(&mut self) {
        self.play = true;
    }

    /// Tell the renderer to only display the current game state and not advance
    /// it.
    pub fn pause(&mut self) {
        self.play = false;
    }

    /// Whether this renderer advances the game state after rendering.
    pub fn playing(&self) -> bool {
        self.play
    }

    /// Increase the framerate by 1 FPS, up to a max value.
    pub fn inc_framerate(&mut self) {
        if self.framerate < MAX_FRAMERATE {
            self.framerate += 1;
            self.min_render_nanos = 1_000_000_000 / self.framerate;
        } else if self.framerate == MAX_FRAMERATE {
            self.framerate += 1;
            self.min_render_nanos = 0;
        }
    }

    /// Decrease the framerate by 1 FPS, down to a minimum of 1 FPS.
    pub fn dec_framerate(&mut self) {
        if self.framerate > 1 {
            self.framerate -= 1;
            self.min_render_nanos = 1_000_000_000 / self.framerate;
        }
    }

    /// Increase the number of game states advanced after rendering by 1, up to
    /// a max value.
    pub fn inc_steps_per_frame(&mut self) {
        if self.steps_per_frame < MAX_STEPS_PER_FRAME {
            self.steps_per_frame += 1;
        }
    }

    /// Decrease the number of game states advanced after rendering by 1, down
    /// to a minimum of 1.
    pub fn dec_steps_per_frame(&mut self) {
        if self.steps_per_frame > 1 {
            self.steps_per_frame -= 1;
        }
    }

    /// Step the game state by `step_count` independent of rendering or playing.
    pub fn step(&mut self, step_count: usize) {
        for _ in 0..step_count {
            self.game.step();
        }
        self.step_count += u128::try_from(step_count).unwrap();
    }
}
