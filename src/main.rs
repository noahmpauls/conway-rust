use clap::{Arg, App};
use regex::Regex;

use conway::{GameOfLife, SdlRender};

const DEFAULT_CELL_SIZE: usize = 5;

fn main() {
    let cli = App::new("Game of Life")
        .version("1.0")
        .author("Noah Pauls")
        .about("Simulator for Conway's Game of Life.")
        .after_help(
            "This program simulates Conway's Game of Life on a toroidal surface \
             (edges are connected). Use SPACE to play/pause the simulation, N \
             to single step the simulation while paused, and the arrow keys to \
             adjust the framerate/evolutions per frame of the simulation."
        )
        .arg(Arg::with_name("file")
            .help("the pattern file to start the game with; omit to use random pattern")
            .short("f")
            .long("file")
            .takes_value(true))
        .arg(Arg::with_name("dimensions")
            .help("the dimensions of the game grid in cells, as `{rows}x{cols}`")
            .short("d")
            .long("dimensions")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("cell_size")
            .help("the display size of each cell in pixels")
            .short("c")
            .long("cell")
            .takes_value(true));

    let matches = cli.get_matches();

    // get filename of game to start with
    let file = matches.value_of("file");

    // get dimensions of board
    let dimensions = matches.value_of("dimensions").unwrap();
    let re = Regex::new(r"(?P<rows>[\d]+)x(?P<cols>[\d]+)").unwrap();
    let dimensions = re.captures(dimensions).unwrap();
    let (rows, cols) = (dimensions.name("rows").unwrap().as_str(), dimensions.name("cols").unwrap().as_str());
    let (rows, cols) = (rows.parse::<usize>().unwrap(), cols.parse::<usize>().unwrap());

    // get size of cell
    let cell_size = match matches.value_of("cell_size") {
        Some(value) => value.parse::<usize>().unwrap(),
        None => DEFAULT_CELL_SIZE,
    };

    run(file, rows, cols, cell_size);
}

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use sdl2::video::Window;

fn run(file: Option<&str>, rows: usize, cols: usize, cell_size: usize) {
    // Initialize SDL window, canvas, and event pump.
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let (window_width, window_height) = (
        rows * cell_size,
        cols * cell_size,
    );
    let window = video_subsystem.window(
        "GoL", 
        window_height.try_into().unwrap(), 
        window_width.try_into().unwrap()
    ).position_centered().build().unwrap();
    let canvas : Canvas<Window> = window.into_canvas()
        .present_vsync()
        .build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    // Initialize game and renderer.
    let game = match file {
        Some(file) => GameOfLife::from_file(file, rows, cols),
        None => GameOfLife::random(rows, cols),
    };
    let mut renderer = SdlRender::new(game, canvas, cell_size);

    'render: loop {
        for event in event_pump.poll_iter() {
            match event {
                // Quit on ESC, Q, or close window.
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Q), .. } | 
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'render;
                },
                // Toggle play/pause with SPACE.
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => { 
                    match renderer.playing() {
                        true => renderer.pause(),
                        false => renderer.play(),
                    }
                },
                // Render frame by frame with N when paused.
                Event::KeyDown { keycode: Some(Keycode::N), .. } => { 
                    if !renderer.playing() {
                        renderer.step(1);
                    }
                },
                // Increase/decrease framerate with UP/DOWN arrows.
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => { 
                    renderer.inc_framerate();
                },
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => { 
                    renderer.dec_framerate();
                },
                // Increase/decrease generations per frame with RIGHT/LEFT arrows.
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => { 
                    renderer.inc_steps_per_frame();
                },
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => { 
                    renderer.dec_steps_per_frame();
                },
                _ => (),
            }
        }

        renderer.render();
    }
}
