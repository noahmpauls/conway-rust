use rand::Rng;
use std::collections::HashSet;
use std::string::ToString;
use std::fs;
use std::cmp::max;
use regex::Regex;

/// Represents a cell in the Game of Life board.
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct Cell {
    pub r: usize,  // cell row
    pub c: usize,  // cell column
}

/// Represents a Game of Life.
pub struct GameOfLife {
    pub rows: usize,
    pub cols: usize,
    live: HashSet<Cell>,
}

impl GameOfLife {
    /// Generate a game of a given size with a random set of live cells.
    pub fn random(rows: usize, cols: usize) -> GameOfLife {
        let mut rng = rand::thread_rng();
        let mut live = HashSet::new();
        for r in 0..rows {
            for c in 0..cols {
                if rng.gen_range(0..10) == 0 {
                    live.insert(Cell { r, c });
                }
            }
        }
        GameOfLife { rows, cols, live }
    }

    /// Generate a game of a given size from a pattern file, centering the
    /// pattern in the middle of the game space.
    pub fn from_file(path: &str, rows: usize, cols: usize) -> GameOfLife {
        let contents = fs::read_to_string(path).expect("error reading file");

        let chars = "chars";
        let coords = "coords";
        if contents.starts_with(chars) {
            Self::parse_chars(&contents, rows, cols)
        } else if contents.starts_with(coords) {
            Self::parse_coords(&contents, rows, cols)
        } else {
            panic!("error parsing file");
        }
    }

    fn parse_chars(file_contents: &str, rows: usize, cols: usize) -> GameOfLife {
        let re = Regex::new(r"\{(?P<dead>.)(?P<alive>.)\}").unwrap();
        let chars = re.captures(file_contents).unwrap();
        let dead = chars.name("dead").unwrap().as_str().chars().next().unwrap();
        let alive = chars.name("alive").unwrap().as_str().chars().next().unwrap();

        let re = Regex::new(&format!("(?m)^[{}{}]+", dead, alive)).unwrap();
        let lines = re.find_iter(file_contents);
        let mut live = HashSet::new();
        for (r, line) in lines.enumerate() {
            for (c, char) in line.as_str().chars().enumerate() {
                if char == alive {
                    live.insert(Cell { r, c });
                }
            }
        }

        live = Self::center_pattern(&live, rows, cols);

        GameOfLife { rows, cols, live }
    }

    fn parse_coords(file_contents: &str, rows: usize, cols: usize) -> GameOfLife {
        let re = Regex::new(r"\d+,\d+").unwrap();
        let coords = re.find_iter(file_contents);
        
        let mut live = HashSet::new();
        for coord in coords {
            let mut coord_iter = coord.as_str().split(",");
            let (r, c) = (coord_iter.next().unwrap(), coord_iter.next().unwrap());
            let (r, c) = (r.parse::<usize>().unwrap(), c.parse::<usize>().unwrap());
            live.insert(Cell { r, c });
        }

        live = Self::center_pattern(&live, rows, cols);

        GameOfLife { rows, cols, live }
    }

    fn center_pattern(pattern: &HashSet<Cell>, rows: usize, cols: usize) -> HashSet<Cell> {
        let (mut max_r, mut max_c) = (0, 0);
        for cell in pattern.iter() {
            max_r = max(max_r, cell.r);
            max_c = max(max_c, cell.c);
        }

        let r_shift = (rows - max_r) / 2;
        let c_shift = (cols - max_c) / 2;

        let mut centered = HashSet::new();
        for cell in pattern.iter() {
            centered.insert(Cell { r: cell.r + r_shift, c: cell.c + c_shift });
        }
        centered
    }

    /// Evolve one generation in the game.
    pub fn step(&mut self) {
        let mut next_live = HashSet::new();
        let mut dead_memo = HashSet::new();

        for cell in self.live.iter().copied() {
            self.scan_live(&cell, &mut next_live, &mut dead_memo);
        }

        self.live = next_live;
    }

    fn scan_live(&self, cell: &Cell, next_live: &mut HashSet<Cell>, dead_memo: &mut HashSet<Cell>) {
        let mut live_neighbors = 0;

        let (neighbor_r, neighbor_c) = self.range_wrap(cell.r, cell.c);
        for r in neighbor_r.iter().copied() {
            for c in neighbor_c.iter().copied() {
                let neighbor = Cell { r, c };
                if *cell == neighbor {
                    continue;
                }
                if self.is_live(&neighbor) {
                    live_neighbors += 1;
                } else if !dead_memo.contains(&neighbor) {
                    self.scan_dead(&neighbor, next_live);
                    dead_memo.insert(neighbor);
                }
            }
        }

        match live_neighbors {
            2 | 3 => {next_live.insert(*cell);},
            _ => (),
        }
    }

    fn scan_dead(&self, cell: &Cell, next_live: &mut HashSet<Cell>) {
        let mut live_neighbors = 0;

        let (neighbor_r, neighbor_c) = self.range_wrap(cell.r, cell.c);
        for r in neighbor_r.iter().copied() {
            for c in neighbor_c.iter().copied() {
                let neighbor = Cell { r, c };
                if *cell == neighbor {
                    continue;
                }
                if self.is_live(&neighbor) {
                    live_neighbors += 1;
                }
            }
        }

        match live_neighbors {
            3 => {next_live.insert(*cell);},
            _ => (),
        }
    }

    fn range_wrap(&self, r: usize, c: usize) -> ([usize; 3], [usize; 3]) {
        let (r_max, c_max) = (self.rows - 1, self.cols - 1);
        (
            if r == 0 {
                [r_max, 0, 1]
            } else if r == r_max {
                [r_max-1, r_max, 0]
            } else {
                [r-1, r, r+1]
            },

            if c == 0 {
                [c_max, 0, 1]
            } else if c == c_max {
                [c_max-1, c_max, 0]
            } else {
                [c-1, c, c+1]
            },
        )
    }

    fn is_live (&self, cell: &Cell) -> bool {
        self.live.contains(cell)
    }

    /// Get all cells that are currently alive in the game.
    pub fn live_cells(&self) -> Vec<Cell> {
        self.live.iter().copied().collect()
    }
}

impl ToString for GameOfLife {
    fn to_string(&self) -> String {
        let mut res = String::new();
        for r in 0..self.rows {
            for c in 0..self.cols {
                if self.is_live(&Cell { r: r.try_into().unwrap(), c: c.try_into().unwrap() }) {
                    res.push('█');
                } else {
                    res.push(' ');
                }
            }
            if r < self.rows - 1 {
                res.push('\n');
            }
        }
        res
    }
}
