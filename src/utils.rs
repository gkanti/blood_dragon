use crate::wasm4::*;
use lazy_static::lazy_static;
use core::panic;
use std::sync::Mutex;
use std::ops::{Add, Sub, Neg, Mul, Div};

// -------------------------------
// ボタン入力
// -------------------------------
lazy_static! {
  static ref BTN_HANDLER: Mutex<ButtonHandler> = Mutex::new(ButtonHandler::new());
}

pub const BTN_X:     u8 = 0;
pub const BTN_Z:     u8 = 1;
pub const BTN_LEFT:  u8 = 4;
pub const BTN_RIGHT: u8 = 5;
pub const BTN_UP:    u8 = 6;
pub const BTN_DOWN:  u8 = 7;

// ボタンの入力を更新する
pub fn update_btn() {
  BTN_HANDLER.lock().expect("btn_state").update();
}
// ボタンが押されている長さを取得する
pub fn get_pressed_time(btn: u8) -> u16 {
  BTN_HANDLER.lock().expect("btn_state").btn_states[btn as usize]
}
// ボタンが押されているか取得する
pub fn is_pressed(btn: u8) -> bool {
  BTN_HANDLER.lock().expect("btn_state").btn_states[btn as usize] >= 1
}
// ボタンが押された瞬間か取得する
pub fn is_just_pressed(btn: u8) -> bool {
  BTN_HANDLER.lock().expect("btn_state").btn_states[btn as usize] == 1
}
// ボタンが話された瞬間か取得する
pub fn is_just_released(btn: u8) -> bool {
  let handler = BTN_HANDLER.lock().expect("btn_state");
  // 「ボタンの押された長さが0である」かつ、「直近にボタンの状態が変化した」であれば離された瞬間
  (handler.btn_states[btn as usize] == 0) && (handler.just_changed_btn_states & (1 << btn) != 0)
}

pub struct ButtonHandler {
  btn_states: [u16; 8],
  old_btn_states: u8,
  just_changed_btn_states: u8
}

impl ButtonHandler {
  pub fn new() -> Self {
    Self { btn_states: [0; 8], old_btn_states: 0, just_changed_btn_states: 0}
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
// 画像関連
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
  // フラグを持たせたい時に使用する
  pub const fn newf(col_idx: u16, raw: &'static RawImage, xflip: bool, yflip: bool, rot: bool) -> Self {
    Self {col_idx, raw, xflip, yflip, rot}
  }
  // 画像の幅を取得する
  pub fn get_width(&self) -> u32 { return self.raw.width; }
  // 画像の高さを取得する
  pub fn get_height(&self) -> u32 { return self.raw.height; }
  // 画像そのものの描画フラグで描画
  pub fn draw(&self, x: i32, y: i32) {
    set_drawcolor_idx(self.col_idx);
    let mut add_flags: u32 = 0;
    if self.xflip { add_flags |= BLIT_FLIP_X }
    if self.yflip { add_flags |= BLIT_FLIP_Y }
    if self.rot   { add_flags |= BLIT_ROTATE }
    blit(self.raw.data, x, y, self.raw.width, self.raw.height, self.raw.flags | add_flags);
  }
  // 画像の描画フラグは無視し、新たにフラグを用いて描画
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
  // 画像の描画フラグは無視し、新たにフラグを用いて描画(一部のみ)
  pub fn draw_subf(&self, x: i32, y: i32, w: u32, h: u32, sx: u32, sy: u32, flags: u32) {
    set_drawcolor_idx(self.col_idx);
    blit_sub(self.raw.data, x, y, w, h, sx, sy, self.raw.width, self.raw.flags | flags);
  }
}

// -------------------------------
// アニメーション関連
// -------------------------------
pub struct Animation {
  pub images: &'static[&'static Image],
  pub wait_frames: &'static [u8],
  pub frame_count: u8,
  pub now_idx: u8,
  pub max_idx: u8,
}
impl Animation {
  pub fn new(images: &'static [&'static Image], wait_frames: &'static [u8]) -> Self {
    Self { images, wait_frames, max_idx: images.len() as u8, frame_count: 0, now_idx: 0, }
  }
  // アニメーションを再生する
  pub fn play(&mut self) {
    self.frame_count += 1;
    if self.frame_count >= self.wait_frames[self.now_idx as usize] {
      self.now_idx += 1;
      self.frame_count = 0;
      if self.now_idx >= self.max_idx { self.now_idx = 0; }
    }
  }
  // アニメーションを描画する
  pub fn draw(&self, x: i32, y: i32) {
    self.images[self.now_idx as usize].draw(x, y);
  }
  // フラグを指定してアニメーションを描画する
  pub fn drawf(&self, x: i32, y: i32, flags: u32) {
    self.images[self.now_idx as usize].drawf(x, y, flags);
  }
  // アニメーションをリセットする
  pub fn reset(&mut self) {
    self.now_idx = 0;
    self.frame_count = 0;
  }

}

// -------------------------------
// パーティクル
// -------------------------------
const MAX_PARTICLE_VALUE: u8 = 32;
#[derive(Copy, Clone)]
pub struct Particle {
  pub alive: bool,
  pub pos: Vec2i,
  pub vec: Vec2i,
  life: u8,

}
impl Particle {
  pub fn new() -> Self {
    Self { alive: false, pos: Vec2i::zero(), vec: Vec2i::zero(), life: 0}
  }
  pub fn start(&mut self, pos: Vec2i, vec: Vec2i, life: u8) {
    self.alive = true;
    self.pos = pos;
    self.vec = vec;
    self.life = life;
  }
  pub fn update(&mut self) {
    self.life -= 1;
    if self.life <= 0 { self.alive = false; return }
    self.pos.x += self.vec.x;
    self.pos.y += self.vec.y;
  }

}

// -------------------------------
// テキスト
// -------------------------------
// テキストを画面中央(横軸)に表示する
pub fn text_center_x<T: AsRef<[u8]>>(msg: T, y: i32) {
  let msg_ref = msg.as_ref();
  let x = ((160 - (msg_ref.len()*8)) / 2) as i32;
  text(msg, x, y);
}
// テキストを画面中央(縦軸)に表示する
pub fn text_center_y<T: AsRef<[u8]>>(msg: T, x: i32) {
  let y = 160 - (8 / 2);
  text(msg, x, y);
}
// テキストを画面中央に表示する
pub fn text_center<T: AsRef<[u8]>>(msg: T) {
  let msg_ref = msg.as_ref();
  let x = ((160 - (msg_ref.len()*8)) / 2) as i32;
  let y = 160 - (8 / 2);
  text(msg, x, y);
}
// -------------------------------
// 色指定
// -------------------------------
// 描画色を指定する(簡単)
pub fn set_drawcolor(fcol: u16, scol: u16) {
  unsafe { *DRAW_COLORS = (scol << 4) | fcol; }
}
// 描画色を指定する
pub fn set_drawcolor_idx(idx: u16) {
  unsafe { *DRAW_COLORS = idx }
}

// -------------------------------
// 数学系
// -------------------------------
// ベクトル
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
impl Add<Vec2i> for Vec2i {
  type Output = Vec2i;
  fn add(self, rhs: Vec2i) -> Vec2i {
    Vec2i::new(self.x + rhs.x, self.y + rhs.y)
  }
}
impl Add<i16> for Vec2i {
  type Output = Vec2i;
  fn add(self, rhs: i16) -> Vec2i {
    Vec2i::new(self.x + rhs, self.y + rhs)
  }
}
impl Sub<Vec2i> for Vec2i {
  type Output = Vec2i;
  fn sub(self, rhs: Vec2i) -> Vec2i {
    Vec2i::new(self.x - rhs.x, self.y - rhs.y)
  }
}
impl Sub<i16> for Vec2i {
  type Output = Vec2i;
  fn sub(self, rhs: i16) -> Vec2i {
    Vec2i::new(self.x - rhs, self.y - rhs)
  }
}
impl Neg for Vec2i {
  type Output = Vec2i;
  fn neg(self) -> Vec2i {
    Vec2i::new(-self.x, -self.y)
  }
}
impl Mul<Vec2i> for Vec2i {
  type Output = Vec2i;
  fn mul(self, rhs: Vec2i) -> Vec2i {
    Vec2i::new(self.x * rhs.x, self.y * rhs.y)
  }
}
impl Mul<i16> for Vec2i {
  type Output = Vec2i;
  fn mul(self, rhs: i16) -> Vec2i {
    Vec2i::new(self.x * rhs, self.y * rhs)
  }
}
impl Div<Vec2i> for Vec2i {
  type Output = Vec2i;
  fn div(self, rhs: Vec2i) -> Vec2i {
    Vec2i::new(self.x / rhs.x, self.y / rhs.y)
  }
}
impl Div<i16> for Vec2i {
  type Output = Vec2i;
  fn div(self, rhs: i16) -> Vec2i {
    Vec2i::new(self.x / rhs, self.y / rhs)
  }
}

// -------------------------------
// タイミング系
// -------------------------------
pub struct Clock {
  wait_frame: u16,
  now_frame: u16,
}
impl Clock {
  pub fn new(wait_frame: u16) -> Self {
    Self { wait_frame, now_frame: 0 }
  }
  pub fn tick(&mut self) {
    if self.now_frame >= self.wait_frame { return }
    self.now_frame += 1;
  }
  pub fn is_time_out(&self) -> bool { return self.now_frame >= self.wait_frame }
  pub fn reset(&mut self) {
    self.now_frame = 0;
  }
}

// -------------------------------
// イベント系
// -------------------------------
type UpdateProc<T> = fn(&mut T);
type DrawProc<T> = fn(&T);

trait CallableMut<T> {
  fn call_func(&self, actor: &mut T);
}
trait Callable<T> {
  fn call_func(&self, actor: &T);
}

impl<T> CallableMut<T> for UpdateProc<T> {
  fn call_func(&self, actor: &mut T) {
      (self)(actor)
  }
}
impl<T> Callable<T> for DrawProc<T> {
  fn call_func(&self, actor: &T) {
      (self)(actor)
  }
}

pub struct Event<T> {
  duration: u16,
  update: Option<UpdateProc<T>>,
  draw: Option<DrawProc<T>>
}
impl<T> Event<T> {
  pub const fn new(duration: u16, update: Option<UpdateProc<T>>, draw: Option<DrawProc<T>>) -> Self {
    Self { duration, update, draw }
  }
}

pub struct TimeLine<T: 'static, const N: usize> {
  events: &'static [Event<T>; N],
  idx: usize,
  frame: u16,
  pub is_end: bool,
}
impl<T: 'static, const N: usize> TimeLine<T, N> {
  pub const fn new(events: &'static [Event<T>; N]) -> Self {
    Self { events, idx: 0, frame: 0, is_end: false}
  }
  pub fn update(&mut self, actor: &mut T) {
    if self.is_end { return; }

    let event = &self.events[self.idx];
    if let Some(update_proc) = &event.update {
      update_proc.call_func(actor);
    }

    self.frame += 1;
    if self.frame >= event.duration {
      self.frame = 0;
      self.idx += 1;
      if self.idx >= N { self.idx = N-1; self.is_end = true; }
    }
  }
  pub fn draw(&self, actor: &T) {
    if self.is_end { return; }

    let event = &self.events[self.idx];
    if let Some(draw_proc) = &event.draw {
      draw_proc.call_func(actor);
    }
  }
}

// -------------------------------
// 音楽系
// -------------------------------

pub fn tone_frequency(freq1: u32, freq2: u32) -> u32 { freq1 | (freq2 << 16) }
pub fn tone_duration(attack: u32, decay: u32, sustain: u32, release: u32) -> u32 { (attack << 24) | (decay << 16) | sustain | (release << 8) }
pub fn tone_volume(peak: u32, volume: u32) -> u32 { (peak << 8) | volume }
pub fn tone_flags(channel: u32, mode: u32, pan: u32) -> u32 { channel | (mode << 2) | (pan << 4) }
/* 
fn to_u16(bytes: &[u8]) -> Option<u16> {
  let mut result: u16 = 0;
  for b in bytes {
    if !b.is_ascii_digit() { return None; }
    result = result * 10 + (b - b'0') as u16; 
  }
  return Some(result);
}

fn is_note(byte: u8) -> bool {
  matches!(byte, b'a'..=b'g' | b'r')
}

fn remove_spaces(ascii: &str) -> String {
  let mut result = String::with_capacity(ascii.len());

  for b in ascii.bytes() {
    if b != b' ' { result.push(b as char); }
  }
  return result;
}
pub enum MMLCmd {
  C(u8),          // c duration: u8(上位2bitを付点として扱う)
  D(u8),          // d duration: u8(上位2bitを付点として扱う)
  E(u8),          // e duration: u8(上位2bitを付点として扱う)
  F(u8),          // f duration: u8(上位2bitを付点として扱う)
  G(u8),          // g duration: u8(上位2bitを付点として扱う)
  A(u8),          // a duration: u8(上位2bitを付点として扱う)
  B(u8),          // b duration: u8(上位2bitを付点として扱う)
  Channel(u8),    // @ channel: u8(1~4)
  Portamento,     // &
  Manual(u8),     // N frequency: u8
  PanLeft,        // <
  PanRight,       // >
  Sharp,          // +
  Flat,           // -
  Rest,           // R1,2,4,8,16,32,64(., ..)
  Duration,       // C1,2,4,8,16,32,64(., ..)
  OctaveUp,       // /
  OctaveDown,     // \
  Volume,         // V100
  Attack,         // a100
  Decay,          // d100
  Release,        // r100
  Comment,        // ;, 空白, |
}

enum MMLErrCode {

}
pub struct Note {
  cmd: MMLCmd,
}
impl Note {
  pub fn new(cmd: MMLCmd) -> Self {
    Self { cmd, }
  }
}

pub struct StrStream {
  data: &'static [u8],
  idx: usize,
  max_idx: usize,
  is_end: bool,
}

impl StrStream {
  pub fn new(data: &'static str) -> Self {
    Self { data: data.as_bytes(), idx: 0, max_idx: data.len(), is_end: false }
  }
  fn remove_spaces(&mut self) {

  }
  pub fn read(&mut self) -> Option<u8> {
    if self.is_end { return None; }
  
    if self.idx >= self.max_idx { self.is_end = true; return None; }

    let mut result: u8;
    while self.idx < self.max_idx {
      result = self.data[self.idx];
      self.idx += 1;
      match result {
        b' ' | b'\n' | b'|' => { continue; }
        _ => { return Some(result) }
      }
    }
    return None

  }
  pub fn read_while<P>(&mut self, mut predicate: P) -> Vec<u8>
    where P: FnMut(u8) -> bool 
  {
    let mut result: Vec<u8> = Vec::with_capacity(8);
    while let Some(d) = self.read() {
      if predicate(d) { result.push(d); }
      else            { break; }
    }
    return result;
  }
}

pub struct MMLPlayer {
  idx: usize,
  now_channel: u8,
  ch1: Vec<Note>,
  ch2: Vec<Note>,
  ch3: Vec<Note>,
  ch4: Vec<Note>,
}

impl MMLPlayer {
  pub fn new() -> Self {
    Self {
      idx: 0, 
      now_channel: 0,
      ch1: Vec::with_capacity(128),
      ch2: Vec::with_capacity(128),
      ch3: Vec::with_capacity(128),
      ch4: Vec::with_capacity(128)
    }
  }
  pub fn read_mml(&mut self, mml: &'static str) {
    let mut stream = StrStream::new(mml);

    while let Some(cmd) = stream.read() {
      match cmd {
        // チャンネル指定
        b'@' => { self.set_channel(&mut stream); }

        // C
        b'c' => {
          let args = stream.read_while(|d| is_note(d));
          

        }
        
        // コメント(改行まで読み飛ばし)
        b';' => {
          let raw_comment: &[u8] = &stream.read_while(|d| d != b'\n')[..];
          //let comment: &str = unsafe { std::str::from_utf8_unchecked(raw_comment) };
          //trace("comment: ".to_string() + comment);
        }
        // エラー
        _ => { trace("mml syntax error.") }
      }
    }
  }
  fn set_channel(&mut self, stream: &mut StrStream) {
    let new_channel = to_u16(&stream.read_while(|b| b.is_ascii_digit()));
    if let Some(nc) = new_channel {
      if nc >= 1 && nc <= 4 { self.now_channel = nc as u8; }
      else { panic!("invalid channel idx. the index must be a value between 1 and 4.") }
      trace("set channel to ".to_string() + &nc.to_string())
    }
    else { panic!("invalid channel idx. the index must be a integer value.") }

  }
  fn read_args(&mut self, mml: &str) {
    self.idx += 1;
    let mut length: usize = 1;

  }
}
*/