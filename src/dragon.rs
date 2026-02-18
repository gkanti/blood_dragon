use crate::wasm4::*;
use crate::utils::*;
use crate::assets::img::*;
use crate::stage::*;

pub const DRAGON_WIDTH: i16 = 16;
pub const DRAGON_HEIGHT: i16 = 16;
const JUMP_POWER: i16 = -4;
const MAX_FALL_SPD_Y: i16 = 4;
const FUNBARI_TIME: u8 = 60;
const INV_TIME: u8 = 90;

#[derive(Copy, Clone, PartialEq)]
enum State {
  Idle,
  Walk,
  Jump,
  Fly,
  Fall,
}
pub struct Dragon {
  anim: [Timeline; 5],
  pub pos: Vec2i,
  vel: Vec2i,
  pub hp: u8,
  
  now_state: State,
  old_state: State,
  move_frames: u8,
  jump_frames: u8,
  fly_frames: u8,
  inv_frames: u8,

  is_jump: bool,
  is_inv: bool,
  on_ground: bool,
  xflip: bool,
  yflip: bool,
  rot: bool,
}

fn is_solid_tile(tile: Option<&Tile>) -> bool {
  if let Some(t) = tile {
    matches!(t.id, TileId::Wall | TileId::NeedleDown | TileId::NeedleLeft | TileId::NeedleRight | TileId::NeedleUp)
  } else { false }
}
fn is_needle_tile(tile: Option<&Tile>) -> bool {
  if let Some(t) = tile {
    matches!(t.id, TileId::NeedleDown | TileId::NeedleLeft | TileId::NeedleRight | TileId::NeedleUp)
  } else { false }
}

impl Dragon {
  pub fn reset(&mut self, x: i16, y: i16) {
    self.pos.x = x;
    self.pos.y = y;
    self.now_state = State::Idle;
    self.old_state = State::Idle;
    self.move_frames = 0;
    self.jump_frames = 0;
    self.fly_frames = 0;
    self.inv_frames = 0;
    self.is_jump = false;
    self.is_inv = false;
    self.on_ground = true;
    self.xflip = false;
    self.yflip = false;
    self.rot = false;
  }
  pub fn new() -> Self {
    Self { anim: [
      Timeline::new(ANIM_DRAGON_IDLE, [255].to_vec()),
      Timeline::new(ANIM_DRAGON_WALK, [6, 6, 6].to_vec()),
      Timeline::new(ANIM_DRAGON_JUMP, [255].to_vec()),
      Timeline::new(ANIM_DRAGON_FLY,  [4, 4].to_vec()),
      Timeline::new(ANIM_DRAGON_FALL, [255].to_vec()),
      ],
      pos: Vec2i::zero(), vel: Vec2i::zero(), hp: 6,
      now_state: State::Idle, old_state: State::Idle,
      move_frames: 0, jump_frames: 0, fly_frames: 0, inv_frames: 0,
      is_jump: false, is_inv: false, on_ground: true, xflip: false, yflip: false, rot: false
    }
  }

  pub fn check_interactive(&mut self, tl: Option<&Tile>, tr: Option<&Tile>, bl: Option<&Tile>, br: Option<&Tile>) -> [Option<InteractiveCmd>; 4] {
    // クロージャを使ってみたかっただけ
    let get_cmd = |t: Option<&Tile>| -> Option<InteractiveCmd> {
      if let Some(tile) = t {
        match tile.id {
          TileId::Key => { Some(InteractiveCmd::GetKey(tile.pos.x, tile.pos.y)) }
          TileId::Fragment => { Some(InteractiveCmd::GetFragment(tile.pos.x, tile.pos.y)) }
          TileId::GoalOpened => { Some(InteractiveCmd::ClearStage) }
          _ => { None }
        }
      } else { None }
    };

    return [get_cmd(tl), get_cmd(tr), get_cmd(bl), get_cmd(br)]

  }
  pub fn check_collision_x(&mut self, tl: Option<&Tile>, tr: Option<&Tile>, bl: Option<&Tile>, br: Option<&Tile>) {
    // 移動していなかったらNOP
    if self.vel.x == 0 { return }
    
    // 左移動時
    if self.vel.x < 0 {

      if is_solid_tile(tl) || is_solid_tile(bl) {
        // ダメージ判定
        if (is_needle_tile(tl) || is_needle_tile(bl)) && !self.is_inv {
          self.hp -= 1;
          self.is_inv = true;
        }
        // 壁判定
        let tile = tl.unwrap();
        let tile_right = tile.pos.x + DRAGON_WIDTH;
        if self.pos.x < tile_right {
          self.pos.x = tile_right;
          self.vel.x = 0;
        }
      }

    }
    // 右移動時
    else if self.vel.x > 0 {

      if is_solid_tile(tr) || is_solid_tile(br) {
        // ダメージ判定
        if (is_needle_tile(tr) || is_needle_tile(br)) && !self.is_inv {
          self.hp -= 1;
          self.is_inv = true;
        }
        // 壁判定
        let tile = tr.unwrap();
        let tile_left = tile.pos.x;
        if self.pos.x + DRAGON_WIDTH > tile_left {
          self.pos.x = tile_left - DRAGON_WIDTH;
          self.vel.x = 0;
        }
      }

    }

  }
  pub fn check_collision_y(&mut self, tl: Option<&Tile>, tr: Option<&Tile>, bl: Option<&Tile>, br: Option<&Tile>) {
    // 移動していなかったらNOP
    if self.vel.y == 0 { return }

    // 落下時
    if self.vel.y > 0 {

      if is_solid_tile(bl) || is_solid_tile(br) {
        // ダメージ処理
        if (is_needle_tile(bl) || is_needle_tile(br)) && !self.is_inv {
          self.hp -= 1;
          self.is_inv = true;
        }
        // 壁判定
        let tile = bl.unwrap();
        let tile_top = tile.pos.y;
        let dragon_btm = self.pos.y + DRAGON_HEIGHT;
        if dragon_btm >= tile_top {
          // 着地処理
          self.pos.y = tile_top - DRAGON_HEIGHT;
          self.vel.y = 0;
          self.fly_frames = 0;
          self.jump_frames = 0;
          self.on_ground = true;
        }
      // 着地処理
      } else { self.on_ground = false; }

    }
    // 上昇時
    else if self.vel.y < 0 {
      
      if is_solid_tile(tl) || is_solid_tile(tr) {
        // ダメージ処理
        if (is_needle_tile(tl) || is_needle_tile(tr)) && !self.is_inv {
          self.hp -= 1;
          self.is_inv = true;
        }
        // 壁判定
        let tile = tl.unwrap();
        let tile_btm = tile.pos.y + DRAGON_HEIGHT;
        let dragon_top = self.pos.y;
        if dragon_top < tile_btm {
          // 頭打ち
          self.pos.y = tile_btm;
          self.vel.y = 0;
        }
      }

    }


  }

  pub fn update_x(&mut self) {
    // ----------------
    // X軸の処理
    // ----------------
    self.vel.x = 0;

    // 入力を取得
    if is_pressed(BTN_RIGHT) { self.vel.x += 1; }
    if is_pressed(BTN_LEFT)  { self.vel.x -= 1; }

    if self.vel.x != 0 {
      self.now_state = State::Walk;
      self.xflip = self.vel.x < 0; // 反転
      
      self.move_frames += 1;
      // 4fに一度の加速処理
      if self.move_frames >= 3 { 
        if self.vel.x > 0 { self.vel.x += 1; }
        else { self.vel.x -= 1; }
        self.move_frames = 0;
      }
    } else {
      self.move_frames = 0;
      self.now_state = State::Idle;
    }

    // 座標を更新
    self.pos.x+= self.vel.x as i16;
  }

  pub fn update_y(&mut self) {
    // --------
    // Y軸の処理
    // --------
    self.vel.y += 1;

    // 入力を取得
    if is_just_pressed(BTN_2) && self.on_ground {
      self.vel.y = JUMP_POWER;
      self.is_jump = true;
      self.on_ground = false; }

    // 空中の挙動
    if !self.on_ground {
      // 降下中
      if self.vel.y > 0 {
        // 速度制限
        if self.vel.y >= MAX_FALL_SPD_Y { self.vel.y = MAX_FALL_SPD_Y; }
        // 踏ん張り
        if is_pressed(BTN_2) {
          self.fly_frames += 1;
          if self.fly_frames <= FUNBARI_TIME {
            self.now_state = State::Fly;
            // fly_framesが奇数時のみ踏ん張りで上昇
            if self.fly_frames & 0b01 == 0b01 { self.vel.y = 0; } else { self.vel.y = 1; }
          } else { self.now_state = State::Fall }

        // 通常落下
        } else { self.now_state = State::Fall; }
      }
      // 上昇中
      else {
        self.now_state = State::Jump;
        if is_pressed(BTN_2) { self.jump_frames += 1; }
        else { self.jump_frames = 0; self.is_jump = false; }

        if self.jump_frames <= 10 && self.is_jump { self.vel.y -= 1; }
      }

    }

    // 座標を更新
    self.pos.y += self.vel.y as i16;
  }

  pub fn update(&mut self) {
    // 無敵時間さん
    if self.is_inv {
      self.inv_frames += 1;
      if self.inv_frames > INV_TIME {
        self.inv_frames = 0;
        self.is_inv = false;
      }
    }

    // アニメーション更新
    if self.now_state != self.old_state {
      self.anim[self.old_state as usize].reset();
    }
    self.anim[self.now_state as usize].play();
    self.old_state = self.now_state;
  }

  pub fn draw(&self, offset_x: i16, offset_y: i16) {
    // 無敵の点滅処理(2fに一回描画をパスする)
    if self.is_inv && self.inv_frames & 0b10 == 0b10 { return; }

    let mut flag: u32 = 0;
    if self.xflip { flag |= BLIT_FLIP_X }
    if self.yflip { flag |= BLIT_FLIP_Y }
    if self.rot   { flag |= BLIT_ROTATE }
    self.anim[self.now_state as usize].draw((self.pos.x + offset_x) as i32, (self.pos.y + offset_y) as i32, flag);
  }



}