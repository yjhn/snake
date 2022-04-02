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
type SnakeType = Vec<SnakeTile>;

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
}

// only supports += 1
impl AddAssign<usize> for Wrap {
    fn add_assign(&mut self, rhs: usize) {
        self.number = if self.number + rhs == self.modulus {
            0
        } else {
            self.number + rhs
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
    game.set_up_screen();
    game.play();
    game.tear_down_screen();

    /*let mut snake: SnakeType = vec![
        SnakeTile {
            x: Wrap::new(10, width),
            y: Wrap::new(10, height),
            snake_tile_type: SnakePart::Head(Direction::Left),
            eating: false,
        },
        SnakeTile {
            x: Wrap::new(11, width),
            y: Wrap::new(10, height),
            snake_tile_type: SnakePart::Body(BodyPartDirection::TopRightCornerLeft),
            eating: false,
        },
        SnakeTile {
            x: Wrap::new(11, width),
            y: Wrap::new(11, height),
            snake_tile_type: SnakePart::Body(BodyPartDirection::Up),
            eating: false,
        },
        SnakeTile {
            x: Wrap::new(11, width),
            y: Wrap::new(12, height),
            snake_tile_type: SnakePart::Body(BodyPartDirection::Up),
            eating: false,
        },
        SnakeTile {
            x: Wrap::new(11, width),
            y: Wrap::new(13, height),
            snake_tile_type: SnakePart::Body(BodyPartDirection::BottomLeftCornerUp),
            eating: false,
        },
        SnakeTile {
            x: Wrap::new(12, width),
            y: Wrap::new(13, height),
            snake_tile_type: SnakePart::Tail(Direction::Left),
            eating: false,
        },
    ];
    snake[0].x.modulus = 30000;*/
    /*
        let board: Board = vec![vec![Tile::Empty; width]; height];

        out.queue(cursor::Hide)?
            .queue(terminal::EnterAlternateScreen)?
            .queue(cursor::MoveTo(0, 0))?
            .flush()?;
        terminal::enable_raw_mode()?;

        game_loop(snake, board, &mut out)?;

        terminal::disable_raw_mode()?;
        out.queue(cursor::Show)?
            .queue(terminal::LeaveAlternateScreen)?
            .flush()?;
    */
    Ok(())
}

// TODO: move game structs and logic to module to make internals private
struct Snake {
    head: SnakeTile,
    body: Vec<SnakeTile>,
    tail: SnakeTile,
}

impl Snake {
    pub fn new(board_width: usize, board_height: usize) -> Self {
        Snake {
            head: SnakeTile {
                x: Wrap::new(10, board_width),
                y: Wrap::new(10, board_height),
                snake_tile_type: SnakePart::Head(Direction::Left),
                eating: false,
            },
            body: vec![
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
            ],
            tail: SnakeTile {
                x: Wrap::new(12, board_width),
                y: Wrap::new(13, board_height),
                snake_tile_type: SnakePart::Tail(Direction::Left),
                eating: false,
            },
        }
    }

    pub fn len(&self) -> usize {
        self.body.len() + 2
    }

    pub fn head(&self) -> &SnakeTile {
        &self.head
    }

    pub fn head_mut(&mut self) -> &mut SnakeTile {
        &mut self.head
    }

    pub fn body(&self) -> &Vec<SnakeTile> {
        &self.body
    }

    pub fn tail(&self) -> &SnakeTile {
        &self.tail
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
            out: out,
            board: vec![vec![Tile::Empty; board_width]; board_height],
            board_width,
            board_height,
            snake: Snake::new(board_width, board_height),
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
                    Event::Mouse(_) => unreachable!("disabled in crossterm by default"),
                }
            }

            self.snake = self.move_snake();
        }

        Ok(())
    }

    fn add_snake_to_board(&mut self) {
        // head
        let head = self.snake.head();
        self.board[usize::from(head.y)][usize::from(head.x)] =
            Tile::SnakePart(head.snake_tile_type, head.eating);
        // tail
        let tail = self.snake.tail();
        self.board[usize::from(tail.y)][usize::from(tail.x)] =
            Tile::SnakePart(tail.snake_tile_type, tail.eating);
        // body
        for tile in self.snake.body() {
            self.board[usize::from(tile.y)][usize::from(tile.x)] =
                Tile::SnakePart(tile.snake_tile_type, tile.eating);
        }
    }

    fn remove_snake_from_board(&mut self) {
        // head
        let head = self.snake.head();
        self.board[usize::from(head.y)][usize::from(head.x)] = Tile::Empty;
        // tail
        let tail = self.snake.tail();
        self.board[usize::from(tail.y)][usize::from(tail.x)] = Tile::Empty;
        // body
        for tile in self.snake.body() {
            self.board[usize::from(tile.y)][usize::from(tile.x)] = Tile::Empty;
        }
    }

    // player controls already applied to the head
    // snake wraps around board edges
    // TODO: maybe move this to Snake impl?
    // also if possible this should be simplified
    fn move_snake(&mut self) -> Snake {
        let mut res = Snake::new(self.board_width, self.board_height);

        // process snake head
        let SnakeTile {
            snake_tile_type,
            mut x,
            mut y,
            eating: _eating,
        } = self.snake.head();
        match snake_tile_type {
            SnakePart::Head(direction) => match direction {
                Direction::Up => y -= 1,
                Direction::Right => x += 1,
                Direction::Down => y += 1,
                Direction::Left => x -= 1,
            },
            _ => unreachable!(),
        };
        let eating = if self.board[usize::from(y)][usize::from(x)] == Tile::Food(FoodType::Blob) {
            true
        } else {
            false
        };
        res.head = SnakeTile {
            x,
            y,
            snake_tile_type: *snake_tile_type,
            eating,
        };
        // TODO: adapt this
        // maybe just shift forward the whole snake body and then deal with head and tail?
        for i in 1..(self.snake.len() - 1) {
            let previous_tile = self.snake.body()[i - 1];
            let previous_tile_type = previous_tile.snake_tile_type;
            let previous_tile_eating = previous_tile.eating;
            let tile = self.snake.body()[i];
            let SnakeTile {
                mut snake_tile_type,
                mut x,
                mut y,
                eating: _eating,
            } = tile;

            match snake_tile_type {
                SnakePart::Body(ref mut direction) => match direction {
                    BodyPartDirection::Up
                    | BodyPartDirection::BottomLeftCornerUp
                    | BodyPartDirection::BottomRightCornerUp => {
                        y -= 1;
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
                        x += 1;
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
                eating: previous_tile_eating,
            });
        }

        let previous_tile = self.snake.body.last().unwrap();
        let SnakeTile {
            mut snake_tile_type,
            mut x,
            mut y,
            eating: tail_eating,
        } = self.snake.tail;
        if tail_eating {
            res.body.push(*previous_tile);
            res.tail = SnakeTile {
                x,
                y,
                snake_tile_type,
                eating: false,
            };
        } else {
            match snake_tile_type {
                SnakePart::Tail(ref mut direction) => match direction {
                    Direction::Up => {
                        y -= 1;
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
                        x += 1;
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
                        y += 1;
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
                        x -= 1;
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

            res.tail = SnakeTile {
                x,
                y,
                snake_tile_type,
                eating: previous_tile.eating,
            };
        }

        res
    } // TODO: maket his more elegant

    // adds one food particle at random location
    // food is only added to empty tile
    fn add_food(&mut self) {
        if self.count_food_on_board() >= MAX_FOOD_ON_BOARD {
            return;
        }

        let height = self.board.len();
        let width = self.board[0].len();
        let (a, b): (usize, usize) = self.rng.gen();
        let (mut x, mut y) = (a % width, b % height);

        // make sure coordinates are on the board
        // x is width, y is height
        // (0, 0) is the top left corner

        if self.board[y][x] == Tile::Empty {
            self.board[y][x] = Tile::Food(FoodType::Blob);
        } else {
            // this will loop endlessly if there are no free tiles
            while self.board[y][x] != Tile::Empty {
                x = self.rng.gen::<usize>() % width;
                y = self.rng.gen::<usize>() % height;
            }

            self.board[y][x] = Tile::Food(FoodType::Blob);
        }
    }

    fn draw(&mut self, additional_text: &str) -> Result<()> {
        // let height = board.len();
        let width = self.board[0].len();

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

    fn count_food_on_board(&self) -> u32 {
        self.board.iter().flatten().fold(0, |count, tile| {
            if *tile == Tile::Food(FoodType::Blob) {
                count + 1
            } else {
                count
            }
        })
    }
}
/*
fn game_loop(mut snake: SnakeType, mut board: Board, out: &mut impl IOWrite) -> Result<()> {
    // let mut out = stdout();
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
            out,
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
        std::thread::sleep(GAME_STEP_LENGTH);

        if event::poll(Duration::ZERO /*from_millis(10)*/)? {
            match event::read()? {
                Event::Key(KeyEvent { code: key, .. }) => match key {
                    KeyCode::Char(c) => {
                        // \r - return to line start
                        // \n - start a new line
                        print!("\n\rinput: {c}\n\r");
                        match c {
                            'q' => return Ok(()),
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

        snake = move_snake(&board, snake);
    }
}

fn add_snake_to_board(board: &mut Board, snake: &SnakeType) {
    for tile in snake {
        board[usize::from(tile.y)][usize::from(tile.x)] =
            Tile::SnakePart(tile.snake_tile_type, tile.eating);
    }
}

fn remove_snake_from_board(board: &mut Board, snake: &SnakeType) {
    for tile in snake {
        board[usize::from(tile.y)][usize::from(tile.x)] = Tile::Empty;
    }
}

// player controls already applied to the head
// snake wraps around board edges
fn move_snake(board: &Board, snake: SnakeType) -> SnakeType {
    let mut res = Vec::<SnakeTile>::with_capacity(snake.len());

    // process snake head
    let head = snake[0];
    let SnakeTile {
        snake_tile_type,
        mut x,
        mut y,
        eating: _eating,
    } = head;
    match snake_tile_type {
        SnakePart::Head(direction) => match direction {
            Direction::Up => y -= 1,
            Direction::Right => x += 1,
            Direction::Down => y += 1,
            Direction::Left => x -= 1,
        },
        _ => unreachable!(),
    };
    let eating = if board[usize::from(y)][usize::from(x)] == Tile::Food(FoodType::Blob) {
        true
    } else {
        false
    };
    res.push(SnakeTile {
        x,
        y,
        snake_tile_type,
        eating,
    });

    for i in 1..(snake.len() - 1) {
        let previous_tile = snake[i - 1];
        let previous_tile_type = previous_tile.snake_tile_type;
        let previous_tile_eating = previous_tile.eating;
        let tile = snake[i];
        let SnakeTile {
            mut snake_tile_type,
            mut x,
            mut y,
            eating: _eating,
        } = tile;

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

        res.push(SnakeTile {
            x,
            y,
            snake_tile_type,
            eating: previous_tile_eating,
        });
    }

    let previous_tile = snake[snake.len() - 2];
    let tile = snake[snake.len() - 1];
    let SnakeTile {
        mut snake_tile_type,
        mut x,
        mut y,
        eating: _eating,
    } = tile;
    if snake[snake.len() - 1].eating {
        res.push(previous_tile);
        res.push(SnakeTile {
            x,
            y,
            snake_tile_type,
            eating: false,
        });
    } else {
        match snake_tile_type {
            SnakePart::Tail(ref mut direction) => match direction {
                Direction::Up => {
                    y -= 1;
                    match previous_tile.snake_tile_type {
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
                    match previous_tile.snake_tile_type {
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
                    x -= 1;
                    match previous_tile.snake_tile_type {
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

        res.push(SnakeTile {
            x,
            y,
            snake_tile_type,
            eating: previous_tile.eating,
        });
    }

    res
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
    // let height = board.len();
    let width = board[0].len();

    // top line of the board
    let top = "╔".to_owned() + &"═".repeat(width) + "╗";
    let bottom = "╚".to_owned() + &"═".repeat(width) + "╝";
    out.queue(terminal::Clear(ClearType::All))?
        .queue(cursor::MoveTo(0, 0))?
        .queue(Print(format!("{top}\n\r")))?;

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
    )))?
    .flush()?;

    Ok(())
}
*/
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
