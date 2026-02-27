use crate::wasm4::*;
use crate::utils::*;
use crate::assets::img::*;
use crate::stage::*;

pub const DRAGON_WIDTH: i16 = 16;
pub const DRAGON_HEIGHT: i16 = 16;
const JUMP_POWER: i16 = -4;
const MAX_HP: u8 = 6;
const MAX_FALL_SPD_Y: i16 = 4;
const FUNBARI_TIME: u8 = 60;
const INV_TIME: u8 = 90;
const SPRING_FORCE_X: i16 = 12;
const SPRING_FORCE_Y: i16 = 18;

#[derive(Copy, Clone, PartialEq)]
enum State {
  Idle,
  Walk,
  Jump,
  Fly,
  Fall,
  Death,
}
pub struct Dragon {
  anim: [Timeline; 6],
  pub pos: Vec2i,
  vel: Vec2i,
  pub force: Vec2i,
  pub hp: u8,
  pub frag_count: u8,
  
  now_state: State,
  old_state: State,
  evt_death_clock: Clock,

  move_frames: u8,
  jump_frames: u8,
  fly_frames: u8,
  inv_frames: u8,
  death_frames: u8,
  is_jump: bool,
  is_inv: bool,
  pub is_death: bool,

  on_ground: bool,
  xflip: bool,
  yflip: bool,
  rot: bool,
}

fn is_solid_tile(tile: Option<&Tile>) -> bool {
  if let Some(t) = tile {
    matches!(t.id, TileId::Wall | TileId::SpringVert | TileId::SpringHori | TileId::NeedleDown | TileId::NeedleLeft | TileId::NeedleRight | TileId::NeedleUp)
  } else { false }
}

impl Dragon {
  pub fn reset(&mut self, pos: Vec2i) {
    self.pos = pos;
    self.vel = Vec2i::zero();
    self.force = Vec2i::zero();
    self.hp = MAX_HP;
    self.frag_count = 0;
    self.anim[self.now_state as usize].reset();
    self.now_state = State::Idle;
    self.old_state = State::Idle;
    self.evt_death_clock.reset();
    self.move_frames = 0;
    self.jump_frames = 0;
    self.fly_frames = 0;
    self.inv_frames = 0;
    self.death_frames = 0;
    self.is_jump = false;
    self.is_inv = false;
    self.is_death = false;
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
      Timeline::new(ANIM_DRAGON_DEATH, [5, 5, 5, 5].to_vec())
      ],
      pos: Vec2i::zero(), vel: Vec2i::zero(), force: Vec2i::zero(), hp: MAX_HP, frag_count: 0,
      now_state: State::Idle, old_state: State::Idle, evt_death_clock: Clock::new(80),
      move_frames: 0, jump_frames: 0, fly_frames: 0, inv_frames: 0, death_frames: 0,
      is_jump: false, is_inv: false, is_death: false, on_ground: true, xflip: false, yflip: false, rot: false
    }
  }
  fn check_damage(&mut self, tile1: Option<&Tile>, tile2: Option<&Tile>, id: TileId) {
    // お前は既に死んでいる
    if self.is_death || self.is_inv { return }

    let mut damaged = false;
    // 針の方向によってダメージが通るか判定
    if let Some(t) = tile1 { if t.id == id { damaged = true } }
    if let Some(t) = tile2 { if t.id == id { damaged = true } }
    if damaged {
      self.hp -= 1;
      // 無敵になる
      self.is_inv = true;
    }
  }
  fn check_death(&mut self) {
    if self.hp <= 0 {
      self.is_death = true;
      self.force.y -= 8;
    }
  }
  pub fn check_interactive(&mut self, tl: Option<&Tile>, tr: Option<&Tile>, bl: Option<&Tile>, br: Option<&Tile>) -> [Option<InteractiveCmd>; 4] {
    // クロージャを使ってみたかっただけ
    let get_cmd = |t: Option<&Tile>| -> Option<InteractiveCmd> {
      if let Some(tile) = t {
        match tile.id {
          TileId::Key => { Some(InteractiveCmd::GetKey(tile.local_x, tile.local_y)) }
          TileId::Fragment => { Some(InteractiveCmd::GetFragment(tile.local_x, tile.local_y)) }
          TileId::GoalOpened => { Some(InteractiveCmd::ClearStage) }
          _ => { None }
        }
      } else { None }
    };

    return [get_cmd(tl), get_cmd(tr), get_cmd(bl), get_cmd(br)]

  }
  pub fn check_collision_x(&mut self, tl: Option<&Tile>, tr: Option<&Tile>, bl: Option<&Tile>, br: Option<&Tile>) {
    // 左移動時
    // ダメージ判定
    self.check_damage(tl, bl, TileId::NeedleRight);
    // 壁となるタイルかどうか判定
    if is_solid_tile(tl) || is_solid_tile(bl) {
      // 壁判定
      let tile = tl.unwrap(); // 壁の右端の座標を知りたいだけなのでtl, blどちらでもいい
      let tile_right = tile.get_global_pos_x() + DRAGON_WIDTH;
      if self.pos.x < tile_right {
        self.pos.x = tile_right;
        self.vel.x = 0;
      }
    }
    // ばね判定
    let mut bounced = false;
    if let Some(t) = tl { if t.id == TileId::SpringHori { bounced = true } }
    if let Some(t) = bl { if t.id == TileId::SpringHori { bounced = true } }
    if bounced { self.force.x = SPRING_FORCE_X }

    // 右移動時
    // ダメージ判定
    self.check_damage(tr, br, TileId::NeedleLeft);
    // 壁判定
    if is_solid_tile(tr) || is_solid_tile(br) {
      let tile = tr.unwrap();
      let tile_left = tile.get_global_pos_x() as i16;
      if self.pos.x + DRAGON_WIDTH > tile_left {
        self.pos.x = tile_left - DRAGON_WIDTH;
        self.vel.x = 0;
      }
    }
    // ばね判定
    let mut bounced = false;
    if let Some(t) = tr { if t.id == TileId::SpringHori { bounced = true } }
    if let Some(t) = br { if t.id == TileId::SpringHori { bounced = true } }
    if bounced { self.force.x = -SPRING_FORCE_X }



  }
  pub fn check_collision_y(&mut self, tl: Option<&Tile>, tr: Option<&Tile>, bl: Option<&Tile>, br: Option<&Tile>) {
    // 落下時

    // ダメージ判定
    self.check_damage(bl, br, TileId::NeedleUp);
    // ばね判定
    let mut bounced = false;
    if let Some(t) = bl { if t.id == TileId::SpringVert { bounced = true } }
    if let Some(t) = br { if t.id == TileId::SpringVert { bounced = true } }
    if bounced { self.force.y = -SPRING_FORCE_Y }

    if is_solid_tile(bl) || is_solid_tile(br) {
      // 壁判定
      let tile = bl.unwrap();
      let tile_top = tile.get_global_pos_y();
      let dragon_btm = self.pos.y + DRAGON_HEIGHT;
      if dragon_btm >= tile_top {
        // 着地処理
        self.pos.y = tile_top - DRAGON_HEIGHT;
        self.vel.y = 0;
        self.fly_frames = 0;
        self.jump_frames = 0;
        self.on_ground = true && !bounced; // ばねは接地しない
      }
    // 着地処理
    } else { self.on_ground = false; }



    // 上昇時

    // ダメージ判定
    self.check_damage(tl, tr, TileId::NeedleDown);

    if is_solid_tile(tl) || is_solid_tile(tr) {
      // 壁判定
      let tile = tl.unwrap();
      let tile_btm = tile.get_global_pos_y() + DRAGON_HEIGHT;
      let dragon_top = self.pos.y;
      if dragon_top < tile_btm {
        // 頭打ち
        self.pos.y = tile_btm;
        self.vel.y = 0;
        self.force.y = 0;
      }
    }
    // ばね判定
    let mut bounced = false;
    if let Some(t) = tl { if t.id == TileId::SpringVert { bounced = true } }
    if let Some(t) = tr { if t.id == TileId::SpringVert { bounced = true } }
    if bounced { self.force.y = SPRING_FORCE_Y }

  }

  pub fn update_x(&mut self) {
    // ----------------
    // X軸の処理
    // ----------------
    self.vel.x = 0;
    if self.force.x < 0 { self.force.x += 1 }
    else if self.force.x > 0 { self.force.x -= 1 }
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
    self.pos.x += self.vel.x + self.force.x;
  }

  pub fn update_y(&mut self) {
    // --------
    // Y軸の処理
    // --------
    self.vel.y += 1;
    if self.force.y < 0 { self.force.y += 1 }
    else if self.force.y > 0 { self.force.y -= 1 }

    // 入力を取得
    if is_just_pressed(BTN_Z) && self.on_ground {
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
        if is_pressed(BTN_Z) {
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
        if is_pressed(BTN_Z) { self.jump_frames += 1; }
        else { self.jump_frames = 0; self.is_jump = false; }

        if self.jump_frames <= 10 && self.is_jump { self.vel.y -= 1; }
      }

    }

    // 座標を更新
    self.pos.y += self.vel.y + self.force.y;
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

    self.check_death();
  }

  pub fn update_death(&mut self) {
    self.death_frames += 1;
    if self.force.y > 0 { self.force.y += 1; } 
    if self.death_frames & 0b10 == 0b10 { self.vel.y += 1 }
    self.anim[State::Death as usize].play();
    let dx = self.vel.x + self.force.x;
    let mut dy = self.vel.y + self.force.y;
    if dy > MAX_FALL_SPD_Y { dy = MAX_FALL_SPD_Y }
    self.pos.x += dx;
    self.pos.y += dy;
  }

  pub fn draw(&self, offset_x: i16, offset_y: i16) {
    // 無敵の点滅処理(2fに一回描画をパスする)
    if self.is_inv && self.inv_frames & 0b10 == 0b10 { return; }

    let mut flag: u32 = 0;
    if self.xflip { flag |= BLIT_FLIP_X }
    if self.yflip { flag |= BLIT_FLIP_Y }
    if self.rot   { flag |= BLIT_ROTATE }
    self.anim[self.now_state as usize].drawf((self.pos.x + offset_x) as i32, (self.pos.y + offset_y) as i32, flag);
  }

  pub fn draw_death(&self, offset_x: i16, offset_y: i16) {
    // 死亡時
    if self.is_death {
      self.anim[State::Death as usize].draw((self.pos.x + offset_x) as i32, (self.pos.y + offset_y) as i32)
    }
  }

}