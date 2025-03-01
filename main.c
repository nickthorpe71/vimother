#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <unistd.h>
#include <termios.h>
#include <fcntl.h>
#include <string.h>

#define MAP_WIDTH 80
#define MAP_HEIGHT 40
#define DIALOG_HEIGHT 10

typedef struct {
    int x, y;
} Vector2;

typedef enum {
    OVERWORLD,
    BATTLE_TRANSITION,
    IN_BATTLE
} GameState;

typedef struct {
    GameState game_state;
    Vector2 player_pos;
    char map[MAP_HEIGHT][MAP_WIDTH];
} Game;

void enable_raw_mode() {
    struct termios term;
    tcgetattr(STDIN_FILENO, &term);
    term.c_lflag &= ~(ICANON | ECHO);
    tcsetattr(STDIN_FILENO, TCSANOW, &term);
}

void disable_raw_mode() {
    struct termios term;
    tcgetattr(STDIN_FILENO, &term);
    term.c_lflag |= ICANON | ECHO;
    tcsetattr(STDIN_FILENO, TCSANOW, &term);
}

char get_input() {
    char ch;
    read(STDIN_FILENO, &ch, 1);
    return ch;
}

int roll(int chance) {
    return (rand() % 100) < chance;
}

int roll_range(int min, int max) {
    return (rand() % (max - min + 1) + min) < 35;
}

void init_game(Game* game) {
    srand(time(NULL));
    game->game_state = OVERWORLD;
    game->player_pos.x = MAP_WIDTH / 2;
    game->player_pos.y = MAP_HEIGHT / 2;
    
    for (int y = 0; y < MAP_HEIGHT; y++) {
        for (int x = 0; x < MAP_WIDTH; x++) {
            if (y == 0 || y == MAP_HEIGHT - 1 || x == 0 || x == MAP_WIDTH - 1) {
                game->map[y][x] = '#';
            } else if (roll(10)) {
                game->map[y][x] = '~';
            } else {
                game->map[y][x] = ' ';
            }
        }
    }
}

void draw_map(Game* game) {
    printf("\033[H"); // Move cursor to top-left corner
    for (int y = 0; y < MAP_HEIGHT; y++) {
        for (int x = 0; x < MAP_WIDTH; x++) {
            if (x == game->player_pos.x && y == game->player_pos.y) {
                printf("\033[36m@\033[0m");
            } else {
                putchar(game->map[y][x]);
            }
        }
        putchar('\n');
    }
    fflush(stdout);
}

void handle_move_input(Game* game, int dx, int dy) {
    if (game->game_state != OVERWORLD) return;
    
    int new_x = game->player_pos.x + dx;
    int new_y = game->player_pos.y + dy;
    
    if (new_x < 0 || new_x >= MAP_WIDTH || new_y < 0 || new_y >= MAP_HEIGHT) return;
    char target_tile = game->map[new_y][new_x];
    
    if (target_tile == '#' || target_tile == '~') return;
    
    game->player_pos.x = new_x;
    game->player_pos.y = new_y;
    
    if (roll_range(15, 35)) {
        printf("Encounter! Player at (%d, %d)\n", new_x, new_y);
    }
}

int update(Game* game) {
    char key = get_input();
    switch (key) {
        case 'k': handle_move_input(game, 0, -1); break;
        case 'j': handle_move_input(game, 0, 1); break;
        case 'l': handle_move_input(game, 1, 0); break;
        case 'h': handle_move_input(game, -1, 0); break;
        case 'q': return 0;
        default: break;
    }
    return 1;
}

int main() {
    enable_raw_mode();
    printf("\033[2J"); // Clear screen
    
    Game game;
    init_game(&game);
    
    while (1) {
        draw_map(&game);
        if (!update(&game)) break;
    }
    
    disable_raw_mode();
    printf("Game Over!\n");
    return 0;
}
