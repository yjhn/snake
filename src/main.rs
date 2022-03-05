use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    style::Print,
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand, Result,
};
use rand::prelude::*;
use std::{
    fmt::Write,
    io::{stdout, Error, Write as IOWrite},
    time::Duration,
};

#[derive(Debug, PartialEq, Clone, Copy)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum BodyPartDirection {
    Vertical,
    Horizontal,
    TopLeftCorner,
    TopRightCorner,
    BottomLeftCorner,
    BottomRightCorner,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Tile {
    Empty,
    Food(FoodType), // variable is for the type of food
    Obstacle,
    SnakePart(SnakePart),
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum FoodType {
    Blob,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum SnakePart {
    Head(Direction), // bool = true => vertical
    Body(BodyPartDirection),
    Tail(Direction),
}

const BOARD_WIDTH: usize = 50;
const BOARD_HEIGHT: usize = 20;
const GAME_STEP_LENGTH: u64 = 500;
const MAX_FOOD_ON_BOARD: u32 = 20;

type Board = Vec<Vec<Tile>>;
type Snake = Vec<SnakeTile>;

impl Direction {
    fn opposite_direction(&self) {
        match self {
            Self::Up => Direction::Down,
            Self::Right => Direction::Left,
            Self::Down => Direction::Up,
            Self::Left => Direction::Right,
        };
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct SnakeTile {
    x: usize,
    y: usize,
    snake_tile_type: SnakePart,
}

/* snake tiles need to
 1. have some way of knowing where they are moving - include direction in the tile
 2. whether they are corner tiles - need to somehow know the tile ahead,
    then the tile can adopt tile type of the tile ahead of it
          (maybe a Vec of all snake tiles with positions and all?)
 then snake tiles don't need to be stored in the general buffer and only need to be
 added before printing to the screen -- this could cause performance issues with huge snakes
*/

fn main() -> Result<()> {
    let mut out = stdout();
    let snake: Snake = vec![
        SnakeTile {
            x: 10,
            y: 10,
            snake_tile_type: SnakePart::Head(Direction::Left),
        },
        SnakeTile {
            x: 11,
            y: 10,
            snake_tile_type: SnakePart::Body(BodyPartDirection::Horizontal),
        },
        SnakeTile {
            x: 12,
            y: 10,
            snake_tile_type: SnakePart::Tail(Direction::Left),
        },
    ];

    let board: Board = vec![vec![Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT];

    terminal::enable_raw_mode()?;
    out.queue(terminal::EnterAlternateScreen)?;
    out.queue(cursor::MoveTo(0, 0))?;
    out.flush()?;

    game_loop(snake, board)?;

    terminal::disable_raw_mode()?;
    out.execute(terminal::LeaveAlternateScreen)
        .map(|_| Ok(()))?
}

fn game_loop(mut snake: Snake, mut board: Board) -> Result<()> {
    let mut out = stdout();
    let mut rng = rand::thread_rng();

    loop {
        for _ in 0..5 {
            add_food(&mut board, &mut rng);
        }
        add_snake_to_board(&mut board, &snake);
        draw(&board, &mut out)?;
        remove_snake_from_board(&mut board, &snake);
        // to listen to button presses, use non-blocking IO with poll
        // it will also sleep the program for the right duration
        match read_char_non_blocking()? {
            Some(c) => match c {
                'q' => return Ok(()),
                _ => {
                    std::thread::sleep(Duration::from_secs(2));
                }
            },
            None => (),
        }
    }
}

fn add_snake_to_board(board: &mut Board, snake: &Snake) {
    for tile in snake {
        board[tile.y][tile.x] = Tile::SnakePart(tile.snake_tile_type);
    }
}

fn remove_snake_from_board(board: &mut Board, snake: &Snake) {
    for tile in snake {
        board[tile.y][tile.x] = Tile::Empty;
    }
}

// snake wraps around board edges
fn move_snake(snake: &mut Snake) {
    for tile in snake {
        let SnakeTile {
            snake_tile_type: mut tile_type,
            mut x,
            mut y,
        } = *tile;
        match tile_type {
            SnakePart::Head(direction) => match direction {
                _ => todo!(),
            },
            SnakePart::Body(direction) => match direction {
                _ => todo!(),
            },
            SnakePart::Tail(direction) => match direction {
                _ => todo!(),
            },
        }
    }
}

fn read_char_blocking() -> Result<char> {
    match event::read()? {
        Event::Key(KeyEvent { code: key, .. }) => match key {
            KeyCode::Char(c) => {
                // \r - return to line start
                // \n - start a new line
                print!("input: {c}\r\n");
                Ok(c)
            }
            _ => Ok('e'),
        },
        Event::Mouse(_) => {
            print!("mouse event\r\n");
            Ok('m')
        }
        Event::Resize(x, y) => {
            print!("new size: {x}, {y}\r\n");
            Ok('r')
        }
    }
}

fn read_char_non_blocking() -> Result<Option<char>> {
    if event::poll(Duration::from_millis(GAME_STEP_LENGTH))? {
        match event::read()? {
            Event::Key(KeyEvent { code: key, .. }) => match key {
                KeyCode::Char(c) => {
                    // \r - return to line start
                    // \n - start a new line
                    print!("input: {c}\r\n");
                    Ok(Some(c))
                }
                _ => Ok(Some('e')),
            },
            Event::Mouse(_) => {
                print!("mouse event\r\n");
                Ok(Some('m'))
            }
            Event::Resize(x, y) => {
                print!("new size: {x}, {y}\r\n");
                Ok(Some('r'))
            }
        }
    } else {
        Ok(None)
    }
}

// adds one food particle at random location
// food is only added to empty tile
fn add_food(board: &mut Board, rng: &mut ThreadRng) {
    if count_food(&board) >= MAX_FOOD_ON_BOARD {
        return;
    }

    let height = board.len();
    let width = board[0].len();
    let (a, b): (usize, usize) = rng.gen();
    let (mut x, mut y) = (a % width, b % height);

    // make sure coordinates are on the board
    // x is width, y is height
    // (0, 0) is the top left corner

    if board[y][x] == Tile::Empty {
        board[y][x] = Tile::Food(FoodType::Blob);
    } else {
        // this will loop endlessly if there are no free tiles
        while board[y][x] != Tile::Empty {
            x = rng.gen::<usize>() % width;
            y = rng.gen::<usize>() % height;
        }

        board[y][x] = Tile::Food(FoodType::Blob);
    }
}

fn count_food(board: &Board) -> u32 {
    board.iter().flatten().fold(0, |count, tile| {
        if *tile == Tile::Food(FoodType::Blob) {
            count + 1
        } else {
            count
        }
    })
}

fn draw(board: &Board, out: &mut impl IOWrite) -> Result<()> {
    out.queue(terminal::Clear(ClearType::All))?;
    out.queue(cursor::MoveTo(0, 0))?;

    // let height = board.len();
    let width = board[0].len();

    // top line of the board
    let top = "-".repeat(width + 2);
    out.queue(Print(format!("{top}\r\n")))?;

    for row in board {
        out.queue(Print(format!(
            "|{}|\r\n",
            row.iter()
                .fold(String::with_capacity(width), |mut line, tile| {
                    write!(&mut line, "{}", get_char(tile)).unwrap();
                    line
                })
        )))?;
    }

    // bottom line
    out.queue(Print(format!("{top}\r\nPress any key to exit...")))?;
    out.flush()?;

    Ok(())
}

// snake is drawn using Box Drawing Unicode char block
fn get_char(tile: &Tile) -> char {
    match *tile {
        Tile::Empty => ' ',
        Tile::Food(_) => '*',
        Tile::Obstacle => '@',
        Tile::SnakePart(snake_part) => match snake_part {
            SnakePart::Head(_) => 'H',
            SnakePart::Tail(_) => 'T',
            SnakePart::Body(direction) => match direction {
                BodyPartDirection::Horizontal => '@',
                BodyPartDirection::Vertical => '@',
                BodyPartDirection::TopLeftCorner => '@',
                BodyPartDirection::TopRightCorner => '@',
                BodyPartDirection::BottomLeftCorner => '@',
                BodyPartDirection::BottomRightCorner => '@',
            },
            // BodyPartDirection::Horizontal => '━',
            // BodyPartDirection::Vertical => '┃',
            // BodyPartDirection::TopLeftCorner => '┏',
            // BodyPartDirection::TopRightCorner => '┓',
            // BodyPartDirection::BottomLeftCorner => '┗',
            // BodyPartDirection::BottomRightCorner => '┛',
        },
    }
}
