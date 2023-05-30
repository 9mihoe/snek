// https://users.rust-lang.org/t/usage-of-extern-crate/73619

use bracket_lib::prelude::*;
use std::collections::VecDeque;

const SCREEN_WIDTH : i32 = 48;
const SCREEN_HEIGHT : i32 = 48;

enum Dir {
  Static, // Only at the beginning.
  Left,
  Right,
  Up,
  Down
}

#[derive(PartialEq, Eq, Clone, Copy)]
struct Cell {
  pub x: i32,
  pub y: i32
}

struct Player {
  pub head: Cell,
  pub tail: VecDeque<Cell>,
  pub dir: Dir
}

impl Cell {
  fn new(x: i32, y: i32) -> Self {
    Cell{x: x, y:y}
  }

  fn render(&mut self, ctx: &mut BTerm) {
    let x_pixel = 2*self.x;
    let y_pixel = 2*self.y;
    ctx.set(x_pixel, y_pixel, YELLOW, BLACK, to_cp437('@'));
    ctx.set(x_pixel+1, y_pixel, YELLOW, BLACK, to_cp437('@'));
    ctx.set(x_pixel, y_pixel+1, YELLOW, BLACK, to_cp437('@'));
    ctx.set(x_pixel+1, y_pixel+1, YELLOW, BLACK, to_cp437('@'));
  }

  fn right(curr: Cell) -> Cell {
    Cell::new(curr.x-1, curr.y)
  }

  fn left(curr: Cell) -> Cell {
    Cell::new(curr.x+1, curr.y)
  }

  fn up(curr: Cell) -> Cell {
    Cell::new(curr.x, curr.y-1)
  }

  fn down(curr: Cell) -> Cell {
    Cell::new(curr.x, curr.y+1)
  }
}

impl Player {
  fn new(x: i32, y: i32) -> Self {
      Player {
        head: Cell::new(x, y),
        tail: VecDeque::new(), 
        dir: Dir::Static
      }
  }

  fn render_tail(&mut self, ctx: &mut BTerm) {
    for i in self.tail.iter_mut() {
      i.render(ctx);
    }
  }

  fn render(&mut self, ctx: &mut BTerm) {
    // Always print the head of snek.
    self.head.render(ctx);
    self.render_tail(ctx);
    ctx.set_active_console(0);
  }

  fn update_direction(&mut self, ctx: &mut BTerm) {
    if let Some(key) = ctx.key {
      match key {
        VirtualKeyCode::D => self.dir = Dir::Left,
        VirtualKeyCode::A => self.dir = Dir::Right,
        VirtualKeyCode::W => self.dir = Dir::Up,
        VirtualKeyCode::S => self.dir = Dir::Down,
        _ => (),
      };
    }
  }

  fn update_position(&mut self) {
    if self.tail.len() > 1 {
      self.tail.rotate_left(1);
      self.tail.push_front(self.head);
      self.tail.pop_back();
    }
    
    match self.dir {
      Dir::Left => self.head = Cell::left(self.head),
      Dir::Right => self.head = Cell::right(self.head),
      Dir::Up => self.head = Cell::up(self.head),
      Dir::Down => self.head = Cell::down(self.head),
      Dir::Static => ()
    }
  }

  fn is_out_of_bounds(&mut self) -> bool {
    return self.head.x+1 <= 0 
      || self.head.x+1 >= SCREEN_WIDTH 
      || self.head.y+1 <= 0 
      || self.head.y+1 >= SCREEN_HEIGHT;
  }

  fn grow(&mut self) {
    let last_cell = if self.tail.len()> 0 {self.tail[self.tail.len()-1]} else {self.head};
    match self.dir {
      Dir::Left => self.tail.push_back(Cell::left(last_cell)),
      Dir::Right => self.tail.push_back(Cell::right(last_cell)),
      Dir::Up => self.tail.push_back(Cell::up(last_cell)),
      Dir::Down => self.tail.push_back(Cell::down(last_cell)),
      Dir::Static => ()
    }
  }
}

struct Food {
  pub pos: Cell,
  pos_gen: RandomNumberGenerator
}

impl Food {
  fn new() -> Self {
    let mut rng_new = RandomNumberGenerator::new();
    Food {
      pos: Cell::new(rng_new.range(0, 12), rng_new.range(0, 12)),
      pos_gen: rng_new
    }
  }

  fn render(&mut self, ctx: &mut BTerm) {
    ctx.cls();
    self.pos.render(ctx);
    ctx.set_active_console(0);
  }

  fn respawn(&mut self) {
    self.pos = Cell::new(
      self.pos_gen.range(0, 12), 
      self.pos_gen.range(0, 12)
    );
  }
}

enum GameMode {
  Playing,
  Dead
}

struct State {
  mode: GameMode,
  player: Player,
  ticks: u64,
  food: Food,
  score: i32,
}

impl State {
  fn new() -> Self {
      State {
        mode: GameMode::Playing,
        player: Player::new(2, 2),
        ticks: 0,
        food: Food::new(),
        score: 0,
      }
  }

  fn restart(&mut self, ctx: &mut BTerm) {
    ctx.cls();
    self.player = Player::new(20, 20);
    self.ticks = 0;
    self.food = Food::new();
    self.score = 0;
  }

  fn play(&mut self, ctx: &mut BTerm) {
    ctx.cls();
    self.food.render(ctx);
    self.player.update_direction(ctx);
    if self.ticks % 5 == 0 {
      // println!("ticks: {}", self.ticks);
      self.player.update_position();
    }
    self.player.render(ctx);
    if self.player.is_out_of_bounds() {
      self.mode = GameMode::Dead;
    }
    if self.player.head == self.food.pos {
      self.player.grow();
      self.food.respawn();
    }
  }

  fn dead(&mut self, ctx: &mut BTerm) {
    ctx.cls();
    ctx.print_centered(5, "You are dead!");
    ctx.print_centered(8, "(P) Play Again");
    ctx.print_centered(9, "(Q) Quit Game");

    if let Some(key) = ctx.key {
      match key {
          VirtualKeyCode::P => {
            self.mode = GameMode::Playing;
            self.restart(ctx);
          },
          VirtualKeyCode::Q => ctx.quitting = true,
          _ => {}
      }
    }
  }
}

impl GameState for State {
  fn tick(&mut self, ctx: &mut BTerm) {
    match self.mode {
      GameMode::Playing => self.play(ctx),
      GameMode::Dead => self.dead(ctx),
    }
    self.ticks += 1;
  }
}

fn main() -> BError {
  let context = BTermBuilder::simple(SCREEN_WIDTH, SCREEN_HEIGHT)
    .unwrap()
    .with_title("Snek")
    .build()?;
  main_loop(context, State::new())
}