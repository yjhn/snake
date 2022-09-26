use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    style::Print,
    terminal::{self, ClearType},
    QueueableCommand, Result,
};
use rand::{prelude::SmallRng, Rng, SeedableRng};
use std::{
    fmt::Write,
    io::{stdout, Stdout, Write as IOWrite},
    ops::SubAssign,
    time::Duration,
};
use std::{ops::AddAssign, time::SystemTime};

const BOARD_WIDTH: usize = 50;
const BOARD_HEIGHT: usize = 20;
const STEP_LENGTH: u64 = 300;
const GAME_STEP_LENGTH: Duration = Duration::from_millis(STEP_LENGTH);
const MAX_FOOD_ON_BOARD: usize = 20;

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
    Food(FoodType),
    //Obstacle,
    SnakePart(SnakePart, bool),
}

impl Tile {
    fn is_empty(&self) -> bool {
        *self == Tile::Empty
    }

    fn has_food(&self) -> bool {
        matches!(*self, Tile::Food(_))
    }
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

#[derive(Debug, PartialEq, Clone, Copy)]
struct SnakeTile {
    x: Wrap,
    y: Wrap,
    snake_tile_type: SnakePart,
    eating: bool,
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Wrap {
    modulus: usize,
    number: usize,
}

impl Wrap {
    fn new(number: usize, modulus: usize) -> Self {
        Self {
            modulus,
            number: number % modulus,
        }
    }

    // increment the number, wrap if needed
    fn inc(&mut self) {
        self.number = if self.number == self.modulus - 1 {
            0
        } else {
            self.number + 1
        }
    }

    // decrement the number, wrap if needed
    fn dec(&mut self) {
        self.number = if self.number == 0 {
            self.modulus - 1
        } else {
            self.number - 1
        }
    }
}

impl AddAssign<usize> for Wrap {
    fn add_assign(&mut self, rhs: usize) {
        self.number = if self.number + rhs == self.modulus {
            0
        } else {
            (self.number + rhs) % self.modulus
        }
    }
}

impl SubAssign<usize> for Wrap {
    fn sub_assign(&mut self, rhs: usize) {
        self.number = if self.number as isize - rhs as isize == -1 {
            self.modulus - 1
        } else {
            self.number - rhs
        }
    }
}

impl From<Wrap> for usize {
    fn from(w: Wrap) -> Self {
        w.number
    }
}

fn main() -> Result<()> {
    let width = BOARD_WIDTH;
    let height = BOARD_HEIGHT;

    let mut game: SnakeGame<SmallRng, Stdout> = SnakeGame::new(width, height, stdout());
    game.set_up_screen()?;
    game.play()?;
    game.tear_down_screen()?;

    Ok(())
}

// TODO: move game structs and logic to module to make internals private
#[derive(Debug)]
struct Snake {
    body: Vec<SnakeTile>,
}

impl Snake {
    pub fn new() -> Self {
        Snake {
            body: Vec::with_capacity(100),
        }
    }

    pub fn sample_snake(board_width: usize, board_height: usize) -> Self {
        Snake {
            body: vec![
                SnakeTile {
                    x: Wrap::new(10, board_width),
                    y: Wrap::new(10, board_height),
                    snake_tile_type: SnakePart::Head(Direction::Left),
                    eating: false,
                },
                SnakeTile {
                    x: Wrap::new(11, board_width),
                    y: Wrap::new(10, board_height),
                    snake_tile_type: SnakePart::Body(BodyPartDirection::TopRightCornerLeft),
                    eating: false,
                },
                SnakeTile {
                    x: Wrap::new(11, board_width),
                    y: Wrap::new(11, board_height),
                    snake_tile_type: SnakePart::Body(BodyPartDirection::Up),
                    eating: false,
                },
                SnakeTile {
                    x: Wrap::new(11, board_width),
                    y: Wrap::new(12, board_height),
                    snake_tile_type: SnakePart::Body(BodyPartDirection::Up),
                    eating: false,
                },
                SnakeTile {
                    x: Wrap::new(11, board_width),
                    y: Wrap::new(13, board_height),
                    snake_tile_type: SnakePart::Body(BodyPartDirection::BottomLeftCornerUp),
                    eating: false,
                },
                SnakeTile {
                    x: Wrap::new(12, board_width),
                    y: Wrap::new(13, board_height),
                    snake_tile_type: SnakePart::Tail(Direction::Left),
                    eating: false,
                },
            ],
        }
    }

    pub fn len(&self) -> usize {
        self.body.len()
    }

    pub fn head(&self) -> &SnakeTile {
        &self.body[0]
    }

    pub fn last(&self) -> &SnakeTile {
        self.body.last().unwrap()
    }

    pub fn head_mut(&mut self) -> &mut SnakeTile {
        &mut self.body[0]
    }

    pub fn whole_snake(&self) -> &Vec<SnakeTile> {
        &self.body
    }
}

struct SnakeGame<R: SeedableRng + Rng, W: IOWrite> {
    out: W,
    board: Board,
    board_width: usize,
    board_height: usize,
    snake: Snake,
    score: u32,
    rng: R,
}

impl<R: SeedableRng + Rng, W: IOWrite> SnakeGame<R, W> {
    pub fn new(board_width: usize, board_height: usize, out: W) -> Self {
        SnakeGame {
            out,
            board: vec![vec![Tile::Empty; board_width]; board_height],
            board_width,
            board_height,
            snake: Snake::sample_snake(board_width, board_height),
            score: 0,
            rng: R::from_entropy(),
        }
    }

    pub fn set_up_screen(&mut self) -> Result<()> {
        self.out
            .queue(cursor::Hide)?
            .queue(terminal::EnterAlternateScreen)?
            .queue(cursor::MoveTo(0, 0))?
            .flush()?;
        terminal::enable_raw_mode()?;

        Ok(())
    }

    pub fn tear_down_screen(&mut self) -> Result<()> {
        terminal::disable_raw_mode()?;
        self.out
            .queue(cursor::Show)?
            .queue(terminal::LeaveAlternateScreen)?
            .flush()?;

        Ok(())
    }

    pub fn play(&mut self) -> Result<()> {
        let mut timer;
        let mut step_time = Duration::ZERO;

        loop {
            timer = SystemTime::now();
            for _ in 0..5 {
                self.add_food();
            }
            if self.snake.head().eating {
                self.score += 1;
            }
            self.add_snake_to_board();
            self.draw(&format!(
                "step time: {} us\n\r\
            snake length: {}\n\r\
            score: {}",
                step_time.as_micros(),
                self.snake.len(),
                self.score
            ))?;
            self.remove_snake_from_board();
            let head_direction = match self.snake.head_mut().snake_tile_type {
                SnakePart::Head(ref mut direction) => direction,
                _ => unreachable!(),
            };
            step_time = timer.elapsed().unwrap();
            std::thread::sleep(GAME_STEP_LENGTH);

            if event::poll(Duration::ZERO /*from_millis(10)*/)? {
                match event::read()? {
                    Event::Key(KeyEvent { code: key, .. }) => match key {
                        KeyCode::Char(c) => {
                            // \r - return to line start
                            // \n - start a new line
                            print!("\n\rinput: {c}\n\r");
                            match c {
                                'q' => break,
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
                    _ => unreachable!(),
                }
            }

            // self.snake = self.move_snake();
            self.move_snake_experimental();
        }

        Ok(())
    }

    fn add_snake_to_board(&mut self) {
        for tile in self.snake.whole_snake() {
            self.board[usize::from(tile.y)][usize::from(tile.x)] =
                Tile::SnakePart(tile.snake_tile_type, tile.eating);
        }
    }

    fn remove_snake_from_board(&mut self) {
        for tile in self.snake.whole_snake() {
            self.board[usize::from(tile.y)][usize::from(tile.x)] = Tile::Empty;
        }
    }

    // player controls already applied to the head
    // also if possible this should be simplified
    fn move_snake(&mut self) -> Snake {
        let mut res = Snake::new();

        // process snake head
        let SnakeTile {
            snake_tile_type,
            mut x,
            mut y,
            eating: _,
        } = self.snake.head();
        match snake_tile_type {
            SnakePart::Head(direction) => match direction {
                Direction::Up => y.dec(),
                Direction::Right => x.inc(),
                Direction::Down => y.inc(),
                Direction::Left => x.dec(),
            },
            _ => unreachable!(),
        };
        let eating = self.board[usize::from(y)][usize::from(x)].has_food();
        res.body.push(SnakeTile {
            x,
            y,
            snake_tile_type: *snake_tile_type,
            eating,
        });

        let snake = self.snake.whole_snake();
        for i in 1..=(snake.len() - 2) {
            let previous_tile = snake[i - 1];
            let previous_tile_type = previous_tile.snake_tile_type;
            let SnakeTile {
                mut snake_tile_type,
                mut x,
                mut y,
                eating: _,
            } = snake[i];

            match snake_tile_type {
                SnakePart::Body(ref mut direction) => match direction {
                    BodyPartDirection::Up
                    | BodyPartDirection::BottomLeftCornerUp
                    | BodyPartDirection::BottomRightCornerUp => {
                        y.dec();
                        match previous_tile_type {
                            SnakePart::Head(dir) => match dir {
                                Direction::Up => *direction = BodyPartDirection::Up,
                                Direction::Right => {
                                    *direction = BodyPartDirection::TopLeftCornerRight
                                }
                                Direction::Left => {
                                    *direction = BodyPartDirection::TopRightCornerLeft
                                }
                                Direction::Down => unreachable!(),
                            },
                            SnakePart::Body(dir) => *direction = dir,
                            SnakePart::Tail(_) => unreachable!(),
                        }
                    }
                    BodyPartDirection::Down
                    | BodyPartDirection::TopLeftCornerDown
                    | BodyPartDirection::TopRightCornerDown => {
                        y.inc();
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
                        x.dec();
                        match previous_tile_type {
                            SnakePart::Head(dir) => match dir {
                                Direction::Up => *direction = BodyPartDirection::BottomLeftCornerUp,
                                Direction::Down => {
                                    *direction = BodyPartDirection::TopLeftCornerDown
                                }
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
                        x.inc();
                        match previous_tile_type {
                            SnakePart::Head(dir) => match dir {
                                Direction::Up => {
                                    *direction = BodyPartDirection::BottomRightCornerUp
                                }
                                Direction::Down => {
                                    *direction = BodyPartDirection::TopRightCornerDown
                                }
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

            res.body.push(SnakeTile {
                x,
                y,
                snake_tile_type,
                eating: previous_tile.eating,
            });
        }

        // process snake tail
        let previous_tile = snake[snake.len() - 2];
        // tail
        let SnakeTile {
            snake_tile_type: mut tail_tile_type,
            mut x,
            mut y,
            eating: tail_eating,
        } = snake.last().unwrap();

        if *tail_eating {
            res.body.push(previous_tile);
            res.body.push(SnakeTile {
                x,
                y,
                snake_tile_type: tail_tile_type,
                eating: false,
            });
        } else {
            match tail_tile_type {
                SnakePart::Tail(ref mut direction) => match direction {
                    Direction::Up => {
                        y.dec();
                        match previous_tile.snake_tile_type {
                            SnakePart::Body(dir) => match dir {
                                BodyPartDirection::Up => (),
                                BodyPartDirection::TopLeftCornerRight => {
                                    *direction = Direction::Right
                                }
                                BodyPartDirection::TopRightCornerLeft => {
                                    *direction = Direction::Left
                                }
                                _ => unreachable!(),
                            },
                            _ => unreachable!(),
                        }
                    }
                    Direction::Right => {
                        x.inc();
                        match previous_tile.snake_tile_type {
                            SnakePart::Body(dir) => match dir {
                                BodyPartDirection::Right => (),
                                BodyPartDirection::BottomRightCornerUp => {
                                    *direction = Direction::Up
                                }
                                BodyPartDirection::TopRightCornerDown => {
                                    *direction = Direction::Down
                                }
                                _ => unreachable!(),
                            },
                            _ => unreachable!(),
                        }
                    }
                    Direction::Down => {
                        y.inc();
                        match previous_tile.snake_tile_type {
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
                        x.dec();
                        match previous_tile.snake_tile_type {
                            SnakePart::Body(dir) => match dir {
                                BodyPartDirection::Left => (),
                                BodyPartDirection::BottomLeftCornerUp => *direction = Direction::Up,
                                BodyPartDirection::TopLeftCornerDown => {
                                    *direction = Direction::Down
                                }
                                _ => unreachable!(),
                            },
                            _ => unreachable!(),
                        }
                    }
                },
                _ => unreachable!(),
            }

            res.body.push(SnakeTile {
                x,
                y,
                snake_tile_type: tail_tile_type,
                eating: previous_tile.eating,
            });
        }

        res
    }

    fn move_snake_experimental(&mut self) {
        // move head
        let SnakeTile {
            mut x,
            mut y,
            snake_tile_type,
            eating: mut head_eating,
        } = self.snake.body[0];

        let old_head_x = x;
        let old_head_y = y;
        let head_direction = match snake_tile_type {
            SnakePart::Head(direction) => match direction {
                Direction::Up => {
                    y.dec();
                    Direction::Up
                }
                Direction::Right => {
                    x.inc();
                    Direction::Right
                }
                Direction::Down => {
                    y.inc();
                    Direction::Down
                }
                Direction::Left => {
                    x.dec();
                    Direction::Left
                }
            },
            _ => unreachable!(),
        };
        let old_head_eating = head_eating;
        head_eating = self.board[usize::from(y)][usize::from(x)].has_food();

        // push all snake tiles forward in snake vec
        self.snake.body.rotate_right(1);

        // copy tail
        let tail = self.snake.body[0];
        if tail.eating {
            // push tail to the end
            self.snake.body.push(SnakeTile {
                eating: false,
                ..tail
            });
        } else {
            // tail replaces the last tile
            let last = self.snake.body.last().unwrap();
            let direction = match last.snake_tile_type {
                SnakePart::Body(direction) => match direction {
                    BodyPartDirection::BottomLeftCornerRight
                    | BodyPartDirection::Right
                    | BodyPartDirection::TopLeftCornerRight => Direction::Right,
                    BodyPartDirection::BottomLeftCornerUp
                    | BodyPartDirection::Up
                    | BodyPartDirection::BottomRightCornerUp => Direction::Up,
                    BodyPartDirection::Down
                    | BodyPartDirection::TopLeftCornerDown
                    | BodyPartDirection::TopRightCornerDown => Direction::Down,
                    BodyPartDirection::Left
                    | BodyPartDirection::TopRightCornerLeft
                    | BodyPartDirection::BottomRightCornerLeft => Direction::Left,
                },
                _ => unreachable!(),
            };

            *self.snake.body.last_mut().unwrap() = SnakeTile {
                x: last.x,
                y: last.y,
                snake_tile_type: SnakePart::Tail(direction),
                eating: last.eating,
            };
        }

        // move head to the start
        self.snake.body[0] = SnakeTile {
            x,
            y,
            snake_tile_type,
            eating: head_eating,
        };

        // add tile after the head that connects the head
        // to the body
        let end_tile_type = self.snake.body[2].snake_tile_type;
        let direction = match end_tile_type {
            SnakePart::Body(direction) => match direction {
                BodyPartDirection::BottomLeftCornerRight
                | BodyPartDirection::Right
                | BodyPartDirection::TopLeftCornerRight => match head_direction {
                    Direction::Up => BodyPartDirection::BottomRightCornerUp,
                    Direction::Down => BodyPartDirection::TopRightCornerDown,
                    Direction::Right => BodyPartDirection::Right,
                    Direction::Left => unreachable!(),
                },
                BodyPartDirection::BottomLeftCornerUp
                | BodyPartDirection::Up
                | BodyPartDirection::BottomRightCornerUp => match head_direction {
                    Direction::Up => BodyPartDirection::Up,
                    Direction::Left => BodyPartDirection::TopRightCornerLeft,
                    Direction::Right => BodyPartDirection::TopLeftCornerRight,
                    Direction::Down => unreachable!(),
                },
                BodyPartDirection::Down
                | BodyPartDirection::TopLeftCornerDown
                | BodyPartDirection::TopRightCornerDown => match head_direction {
                    Direction::Left => BodyPartDirection::BottomRightCornerLeft,
                    Direction::Right => BodyPartDirection::BottomLeftCornerRight,
                    Direction::Down => BodyPartDirection::Down,
                    Direction::Up => unreachable!(),
                },
                BodyPartDirection::Left
                | BodyPartDirection::TopRightCornerLeft
                | BodyPartDirection::BottomRightCornerLeft => match head_direction {
                    Direction::Up => BodyPartDirection::BottomLeftCornerUp,
                    Direction::Down => BodyPartDirection::TopLeftCornerDown,
                    Direction::Left => BodyPartDirection::Left,
                    Direction::Right => unreachable!(),
                },
            },
            _ => unreachable!(),
        };

        self.snake.body[1] = SnakeTile {
            x: old_head_x,
            y: old_head_y,
            snake_tile_type: SnakePart::Body(direction),
            eating: old_head_eating,
        };
    }

    // adds one food particle at random location
    // food is only added to empty tile
    fn add_food(&mut self) {
        if self.count_food_on_board() >= MAX_FOOD_ON_BOARD || self.is_board_full() {
            return;
        }

        let height = self.board_height;
        let width = self.board_width;
        let (a, b): (usize, usize) = self.rng.gen();
        let (mut x, mut y) = (a % width, b % height);

        if self.board[y][x] == Tile::Empty {
            self.board[y][x] = Tile::Food(FoodType::Blob);
        } else {
            while self.board[y][x] != Tile::Empty {
                x = self.rng.gen::<usize>() % width;
                y = self.rng.gen::<usize>() % height;
            }

            self.board[y][x] = Tile::Food(FoodType::Blob);
        }
    }

    fn draw(&mut self, additional_text: &str) -> Result<()> {
        let width = self.board_width;

        // top line of the board
        let top = "╔".to_owned() + &"═".repeat(width) + "╗";
        let bottom = "╚".to_owned() + &"═".repeat(width) + "╝";
        self.out
            .queue(terminal::Clear(ClearType::All))?
            .queue(cursor::MoveTo(0, 0))?
            .queue(Print(format!("{top}\n\r")))?;

        for row in &self.board {
            self.out.queue(Print(format!(
                "║{}║\n\r",
                row.iter()
                    .fold(String::with_capacity(width), |mut line, tile| {
                        write!(&mut line, "{}", get_char(tile)).unwrap();
                        line
                    })
            )))?;
        }

        // bottom line
        self.out
            .queue(Print(format!(
                "{bottom}\n\r\
         Control the snake with W,A,S,D or arrow keys\n\r\
         {additional_text}\n\r\
         Press q to exit..."
            )))?
            .flush()?;

        Ok(())
    }

    fn count_food_on_board(&self) -> usize {
        self.board
            .iter()
            .flatten()
            .filter(|tile| tile.has_food())
            .count()
    }

    fn is_board_full(&self) -> bool {
        !self.board.iter().flatten().any(|tile| tile.is_empty())
    }
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
