use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::Print,
    terminal, ExecutableCommand, QueueableCommand, Result,
};
use rand::prelude::*;
use std::{
    fmt::Write,
    io::{stdin, stdout, Error, Write as IOWrite},
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
    SnakePart(Snake),
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum FoodType {
    Blob,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Snake {
    Head(Direction), // bool = true => vertical
    Body(BodyPartDirection),
    Tail(Direction),
}

const BOARD_WIDTH: u16 = 50;
const BOARD_HEIGHT: u16 = 20;

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
    x: u16,
    y: u16,
    snake_tile_type: Snake,
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
    // let reader = stdin();
    // let mut rng = rand::thread_rng();
    // let mut current_direction = Direction::Right;
    let mut snake: Vec<SnakeTile> = Vec::with_capacity(16);
    // snake starts at (0,0) with length 3, looking to the right
    snake.push(SnakeTile {
        x: 10,
        y: 10,
        snake_tile_type: Snake::Head(Direction::Left),
    });
    snake.push(SnakeTile {
        x: 11,
        y: 10,
        snake_tile_type: Snake::Body(BodyPartDirection::Horizontal),
    });
    snake.push(SnakeTile {
        x: 12,
        y: 10,
        snake_tile_type: Snake::Tail(Direction::Left),
    });

    // strategy: either make a board of terminal dimensions or let a user specify it, or constant size

    // for now - constant size
    // one line of blocks around the board

    // let the user specify whether the snake dies at side or is transported to another side
    let mut board: Vec<Vec<Tile>> =
        vec![vec![Tile::Empty; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize];

    // snake starts at (0,0) with length 3, looking to the right
    // board[0][0] = Tile::SnakeTail(false);
    // board[0][1] = Tile::SnakeBody(BodyPartDirection::Horizontal);
    // board[0][2] = Tile::SnakeHead(false);

    terminal::enable_raw_mode()?;
    // let key_pressed = read_char()?;
    // println!("aaaaaaaaaaa");
    // queue!(stdout(), cursor::Hide)?;
    out.queue(terminal::EnterAlternateScreen)?;
    out.queue(cursor::MoveTo(0, 0))?;
    out.queue(Print("printed\r\n"))?;
    out.flush()?;
    draw(&board, &mut out);
    // std::thread::sleep(Duration::new(2, 0));

    print!("Press any key to exit...\r\n");
    let key_pressed = read_char()?;

    // std::thread::sleep(Duration::new(10, 0));
    // queue!(stdout(), cursor::Show)?;
    terminal::disable_raw_mode()?;
    out.execute(terminal::LeaveAlternateScreen)
        .map(|_| Ok(()))?
}

fn read_char() -> Result<char> {
    /*let Event::Key(KeyEvent {
        code: KeyCode::Char(c),
        ..
    }) = event::read()?;
    println!("Key pressed: {c}");
    Ok(c)*/

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

// adds one food particle at random location
// food is only added to empty tile
fn add_food(board: &mut Vec<Vec<Tile>>, rng: &mut ThreadRng) {
    let height = board.len() as u16;
    let width = board[0].len() as u16;
    let (mut x, mut y): (u16, u16) = rng.gen();

    // make sure coordinates are on the board
    // x is width, y is height
    // (0, 0) is the top left corner

    if board[y as usize][x as usize] == Tile::Empty {
        board[y as usize][x as usize] = Tile::Food(FoodType::Blob);
    } else {
        // this will loop endlessly if there are no free tiles
        while board[y as usize][x as usize] != Tile::Empty {
            x = rng.gen();
            y = rng.gen();
        }

        board[y as usize][x as usize] = Tile::Food(FoodType::Blob);
    }
}

fn draw(board: &Vec<Vec<Tile>>, out: &mut impl IOWrite) -> Result<()> {
    // let height = board.len();
    let width = board[0].len();

    // print the top line of the board
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

    out.queue(Print(format!("{top}\r\n")))?;
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
            Snake::Head(direction) => 'H',
            Snake::Tail(direction) => 'T',
            Snake::Body(direction) => match direction {
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
