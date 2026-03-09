#[cfg(feature = "buddy-alloc")]
mod alloc;
mod wasm4;
mod utils;
mod assets;
mod dragon;
mod stage;
mod scenes;
mod game;
use game::*;
use lazy_static::lazy_static;
use std::sync::Mutex;


lazy_static! {
  static ref GAME: Mutex<Game> = Mutex::new(Game::new());
}


#[no_mangle]
fn start() {
  GAME.lock().expect("").start()
}

#[no_mangle]
fn update() {
  GAME.lock().expect("").update();

}
