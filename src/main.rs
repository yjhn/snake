use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    style::Print,
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand, Result,
};
use rand::prelude::*;
use std::time::SystemTime;
use std::{
    fmt::Write,
    io::{stdout, Write as IOWrite},
    process,
    time::Duration,
};

const BOARD_WIDTH: usize = 50;
const BOARD_HEIGHT: usize = 20;
const STEP_LENGTH: u64 = 300;
const GAME_STEP_LENGTH: Duration = Duration::from_millis(STEP_LENGTH);
const MAX_FOOD_ON_BOARD: u32 = 20;

#[derive(Debug, PartialEq, Clone, Copy)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum BodyPartDirection {
    Up,
    Down,
    Left,
    Right,
    TopLeftCornerRight,
    TopLeftCornerDown,
    TopRightCornerLeft,
    TopRightCornerDown,
    BottomLeftCornerRight,
    BottomLeftCornerUp,
    BottomRightCornerLeft,
    BottomRightCornerUp,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Tile {
    Empty,
    Food(FoodType), // variable is for the type of food
    //Obstacle,
    SnakePart(SnakePart, bool),
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum FoodType {
    Blob,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum SnakePart {
    Head(Direction),
    Body(BodyPartDirection),
    Tail(Direction),
}

type Board = Vec<Vec<Tile>>;
type Snake = Vec<SnakeTile>;

#[derive(Debug, PartialEq, Clone, Copy)]
struct SnakeTile {
    x: usize,
    y: usize,
    snake_tile_type: SnakePart,
    eating: bool,
}

/*struct Wrap {
    modulus: u16,
    number: u16,
}

impl AddAssign<u8> for Wrap {
    fn add_assign(&mut self, rhs: u8) {}
}*/

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
            eating: false,
        },
        SnakeTile {
            x: 11,
            y: 10,
            snake_tile_type: SnakePart::Body(BodyPartDirection::TopRightCornerLeft),
            eating: false,
        },
        SnakeTile {
            x: 11,
            y: 11,
            snake_tile_type: SnakePart::Body(BodyPartDirection::Up),
            eating: false,
        },
        SnakeTile {
            x: 11,
            y: 12,
            snake_tile_type: SnakePart::Body(BodyPartDirection::Up),
            eating: false,
        },
        SnakeTile {
            x: 11,
            y: 13,
            snake_tile_type: SnakePart::Body(BodyPartDirection::BottomLeftCornerUp),
            eating: false,
        },
        SnakeTile {
            x: 12,
            y: 13,
            snake_tile_type: SnakePart::Tail(Direction::Left),
            eating: false,
        },
    ];

    let board: Board = vec![vec![Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT];

    out.queue(cursor::Hide)?;
    out.queue(terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    out.queue(cursor::MoveTo(0, 0))?;
    out.flush()?;

    game_loop(snake, board)?;

    terminal::disable_raw_mode()?;
    out.queue(cursor::Show)?;
    out.execute(terminal::LeaveAlternateScreen)?;

    Ok(())
}

fn game_loop(mut snake: Snake, mut board: Board) -> Result<()> {
    let mut out = stdout();
    let mut rng = rand::thread_rng();
    let mut timer;
    let mut step_time = Duration::ZERO;
    let mut eaten = 0;

    loop {
        timer = SystemTime::now();
        for _ in 0..5 {
            add_food(&mut board, &mut rng);
        }
        if snake[0].eating {
            eaten += 1;
        }
        add_snake_to_board(&mut board, &snake);
        draw(
            &board,
            &mut out,
            &format!(
                "step time: {} us\n\r\
            snake length: {}\n\r\
            food eaten: {eaten}",
                step_time.as_micros(),
                snake.len()
            ),
        )?;
        remove_snake_from_board(&mut board, &snake);
        let head_direction = match snake[0].snake_tile_type {
            SnakePart::Head(ref mut direction) => direction,
            _ => unreachable!(),
        };
        step_time = timer.elapsed().unwrap();
        // to listen to button presses, use non-blocking IO with poll
        // it will also sleep the program for the right duration
        std::thread::sleep(GAME_STEP_LENGTH);

        process_input(head_direction)?;
        snake = move_snake(&board, snake, board[0].len(), board.len());
    }
}

fn add_snake_to_board(board: &mut Board, snake: &Snake) {
    for tile in snake {
        board[tile.y][tile.x] = Tile::SnakePart(tile.snake_tile_type, tile.eating);
    }
}

fn remove_snake_from_board(board: &mut Board, snake: &Snake) {
    for tile in snake {
        board[tile.y][tile.x] = Tile::Empty;
    }
}

// player controls already applied to the head
// snake wraps around board edges
// TODO: handle eating
fn move_snake(board: &Board, snake: Snake, width: usize, height: usize) -> Snake {
    let mut res = Vec::<SnakeTile>::with_capacity(snake.len());

    // process snake head
    let head = snake[0];
    let SnakeTile {
        snake_tile_type,
        x,
        y,
        eating: _eating,
    } = head;
    let mut sx = x as isize;
    let mut sy = y as isize;
    match snake_tile_type {
        SnakePart::Head(direction) => match direction {
            Direction::Up => sy -= 1,
            Direction::Right => sx += 1,
            Direction::Down => sy += 1,
            Direction::Left => sx -= 1,
        },
        _ => unreachable!(),
    };
    let eating = if board[wrap_y(sy, height)][wrap_x(sx, width)] == Tile::Food(FoodType::Blob) {
        true
    } else {
        false
    };
    res.push(make_snake_tile(
        snake_tile_type,
        sx,
        sy,
        eating,
        width,
        height,
    ));

    for i in 1..(snake.len() - 1) {
        let previous_tile = snake[i - 1];
        let previous_tile_type = previous_tile.snake_tile_type;
        let previous_tile_eating = previous_tile.eating;
        let tile = snake[i];
        let SnakeTile {
            mut snake_tile_type,
            x,
            y,
            eating: _eating,
        } = tile;
        let mut x = x as isize;
        let mut y = y as isize;

        match snake_tile_type {
            SnakePart::Body(ref mut direction) => match direction {
                BodyPartDirection::Up
                | BodyPartDirection::BottomLeftCornerUp
                | BodyPartDirection::BottomRightCornerUp => {
                    y -= 1;
                    match previous_tile_type {
                        SnakePart::Head(dir) => match dir {
                            Direction::Up => *direction = BodyPartDirection::Up,
                            Direction::Right => *direction = BodyPartDirection::TopLeftCornerRight,
                            Direction::Left => *direction = BodyPartDirection::TopRightCornerLeft,
                            Direction::Down => unreachable!(),
                        },
                        SnakePart::Body(dir) => *direction = dir,
                        SnakePart::Tail(_) => unreachable!(),
                    }
                }
                BodyPartDirection::Down
                | BodyPartDirection::TopLeftCornerDown
                | BodyPartDirection::TopRightCornerDown => {
                    y += 1;
                    match previous_tile_type {
                        SnakePart::Head(dir) => match dir {
                            Direction::Down => *direction = BodyPartDirection::Down,
                            Direction::Right => {
                                *direction = BodyPartDirection::BottomLeftCornerRight
                            }
                            Direction::Left => {
                                *direction = BodyPartDirection::BottomRightCornerLeft
                            }
                            Direction::Up => unreachable!(),
                        },
                        SnakePart::Body(dir) => *direction = dir,
                        SnakePart::Tail(_) => unreachable!(),
                    }
                }
                BodyPartDirection::Left
                | BodyPartDirection::TopRightCornerLeft
                | BodyPartDirection::BottomRightCornerLeft => {
                    x -= 1;
                    match previous_tile_type {
                        SnakePart::Head(dir) => match dir {
                            Direction::Up => *direction = BodyPartDirection::BottomLeftCornerUp,
                            Direction::Down => *direction = BodyPartDirection::TopLeftCornerDown,
                            Direction::Left => *direction = BodyPartDirection::Left,
                            Direction::Right => unreachable!(),
                        },
                        SnakePart::Body(dir) => *direction = dir,
                        SnakePart::Tail(_) => unreachable!(),
                    }
                }
                BodyPartDirection::Right
                | BodyPartDirection::TopLeftCornerRight
                | BodyPartDirection::BottomLeftCornerRight => {
                    x += 1;
                    match previous_tile_type {
                        SnakePart::Head(dir) => match dir {
                            Direction::Up => *direction = BodyPartDirection::BottomRightCornerUp,
                            Direction::Down => *direction = BodyPartDirection::TopRightCornerDown,
                            Direction::Right => *direction = BodyPartDirection::Right,
                            Direction::Left => unreachable!(),
                        },
                        SnakePart::Body(dir) => *direction = dir,
                        SnakePart::Tail(_) => unreachable!(),
                    }
                }
            },
            SnakePart::Tail(_) => unreachable!(),
            SnakePart::Head(_) => unreachable!(),
        }

        res.push(make_snake_tile(
            snake_tile_type,
            x,
            y,
            previous_tile_eating,
            width,
            height,
        ));
    }

    let previous_tile = snake[snake.len() - 2];
    let previous_tile_type = previous_tile.snake_tile_type;
    let previous_tile_eating = previous_tile.eating;
    let tile = snake[snake.len() - 1];
    let SnakeTile {
        mut snake_tile_type,
        x,
        y,
        eating: _eating,
    } = tile;
    let mut x = x as isize;
    let mut y = y as isize;
    if snake[snake.len() - 1].eating {
        // TODO: lengthen the snake

        res.push(make_snake_tile(
            previous_tile.snake_tile_type,
            previous_tile.x as isize,
            previous_tile.y as isize,
            previous_tile_eating,
            width,
            height,
        ));
        res.push(make_snake_tile(snake_tile_type, x, y, false, width, height));
    } else {
        match snake_tile_type {
            SnakePart::Tail(ref mut direction) => match direction {
                Direction::Up => {
                    y -= 1;
                    match previous_tile_type {
                        SnakePart::Body(dir) => match dir {
                            BodyPartDirection::Up => (),
                            BodyPartDirection::TopLeftCornerRight => *direction = Direction::Right,
                            BodyPartDirection::TopRightCornerLeft => *direction = Direction::Left,
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    }
                }
                Direction::Right => {
                    x += 1;
                    match previous_tile_type {
                        SnakePart::Body(dir) => match dir {
                            BodyPartDirection::Right => (),
                            BodyPartDirection::BottomRightCornerUp => *direction = Direction::Up,
                            BodyPartDirection::TopRightCornerDown => *direction = Direction::Down,
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    }
                }
                Direction::Down => {
                    y += 1;
                    match previous_tile_type {
                        SnakePart::Body(dir) => match dir {
                            BodyPartDirection::Down => (),
                            BodyPartDirection::BottomLeftCornerRight => {
                                *direction = Direction::Right
                            }
                            BodyPartDirection::BottomRightCornerLeft => {
                                *direction = Direction::Left
                            }
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    }
                }
                Direction::Left => {
                    x -= 1;
                    match previous_tile_type {
                        SnakePart::Body(dir) => match dir {
                            BodyPartDirection::Left => (),
                            BodyPartDirection::BottomLeftCornerUp => *direction = Direction::Up,
                            BodyPartDirection::TopLeftCornerDown => *direction = Direction::Down,
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    }
                }
            },
            _ => unreachable!(),
        }

        res.push(make_snake_tile(
            snake_tile_type,
            x,
            y,
            previous_tile_eating,
            width,
            height,
        ));
    }

    res
}

fn make_snake_tile(
    snake_tile_type: SnakePart,
    x: isize,
    y: isize,
    eating: bool,
    width: usize,
    height: usize,
) -> SnakeTile {
    // wrap snake around edges
    let x = wrap_x(x, width);
    let y = wrap_y(y, height);

    SnakeTile {
        snake_tile_type,
        x,
        y,
        eating,
    }
}

fn wrap_x(x: isize, width: usize) -> usize {
    if x == -1 {
        width - 1
    } else {
        let x = x as usize;
        if x == width {
            0
        } else {
            x
        }
    }
}

fn wrap_y(y: isize, height: usize) -> usize {
    if y == -1 {
        height - 1
    } else {
        let y = y as usize;
        if y == height {
            0
        } else {
            y
        }
    }
}

fn process_input(head_direction: &mut Direction) -> Result<()> {
    if event::poll(Duration::ZERO /*from_millis(10)*/)? {
        match event::read()? {
            Event::Key(KeyEvent { code: key, .. }) => match key {
                KeyCode::Char(c) => {
                    // \r - return to line start
                    // \n - start a new line
                    print!("\n\rinput: {c}\n\r");
                    match c {
                        'q' => process::exit(0),
                        // TODO: process user controls
                        'w' if *head_direction != Direction::Down => {
                            *head_direction = Direction::Up
                        }
                        'a' if *head_direction != Direction::Right => {
                            *head_direction = Direction::Left
                        }
                        's' if *head_direction != Direction::Up => {
                            *head_direction = Direction::Down
                        }
                        'd' if *head_direction != Direction::Left => {
                            *head_direction = Direction::Right
                        }
                        _ => {
                            print!("\n\rIgnored user input.\n\r");
                            std::thread::sleep(Duration::from_secs(1));
                        }
                    }
                }
                KeyCode::Up if *head_direction != Direction::Down => {
                    *head_direction = Direction::Up
                }
                KeyCode::Left if *head_direction != Direction::Right => {
                    *head_direction = Direction::Left
                }
                KeyCode::Down if *head_direction != Direction::Up => {
                    *head_direction = Direction::Down
                }
                KeyCode::Right if *head_direction != Direction::Left => {
                    *head_direction = Direction::Right
                }
                _ => {
                    print!("\n\rIgnored user input.\n\r");
                    std::thread::sleep(Duration::from_secs(1));
                }
            },
            Event::Resize(x, y) => {
                print!("new terminal size: {x}, {y}\n\r");
                std::thread::sleep(Duration::from_secs(1));
            }
            Event::Mouse(_) => unreachable!("disabled in crossterm by default"),
        }
    }

    Ok(())
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

fn draw(board: &Board, out: &mut impl IOWrite, additional_text: &str) -> Result<()> {
    out.queue(terminal::Clear(ClearType::All))?;
    out.queue(cursor::MoveTo(0, 0))?;

    // let height = board.len();
    let width = board[0].len();

    // top line of the board
    let top = "╔".to_owned() + &"═".repeat(width) + "╗";
    let bottom = "╚".to_owned() + &"═".repeat(width) + "╝";
    out.queue(Print(format!("{top}\n\r")))?;

    for row in board {
        out.queue(Print(format!(
            "║{}║\n\r",
            row.iter()
                .fold(String::with_capacity(width), |mut line, tile| {
                    write!(&mut line, "{}", get_char(tile)).unwrap();
                    line
                })
        )))?;
    }

    // bottom line
    out.queue(Print(format!(
        "{bottom}\n\r\
         Control the snake with W,A,S,D or arrow keys\n\r\
         {additional_text}\n\r\
         Press q to exit..."
    )))?;
    out.flush()?;

    Ok(())
}

// snake is drawn using Box Drawing Unicode char block
fn get_char(tile: &Tile) -> char {
    match *tile {
        Tile::Empty => ' ',
        Tile::Food(_) => '*',
        // Tile::Obstacle => '@',
        Tile::SnakePart(snake_part, eating) => match snake_part {
            SnakePart::Head(direction) => match direction {
                Direction::Right => {
                    if eating {
                        'e'
                    } else {
                        '>'
                    }
                }
                Direction::Left => {
                    if eating {
                        'e'
                    } else {
                        '<'
                    }
                }
                Direction::Up => {
                    if eating {
                        'e'
                    } else {
                        '⌃'
                    }
                }
                Direction::Down => {
                    if eating {
                        'e'
                    } else {
                        '⌄'
                    }
                }
            },
            SnakePart::Tail(direction) => match direction {
                Direction::Right => {
                    if eating {
                        'e'
                    } else {
                        '>'
                    }
                }
                Direction::Left => {
                    if eating {
                        'e'
                    } else {
                        '<'
                    }
                }
                Direction::Up => {
                    if eating {
                        'e'
                    } else {
                        '⌃'
                    }
                }
                Direction::Down => {
                    if eating {
                        'e'
                    } else {
                        '⌄'
                    }
                }
            },
            SnakePart::Body(direction) => match direction {
                BodyPartDirection::Up => {
                    if eating {
                        'e'
                    } else {
                        '┃'
                    }
                }
                BodyPartDirection::Down => {
                    if eating {
                        'e'
                    } else {
                        '┃'
                    }
                }
                BodyPartDirection::Left => {
                    if eating {
                        'e'
                    } else {
                        '━'
                    }
                }
                BodyPartDirection::Right => {
                    if eating {
                        'e'
                    } else {
                        '━'
                    }
                }
                BodyPartDirection::TopLeftCornerRight => {
                    if eating {
                        'e'
                    } else {
                        '┏'
                    }
                }
                BodyPartDirection::TopLeftCornerDown => {
                    if eating {
                        'e'
                    } else {
                        '┏'
                    }
                }
                BodyPartDirection::TopRightCornerLeft => {
                    if eating {
                        'e'
                    } else {
                        '┓'
                    }
                }
                BodyPartDirection::TopRightCornerDown => {
                    if eating {
                        'e'
                    } else {
                        '┓'
                    }
                }
                BodyPartDirection::BottomLeftCornerRight => {
                    if eating {
                        'e'
                    } else {
                        '┗'
                    }
                }
                BodyPartDirection::BottomLeftCornerUp => {
                    if eating {
                        'e'
                    } else {
                        '┗'
                    }
                }
                BodyPartDirection::BottomRightCornerLeft => {
                    if eating {
                        'e'
                    } else {
                        '┛'
                    }
                }
                BodyPartDirection::BottomRightCornerUp => {
                    if eating {
                        'e'
                    } else {
                        '┛'
                    }
                }
            },
        },
    }
}
