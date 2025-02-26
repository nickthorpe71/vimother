mod utils;

use std::io::{ self, Read, Write };
use std::os::unix::io::AsRawFd;
use libc::{ termios, tcgetattr, tcsetattr, TCSANOW, ECHO, ICANON };
use utils::{ roll, roll_range, random_precept };

const MAP_WIDTH: i8 = 80;
const MAP_HEIGHT: i8 = 40;

#[derive(Debug, Copy, Clone)]
struct Vector2 {
    x: i8,
    y: i8,
}

impl Vector2 {
    fn new(x: i8, y: i8) -> Self {
        Self { x, y }
    }

    fn add(self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

struct Game {
    player_pos: Vector2,
    map: [[char; MAP_WIDTH as usize]; MAP_HEIGHT as usize],
}

impl Game {
    fn new() -> Self {
        let mut map = [['.'; MAP_WIDTH as usize]; MAP_HEIGHT as usize];

        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                if y == 0 || y == MAP_HEIGHT - 1 || x == 0 || x == MAP_WIDTH - 1 {
                    map[y as usize][x as usize] = '█'; // Border walls
                } else if roll(10) {
                    map[y as usize][x as usize] = '~'; // Random water
                }
            }
        }

        Self {
            player_pos: Vector2::new(MAP_WIDTH / 2, MAP_HEIGHT / 2),
            map,
        }
    }

    fn draw_map(&self) {
        print!("\x1B[H"); // Move cursor to the top-left corner

        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let current_tile_symbol = if x == self.player_pos.x && y == self.player_pos.y {
                    "\x1B[36m@\x1B[0m".to_string()
                } else {
                    self.map[y as usize][x as usize].to_string()
                };

                print!("{}", current_tile_symbol);
            }
            println!();
        }
        println!("Use hjkl to move. Press 'q' to quit.");
        io::stdout().flush().unwrap();
    }

    fn update(&mut self) -> bool {
        let mut input = [0; 1];
        io::stdin().read_exact(&mut input).unwrap();
        let key = input[0];

        match key {
            b'k' => self.handle_move_input(0, -1),
            b'j' => self.handle_move_input(0, 1),
            b'l' => self.handle_move_input(1, 0),
            b'h' => self.handle_move_input(-1, 0),
            b'q' => {
                return false;
            }
            _ => {}
        }

        true
    }

    fn handle_move_input(&mut self, dir_x: i8, dir_y: i8) {
        // Determine which tile we are trying to move to
        let start_pos = self.player_pos;
        let new_x = (start_pos.x + dir_x).clamp(0, (MAP_WIDTH as i8) - 1);
        let new_y = (start_pos.y + dir_y).clamp(0, (MAP_HEIGHT as i8) - 1);
        let target_tile: char = self.map[new_y as usize][new_x as usize];

        // Prevent movement if the tile is not trabersable
        let non_traversable: &[char] = &['█', '~'];
        if non_traversable.contains(&target_tile) {
            return;
        }

        // Move the player
        let new_pos = Vector2::new(new_x, new_y);
        self.player_pos = new_pos;

        // Check for encounter
        if roll_range(15, 35) {
            self.gen_encounter();
        }
    }

    fn gen_encounter(&mut self) {
        let precept = random_precept();
        println!("\r{} | {} | {}", precept, self.player_pos.x, self.player_pos.y);
    }
}

// Enable raw mode (disable line buffering & echo)
fn enable_raw_mode() {
    let stdin_fd = io::stdin().as_raw_fd();
    let mut termios = termios {
        c_lflag: 0,
        ..unsafe {
            std::mem::zeroed()
        }
    };

    unsafe {
        tcgetattr(stdin_fd, &mut termios);
        termios.c_lflag &= !(ICANON | ECHO); // Disable line buffering & echo
        tcsetattr(stdin_fd, TCSANOW, &termios);
    }
}

// Restore default terminal mode
fn disable_raw_mode() {
    let stdin_fd = io::stdin().as_raw_fd();
    let mut termios = termios {
        c_lflag: 0,
        ..unsafe {
            std::mem::zeroed()
        }
    };

    unsafe {
        tcgetattr(stdin_fd, &mut termios);
        termios.c_lflag |= ICANON | ECHO; // Re-enable line buffering & echo
        tcsetattr(stdin_fd, TCSANOW, &termios);
    }
}

fn main() {
    enable_raw_mode(); // Enable raw input mode

    print!("\x1B[2J"); // Clear the screen once at the start
    io::stdout().flush().unwrap();

    let mut game = Game::new();

    loop {
        game.draw_map();
        if !game.update() {
            break;
        }
    }

    disable_raw_mode(); // Restore terminal mode
    println!("Game Over!");
}
