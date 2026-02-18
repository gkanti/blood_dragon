use crate::wasm4::*;
use lazy_static::lazy_static;
use std::{sync::Mutex};

// -------------------------------
// Button Handler
// -------------------------------
lazy_static! {
  static ref BTN_HANDLER: Mutex<ButtonHandler> = Mutex::new(ButtonHandler::new());
}
pub const BTN_1:     u8 = 0;
pub const BTN_2:     u8 = 1;
pub const BTN_LEFT:  u8 = 4;
pub const BTN_RIGHT: u8 = 5;
pub const BTN_UP:    u8 = 6;
pub const BTN_DOWN:  u8 = 7;

pub struct ButtonHandler {
  btn_states: [u16; 8],
  old_btn_states: u8,
  just_changed_btn_states: u8
}

pub fn update_btn() {
  BTN_HANDLER.lock().expect("btn_state").update();
}
pub fn get_pressed_time(btn: u8) -> u16 {
  BTN_HANDLER.lock().expect("btn_state").btn_states[btn as usize]
}
pub fn is_pressed(btn: u8) -> bool {
  BTN_HANDLER.lock().expect("btn_state").btn_states[btn as usize] >= 1
}
pub fn is_just_pressed(btn: u8) -> bool {
  BTN_HANDLER.lock().expect("btn_state").btn_states[btn as usize] == 1
}
pub fn is_just_released(btn: u8) -> bool {
  let handler = BTN_HANDLER.lock().expect("btn_state");
  // 「ボタンの押された長さが0である」かつ、「直近にボタンの状態が変化した」であれば離された瞬間
  // (右辺について補足: 調べたいボタンのビットのみ立った数値と、変更があったボタンのビットが立った数値をAND演算している。)
  (handler.btn_states[btn as usize] == 0) && (handler.just_changed_btn_states & (1 << btn) != 0)
}

impl ButtonHandler {
  pub fn new() -> Self {
    Self { btn_states: [0,0,0,0,0,0,0,0], old_btn_states: 0, just_changed_btn_states: 0}
  }
  pub fn update(&mut self) {
    let gamepad = unsafe{*GAMEPAD1};
    // 1f前から状態が変化したボタンを取得(現在のボタン状態とでXORをとる)
    self.just_changed_btn_states = gamepad ^ self.old_btn_states;
    // 各ボタン状態の更新
    if gamepad & BUTTON_1     != 0 { self.btn_states[0] += 1 } else { self.btn_states[0] = 0 }
    if gamepad & BUTTON_2     != 0 { self.btn_states[1] += 1 } else { self.btn_states[1] = 0 }
    if gamepad & BUTTON_LEFT  != 0 { self.btn_states[4] += 1 } else { self.btn_states[4] = 0 }
    if gamepad & BUTTON_RIGHT != 0 { self.btn_states[5] += 1 } else { self.btn_states[5] = 0 }
    if gamepad & BUTTON_UP    != 0 { self.btn_states[6] += 1 } else { self.btn_states[6] = 0 }
    if gamepad & BUTTON_DOWN  != 0 { self.btn_states[7] += 1 } else { self.btn_states[7] = 0 }

    self.old_btn_states = gamepad;
  }
}


// -------------------------------
// Image
// -------------------------------

pub struct RawImage {
  pub width: u32,
  pub height: u32,
  pub flags: u32,
  pub data: &'static [u8]
}
pub struct Image {
  col_idx: u16,
  raw: &'static RawImage,
  pub xflip: bool,
  pub yflip: bool,
  pub rot: bool,
}
impl Image {
  pub const fn new(col_idx: u16, raw: &'static RawImage) -> Self {
    Self {col_idx, raw, xflip: false, yflip: false, rot: false}
  }
  pub const fn newf(col_idx: u16, raw: &'static RawImage, xflip: bool, yflip: bool, rot: bool) -> Self {
    Self {col_idx, raw, xflip, yflip, rot}
  }
  // 画像そのものの描画フラグで描画
  pub fn draw(&self, x: i32, y: i32) {
    set_drawcolor_idx(self.col_idx);
    let mut add_flags: u32 = 0;
    if self.xflip { add_flags |= BLIT_FLIP_X }
    if self.yflip { add_flags |= BLIT_FLIP_Y }
    if self.rot   { add_flags |= BLIT_ROTATE }
    blit(self.raw.data, x, y, self.raw.width, self.raw.height, self.raw.flags | add_flags);
  }
  // 画像の描画フラグは無視して新たにフラグを用いて描画
  pub fn drawf(&self, x: i32, y: i32, flags: u32) {
    set_drawcolor_idx(self.col_idx);
    blit(self.raw.data, x, y, self.raw.width, self.raw.height, self.raw.flags | flags);
  }
  // 画像そのものの描画フラグで描画(一部のみ)
  pub fn draw_sub(&self, x: i32, y: i32, w: u32, h: u32, sx: u32, sy: u32) {
    set_drawcolor_idx(self.col_idx);
    let mut add_flags: u32 = 0;
    if self.xflip { add_flags |= BLIT_FLIP_X }
    if self.yflip { add_flags |= BLIT_FLIP_Y }
    if self.rot   { add_flags |= BLIT_ROTATE }
    blit_sub(self.raw.data, x, y, w, h, sx, sy, self.raw.width, self.raw.flags | add_flags);
  }
  // 画像の描画フラグは無視して新たにフラグを用いて描画(一部のみ)
  pub fn draw_subf(&self, x: i32, y: i32, w: u32, h: u32, sx: u32, sy: u32, flags: u32) {
    set_drawcolor_idx(self.col_idx);
    blit_sub(self.raw.data, x, y, w, h, sx, sy, self.raw.width, self.raw.flags | flags);
  }
}

#[derive(Default)]
pub struct Timeline {
  pub images: &'static[&'static Image],
  pub wait_frames: Vec<u8>,
  pub frame_count: u8,
  pub now_idx: u8,
  pub max_idx: u8,
}
impl Timeline {
  pub fn new(images: &'static [&'static Image], wait_frames: Vec<u8>) -> Self {
    Self { images, wait_frames, max_idx: images.len() as u8, frame_count: 0, now_idx: 0, }
  }
  pub fn play(&mut self) {
    self.frame_count += 1;
    if self.frame_count >= self.wait_frames[self.now_idx as usize] {
      self.now_idx += 1;
      self.frame_count = 0;
      if self.now_idx >= self.max_idx { self.now_idx = 0; }
    }
  }
  pub fn draw(&self, x: i32, y: i32, flags: u32) {
    self.images[self.now_idx as usize].drawf(x, y, flags);
  }
  pub fn reset(&mut self) {
    self.now_idx = 0;
    self.frame_count = 0;
  }

}

// -------------------------------
// Collision
// -------------------------------


// -------------------------------
// Color
// -------------------------------
pub fn set_drawcolor(fcol: u16, scol: u16) {
  unsafe { *DRAW_COLORS = (scol << 4) | fcol; }
}
pub fn set_drawcolor_idx(idx: u16) {
  unsafe { *DRAW_COLORS = idx }
}

// -------------------------------
// Math
// -------------------------------
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vec2i {
  pub x: i16,
  pub y: i16
}
impl Vec2i {
  pub fn new(x: i16, y: i16) -> Self {
    Vec2i { x, y }
  }
  pub fn zero() -> Self {
    Vec2i { x: 0, y: 0 }
  }
}


