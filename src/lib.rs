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

/*
  PALETTE [u32;4](FFFFFF, FFFFFF, FFFFFF) -> 使用する表示色(4色)のリスト。変更可能。
  DRAW_COLORS u16(FFFF) -> 表示色の中から使用する色を選択(idx: 0~4, 0は透明)
    下位0~3ビットは最初の描画色、4~7ビットは二番目の描画色を保持する。
    例: *DRAW_COLORS = 0x42(0000 0000 0100 0010); 描画色は0010(2)、アウトライン色は0100(4)で描画される。
    例: *DRAW_COLORS = 3(0000 0000 0000 0011); 描画色は0011(3)、アウトライン色は0000(0:透明)で描画される。
    例: *DRAW_COLORS = 0x0321(0000 0011 0010 0001); 
*/

lazy_static! {
  static ref GAME: Mutex<Game> = Mutex::new(Game::new());
}


#[no_mangle]
fn start() {
  GAME.lock().expect("game_state").start()
}

#[no_mangle]
fn update() {
  GAME.lock().expect("game_state").update();

}
