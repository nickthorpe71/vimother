mod utils;

use std::io::{ self, Read, Write };
use std::os::unix::io::AsRawFd;
use libc::{ termios, tcgetattr, tcsetattr, TCSANOW, ECHO, ICANON };
use utils::{ roll, roll_range, random_precept };
use std::thread::sleep;
use std::time::Duration;

const MAP_WIDTH: i8 = 80;
const MAP_HEIGHT: i8 = 40;
const DIALOG_HEIGHT: i8 = 10;

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

#[derive(PartialEq)]
enum GameState {
    OverWorld,
    BattleTransition,
    InBattle,
}

struct Game {
    game_state: GameState,
    player_pos: Vector2,
    map: [[char; MAP_WIDTH as usize]; MAP_HEIGHT as usize],
}

impl Game {
    fn new() -> Self {
        let mut map = [[' '; MAP_WIDTH as usize]; MAP_HEIGHT as usize];

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
            game_state: GameState::OverWorld,
        }
    }

    fn draw_map(&self, map: [[char; MAP_WIDTH as usize]; MAP_HEIGHT as usize]) {
        print!("\x1B[H"); // Move cursor to the top-left corner

        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let current_tile_symbol = if x == self.player_pos.x && y == self.player_pos.y {
                    "\x1B[36m@\x1B[0m".to_string()
                } else {
                    map[y as usize][x as usize].to_string()
                };

                print!("{}", current_tile_symbol);
            }
            println!();
        }
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
        // This can likely be moved into a "over_world_movement"
        if self.game_state != GameState::OverWorld {
            return;
        }
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
        if roll_range(2, 10) {
            self.gen_encounter();
        }
    }

    fn gen_encounter(&mut self) {
        self.game_state = GameState::BattleTransition;

        self.draw_encounter_transition();

        self.game_state = GameState::OverWorld;

        let precept = random_precept();
        let text = format!("{} | {} | {}", precept, self.player_pos.x, self.player_pos.y);
        self.draw_dialog(&vec![text]);
    }

    fn draw_dialog(&mut self, pages: &[String]) {
        if pages.is_empty() {
            return;
        }

        let page_1 = &pages[0];

        for i in 0..DIALOG_HEIGHT - 1 {
            for j in 0..MAP_WIDTH {
                if j == 0 || j == MAP_WIDTH - 1 {
                    print!("|");
                } else if i == 1 && j < (page_1.len() as i8) + 1 {
                    print!(
                        "{}",
                        page_1
                            .chars()
                            .nth((j - 1) as usize)
                            .unwrap_or(' ')
                    );
                } else {
                    print!(" ");
                }
            }
            println!();
        }

        // Print bottom border
        for _ in 0..MAP_WIDTH {
            print!("-");
        }
        println!(); // Move to the next line

        io::stdout().flush().unwrap();
    }

    fn draw_encounter_transition(&mut self) {
        let start = self.player_pos;
        let mut x = start.x as i16;
        let mut y = start.y as i16;
        let mut map_clone = self.map.clone();
        let map_w_16 = MAP_WIDTH as i16;
        let map_h_16 = MAP_HEIGHT as i16;
        let mut draw_speed = 90;

        let dirs: [(i16, i16); 4] = [
            (1, 0),
            (0, 1),
            (-1, 0),
            (0, -1),
        ]; // Right, Down, Left, Up
        let mut step_count = 1;
        let mut dir_index = 0;

        while step_count < map_w_16.max(map_h_16) / 2 {
            let (dx, dy) = dirs[dir_index % 4];

            for _ in 0..step_count {
                x += dx;
                y += dy;

                if x >= 0 && x < map_w_16 && y >= 0 && y < map_h_16 {
                    map_clone[y as usize][x as usize] = '*';
                }

                self.draw_map(map_clone);
                sleep(Duration::from_millis(draw_speed));
            }

            dir_index += 1;

            // Increase step size every two turns to create a spiral effect
            if dir_index % 2 == 0 {
                step_count += 1;
                draw_speed = ((draw_speed as f32) * 0.7) as u64;
            }
        }

        // Turn entire screen to '*'
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                map_clone[y as usize][x as usize] = '*';
            }
        }
        self.draw_map(map_clone);
        sleep(Duration::from_millis(1200));
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
        game.draw_map(game.map);
        if !game.update() {
            break;
        }
    }

    disable_raw_mode(); // Restore terminal mode
    println!("Game Over!");
}
