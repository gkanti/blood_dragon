use crate::assets::img::{TILE_STAGE};
use crate::utils::*;

// -------------------------------
// Enums
// -------------------------------
// ステージ識別用のID
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StageID {
  Stage1,
  Stage2,
  Stage3,
  Stage4,
  FalseEnding,
  TrueEnding,
}
// タイル識別用のID
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TileId {
  Empty,
  Start,
  Wall,
  NeedleUp,
  NeedleRight,
  NeedleLeft,
  NeedleDown,
  SpringVert,
  SpringHori,
  GoalClosed,
  GoalOpened,
  Key,
  Fragment,
}
// プレイヤーの行動でステージに変化がある際のコマンド集
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InteractiveCmd {
  None,
  GetKey(u8, u8),
  GetFragment(u8, u8),
  ClearStage,
}

// -------------------------------
// Structs
// -------------------------------
// 不変の要素
#[derive(Copy, Clone, Debug)]
pub struct Tile {
  pub id: TileId,
  pub local_x: u8,
  pub local_y: u8,
}
impl Tile {
  pub fn new(id: TileId, local_x: u8, local_y: u8) -> Self {
    Self { id, local_x, local_y }
  }
  pub fn get_global_pos_x(&self) -> i16 {
    return self.local_x as i16 * STAGE_TILE_SIZE as i16
  }
  pub fn get_global_pos_y(&self) -> i16 {
    return self.local_y as i16 * STAGE_TILE_SIZE as i16
  }
  pub fn get_global_pos(&self) -> Vec2i {
    return Vec2i { x: self.get_global_pos_x(), y: self.get_global_pos_y() }
  }
}

// ステージのデータ格納用
struct StageData {
  width: u8,
  height: u8,
  data: &'static [u8],
}
impl StageData {
  // 文字のステージデータをTileIDへ変換する
  pub fn get_tiles(&self, out: &mut Vec<Tile>) {

    let size = self.width as u16 * self.height as u16;
    // サイズ超過の場合 panic
    if size > MAX_STAGE_DATA_SIZE { panic!("size exceeded") }
    // メモリ確保
    out.clear();
    out.reserve(size as usize);
    let mut x: u8 = 0;
    let mut y: u8 = 0;
    let mut idx: u16 = 0;
    let mut turning: u16 = self.width as u16;

    while idx < size {
      out.push( Tile::new(self.ascii_to_tile_id(self.data[idx as usize]), x, y) );
      idx += 1;
      // 重い計算をしたくない
      if idx < turning { x += 1; }
      else             { x = 0; y += 1; turning += self.width as u16; }
    }

  }
  // ヘルパー
  fn ascii_to_tile_id(&self, c: u8) -> TileId {
    match c {
      b' ' => { TileId::Empty }
      b'#' => { TileId::Wall }
      b'^' => { TileId::NeedleUp }
      b'>' => { TileId::NeedleRight }
      b'<' => { TileId::NeedleLeft }
      b'v' => { TileId::NeedleDown }
      b'Z' => { TileId::SpringVert }
      b'N' => { TileId::SpringHori }
      b'@' => { TileId::Start }
      b'k' => { TileId::Key }
      b'f' => { TileId::Fragment }
      b'g' => { TileId::GoalClosed }
      b'O' => { TileId::GoalOpened }
      _    => { TileId::Empty }
    }

  }
}

// ステージ描画補佐
pub struct StageHandler {
  now_stage: &'static StageData,
  start_tile_idx: usize,
  goal_tile_idx:  usize,
  tiles: Vec<Tile>
}
impl StageHandler {
  pub fn new(id: StageID) -> Self {
    let mut result = Self {
      now_stage: DAT_STAGE_ALL[id as usize],
      start_tile_idx: 0,
      goal_tile_idx: 0,
      tiles: Vec::with_capacity(MAX_STAGE_DATA_SIZE as usize)
    };

    result.setup();
    return result;
  }
  fn setup(&mut self) {
    self.now_stage.get_tiles(&mut self.tiles);
    self.start_tile_idx = self.tiles.iter().position(|t| t.id == TileId::Start).unwrap();
    self.goal_tile_idx = self.tiles.iter().position(|t| t.id == TileId::GoalClosed).unwrap();
  }

  fn get_idx_from_tile_pos(&self, x: u8, y: u8) -> usize {
    return self.now_stage.width as usize * y as usize + x as usize;
  }

  fn get_idx_from_pos(&self, x: i16, y: i16) -> i16 {
    let ix = x / STAGE_TILE_SIZE as i16;
    let iy = y / STAGE_TILE_SIZE as i16;
    
    let result = self.now_stage.width as i16 * iy + ix;
    if x < 0 { return result - 1; }
    else     { return result; }
  }

  fn get_tile_from_tile_pos(&mut self, x: u8, y: u8) -> &mut Tile {
    let idx = self.get_idx_from_tile_pos(x, y);
    return &mut self.tiles[idx as usize];
  }

  fn accept_cmd(&mut self, cmd: InteractiveCmd) {
    match cmd {
      InteractiveCmd::GetKey(x, y) => {
        let tile = self.get_tile_from_tile_pos(x, y);
        if tile.id != TileId::Empty {
          tile.id = TileId::Empty;
          self.tiles[self.goal_tile_idx].id = TileId::GoalOpened;
        }
      }
      InteractiveCmd::GetFragment(x, y) => {
        let tile = self.get_tile_from_tile_pos(x, y);
        if tile.id != TileId::Empty {
          tile.id = TileId::Empty;
        }
      }
      _ => {}
    }
  }

  pub fn update(&mut self, cmds: [InteractiveCmd; 4]) {
    self.accept_cmd(cmds[0]);
    self.accept_cmd(cmds[1]);
    self.accept_cmd(cmds[2]);
    self.accept_cmd(cmds[3]);

  }

  pub fn goto_next_stage(&mut self, id: StageID) {
    self.now_stage = DAT_STAGE_ALL[id as usize];
    self.setup();
  }

  pub fn reload(&mut self) {
    self.setup();
  }

  pub fn get_start_pos(&self) -> Vec2i {
    let start_tile = self.tiles[self.start_tile_idx];
    return Vec2i::new(start_tile.local_x as i16 * STAGE_TILE_SIZE as i16, start_tile.local_y as i16 * STAGE_TILE_SIZE as i16)
  }
  // 受け取った座標に存在するタイルを返す
  pub fn get_tile_from_pos(&self, px: i16, py: i16) -> Option<&Tile> {
    let idx = self.get_idx_from_pos(px, py);
    // 範囲内のidxならタイル、範囲外ならNoneを返却
    if idx >= 0 && idx < self.tiles.len() as i16 {
      return Some(&self.tiles[idx as usize])
    } else { None }
  }
  pub fn get_tiles_from_pos(&self, x: i16, y: i16, w: i16, h: i16) -> [Option<&Tile>; 4] {
    [
      self.get_tile_from_pos(x, y),
      self.get_tile_from_pos(x+w-1, y),
      self.get_tile_from_pos(x, y+h-1),
      self.get_tile_from_pos(x+w-1, y+h-1),
    ]
  }

  pub fn draw(&self, offset_x: i16, offset_y: i16) {
    const FENCE_VALUE: i16 = 11;
    let mut idx = self.get_idx_from_pos(-offset_x, -offset_y);
    let mut count = 0;
    let mut fence = FENCE_VALUE;

    while count < FENCE_VALUE * FENCE_VALUE {
      count += 1;
      if count > fence {
        fence += FENCE_VALUE;
        idx += self.now_stage.width as i16 - FENCE_VALUE;
      }
      if idx < 0 || idx >= self.tiles.len() as i16 { idx += 1; continue; }
      let tile = &self.tiles[idx as usize];
      // 空のタイルは描画しない
      if tile.id == TileId::Empty || tile.id == TileId::Start { idx += 1; continue; }

      let draw_pos_x = tile.get_global_pos_x() + offset_x;
      let draw_pos_y = tile.get_global_pos_y() + offset_y;
      TILE_STAGE[tile.id as usize].draw(draw_pos_x as i32, draw_pos_y as i32);
      idx += 1;
    }
    // ここ最適化できそう
    //for i in 0..self.tiles.len() {
    //  let tile = &self.tiles[i];
    //  // 空のタイルは描画しない
    //  if tile.id == TileId::Empty || tile.id == TileId::Start { continue; }
    //  
    //  let draw_pos_x = tile.get_global_pos_x() + offset_x;
    //  let draw_pos_y = tile.get_global_pos_y() + offset_y;
    //  
    //  // 画面内のタイルのみ描画する
    //  if (draw_pos_x + STAGE_TILE_SIZE as i16) < 0 || draw_pos_x > 160 ||
    //     (draw_pos_y + STAGE_TILE_SIZE as i16) < 0 || draw_pos_y > 160
    //     { continue; }
    //  
    //  TILE_STAGE[tile.id as usize].draw(draw_pos_x as i32, draw_pos_y as i32);
    //
    //}
  }

}




// -------------------------------
// Stage Data
// -------------------------------
 // タイルの大きさ
const STAGE_TILE_SIZE: u8 = 16;
// ステージの大きさ制限 (タイルのidx, グローバル座標は i16 の範囲に収まるはず)
const MAX_STAGE_DATA_SIZE: u16 = 1024;
// 全てのステージデータを格納
const DAT_STAGE_ALL: &[&'static StageData; 6] = &[
  &DAT_STAGE_1,
  &DAT_STAGE_2,
  &DAT_STAGE_3,
  &DAT_STAGE_4,
  &DAT_STAGE_FALSE_ENDING,
  &DAT_STAGE_TRUE_ENDING,
];

// Stage 1
const STAGE_1_WIDTH: u8 = 32;
const STAGE_1_HEIGHT: u8 = 20;
const STAGE_1: [u8; STAGE_1_WIDTH as usize*STAGE_1_HEIGHT as usize] = [
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b'g',b' ',b'k',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'f',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b' ',b' ',b' ',b'#',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b'#',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b' ',b' ',b' ',b'#',b'#',b' ',b' ',b'#',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'^',b'^',b'^',b'#',b'#',b' ',b' ',b'#',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'^',b'^',b'#',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'f',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b' ',b' ',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b' ',b' ',b' ',b' ',b'#',b'#',b'#',b' ',b' ',b' ',b' ',b'#',b'#',b'#',b'#',
  b'#',b'@',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b'#',b'#',b' ',b' ',b' ',b' ',b'#',b'#',b' ',b' ',b' ',b'#',b'#',b'#',b'#',b'#',b' ',b' ',b'#',b'#',b'#',b'#',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
];
const DAT_STAGE_1: StageData = StageData {
  width: STAGE_1_WIDTH,
  height: STAGE_1_HEIGHT,
  data: &STAGE_1
};

// Stage 2
const STAGE_2_WIDTH: u8 = 22;
const STAGE_2_HEIGHT: u8 = 30;
const STAGE_2: [u8; STAGE_2_WIDTH as usize*STAGE_2_HEIGHT as usize] = [
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b'#',
  b'#',b'f',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b'#',b' ',b' ',b'#',b'#',b' ',b' ',b'#',b'#',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b'#',b'#',b'^',b'^',b'#',b'#',b' ',b' ',b'#',b'#',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b' ',b' ',b'#',b'#',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b'#',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b'#',
  b'#',b'^',b'^',b' ',b' ',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b' ',b'^',b' ',b' ',b'^',b' ',b' ',b'#',b'#',b'#',
  b'#',b'#',b'#',b' ',b' ',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b' ',b'#',b' ',b' ',b'#',b' ',b' ',b'#',b'#',b'#',
  b'#',b'#',b'#',b'#',b'#',b' ',b' ',b' ',b'#',b' ',b' ',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'v',b'v',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b'#',b' ',b' ',b'#',b' ',b' ',b'#',b'#',b'#',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b'#',b'^',b'^',b'#',b'^',b'^',b'#',b'#',b'#',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b'#',b'#',b' ',b' ',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b' ',b' ',b' ',b' ',b' ',b'#',b'f',b' ',b' ',b' ',b' ',b' ',b'#',b'>',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b' ',b' ',b' ',b' ',b'#',b'#',b' ',b' ',b' ',b' ',b' ',b'#',b'>',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b' ',b' ',b'#',b'#',b' ',b' ',b'k',b' ',b' ',b'#',b'#',b'#',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b' ',b' ',b' ',b' ',b' ',b'#',b'>',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b' ',b' ',b' ',b' ',b' ',b'#',b'>',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b'#',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b'#',b'#',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b'@',b' ',b' ',b' ',b'#',b'#',b'#',b'g',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
];
const DAT_STAGE_2: StageData = StageData {
  width: STAGE_2_WIDTH,
  height: STAGE_2_HEIGHT,
  data: &STAGE_2
};


const STAGE_3_WIDTH: u8 = 12;
const STAGE_3_HEIGHT: u8 = 84;
const STAGE_3: [u8; STAGE_3_WIDTH as usize*STAGE_3_HEIGHT as usize] = [
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b' ',b'#',b'>',b' ',b' ',b' ',b'<',b'#',
  b'#',b'g',b' ',b' ',b' ',b'#',b'>',b' ',b' ',b' ',b'<',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b'>',b' ',b' ',b' ',b'<',b'#',
  b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b'Z',b'Z',b'#',b'#',
  b'#',b'>',b' ',b' ',b' ',b' ',b'#',b'#',b'#',b'#',b'#',b'#',
  b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b'k',b' ',b'<',b'#',
  b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b'>',b'Z',b'Z',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b'>',b' ',b' ',b'Z',b'Z',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b'#',b'^',b'^',b'^',b'^',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b'#',b' ',b'Z',b'Z',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b'#',b'^',b'^',b'^',b'^',b'^',b'#',
  b'#',b' ',b' ',b' ',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
  b'#',b' ',b' ',b'Z',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
  b'#',b' ',b' ',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b'#',b'#',b'#',b'#',
  b'#',b' ',b' ',b' ',b' ',b'f',b'#',b'#',b'#',b'#',b'#',b'#',
  b'#',b' ',b' ',b' ',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
  b'#',b' ',b' ',b' ',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
  b'#',b' ',b' ',b' ',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
  b'#',b'Z',b' ',b' ',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
  b'#',b'#',b' ',b' ',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
  b'#',b' ',b' ',b' ',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
  b'#',b' ',b' ',b' ',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'f',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b'Z',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'>',b' ',b' ',b' ',b' ',b'Z',b'Z',b'#',
  b'#',b'#',b'#',b'#',b'>',b' ',b' ',b'#',b'#',b'#',b'#',b'#',
  b'#',b'#',b'#',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'>',b'Z',b'Z',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'Z',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'Z',b'#',
  b'#',b' ',b' ',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
  b'#',b' ',b' ',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b'#',b'#',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b'#',b'#',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b'#',b'#',b'#',
  b'#',b'Z',b'Z',b' ',b' ',b' ',b' ',b'#',b'#',b'#',b'#',b'#',
  b'#',b'#',b'#',b'#',b'#',b' ',b' ',b'#',b'#',b'#',b'#',b'#',
  b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',
  b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',
  b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',
  b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',
  b'>',b' ',b'@',b' ',b' ',b'Z',b'Z',b' ',b' ',b' ',b' ',b'<',
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
];
const DAT_STAGE_3: StageData = StageData {
  width: STAGE_3_WIDTH,
  height: STAGE_3_HEIGHT,
  data: &STAGE_3,
};

const STAGE_4_WIDTH: u8 = 32;
const STAGE_4_HEIGHT: u8 = 30;
const STAGE_4: [u8; STAGE_4_WIDTH as usize*STAGE_4_HEIGHT as usize] = [
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
  b'#',b' ',b'@',b' ',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'f',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b' ',b' ',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b'>',b' ',b' ',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b'>',b' ',b' ',b' ',b' ',b' ',b'^',b'^',b'^',b'^',b'^',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b' ',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b'^',b' ',b' ',b'^',b'#',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b' ',b'#',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b' ',b'#',b'#',b' ',b'#',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b'#',b'>',b'Z',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b' ',b'#',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b'#',b'>',b'#',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b' ',b'#',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b' ',b' ',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b'^',b' ',b' ',b'^',b'#',b'#',b'>',b' ',b' ',b'^',b'^',b'^',b'^',b'^',b'#',b' ',b' ',b' ',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b' ',b'#',b'#',b'>',b' ',b' ',b'#',b'#',b'#',b'#',b'#',b'#',b' ',b' ',b' ',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b' ',b'#',b'#',b' ',b'#',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b' ',b' ',b'#',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b' ',b' ',b'f',b' ',b'#',b'#',b'>',b' ',b' ',b' ',b' ',b'k',b' ',b' ',b'#',b' ',b' ',b' ',b'#',b'>',b' ',b' ',b' ',b' ',b'Z',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b' ',b'#',b'#',b'>',b' ',b' ',b' ',b' ',b'g',b' ',b' ',b'#',b' ',b' ',b' ',b'#',b'>',b' ',b' ',b' ',b' ',b'#',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b'^',b'^',b'^',b' ',b'#',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'#',b' ',b' ',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b'v',b'v',b'v',b' ',b'#',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b' ',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b' ',b' ',b' ',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b' ',b' ',b'#',b'#',b' ',b' ',b'N',b' ',b' ',b' ',b' ',b' ',b' ',b'N',b' ',b' ',b'#',b' ',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'>',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',b'>',b'Z',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b'^',b'^',b' ',b' ',b' ',b' ',b'N',b' ',b' ',b' ',b' ',b' ',b' ',b'N',b' ',b' ',b' ',b'#',b'#',b'>',b'#',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'>',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'<',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b' ',b' ',b'#',b'>',b' ',b' ',b'N',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'<',b'#',
  b'#',b' ',b' ',b' ',b' ',b'^',b'^',b' ',b' ',b'^',b'^',b'^',b' ',b' ',b' ',b' ',b' ',b'#',b'>',b' ',b' ',b'N',b' ',b' ',b' ',b' ',b'Z',b'Z',b'Z',b' ',b'<',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
];
const DAT_STAGE_4: StageData = StageData {
  width: STAGE_4_WIDTH,
  height: STAGE_4_HEIGHT,
  data: &STAGE_4,
};
const STAGE_FALSE_ENDING_WIDTH: u8 = 16;
const STAGE_FALSE_ENDING_HEIGHT: u8 = 16;
const STAGE_FALSE_ENDING: [u8; STAGE_FALSE_ENDING_WIDTH as usize*STAGE_FALSE_ENDING_HEIGHT as usize] = [
  b'k',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'@',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'g',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
];
const DAT_STAGE_FALSE_ENDING: StageData = StageData {
  width: STAGE_FALSE_ENDING_WIDTH,
  height: STAGE_FALSE_ENDING_HEIGHT,
  data: &STAGE_FALSE_ENDING,
};
const STAGE_TRUE_ENDING_WIDTH: u8 = 16;
const STAGE_TRUE_ENDING_HEIGHT: u8 = 16;
const STAGE_TRUE_ENDING: [u8; STAGE_TRUE_ENDING_WIDTH as usize*STAGE_TRUE_ENDING_HEIGHT as usize] = [
  b'k',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'@',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b'g',b' ',b' ',b' ',b' ',b' ',b' ',b'#',
  b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',b'#',
];
const DAT_STAGE_TRUE_ENDING: StageData = StageData {
  width: STAGE_TRUE_ENDING_WIDTH,
  height: STAGE_TRUE_ENDING_HEIGHT,
  data: &STAGE_TRUE_ENDING,
};