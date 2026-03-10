use crate::assets::img::IMG_ENDING_PHOTO_01;
use crate::assets::img::IMG_ICON_FRAGMENT;
use crate::assets::img::IMG_ITEM_FRAGMENT;
use crate::wasm4::*;
use crate::utils::*;
use crate::dragon::*;
use crate::stage::*;

// シーン管理
pub struct SceneHandler {
  now_scene: SceneContainer,
}
impl SceneHandler {
  pub fn new() -> Self {
    Self { now_scene: SceneContainer::new(SceneId::Title) }
  }
  // 初期化処理
  pub fn start(&mut self) {
    self.now_scene.start();
  }
  // 更新処理
  pub fn update(&mut self) {
    self.now_scene.update();
    let cmd = self.now_scene.get_scene_cmd();
    // シーンから受け取ったコマンドを元に分岐処理
    match cmd {
      SceneCmd::Change(sid) => {
        self.now_scene = SceneContainer::new(sid);
        self.now_scene.start();
      },
      _ => { }
    }
  }
  // 描画処理
  pub fn draw(&mut self) {
    self.now_scene.draw();
  }
}


// シーンを包む
enum SceneContainer {
  Title(SceneTitle),
  Main(SceneMain),
  FalseEnding(SceneFalseEnding),
  TrueEnding(SceneTrueEnding),
  StaffRoll(SceneStaffRoll),
}
impl SceneContainer {
  pub fn new(id: SceneId) -> Self {
    match id {
      SceneId::Title =>       SceneContainer::Title(SceneTitle::new()),
      SceneId::Main =>        SceneContainer::Main(SceneMain::new()),
      SceneId::EndingFalse => SceneContainer::FalseEnding(SceneFalseEnding::new()),
      SceneId::EndingTrue =>  SceneContainer::TrueEnding(SceneTrueEnding::new()),
      SceneId::StaffRoll =>   SceneContainer::StaffRoll(SceneStaffRoll::new())
    }
  }
  pub fn start(&mut self) {
    match self {
      SceneContainer::Title(s) =>             s.start(),
      SceneContainer::Main(s) =>               s.start(),
      SceneContainer::FalseEnding(s) => s.start(),
      SceneContainer::TrueEnding(s) =>   s.start(),
      SceneContainer::StaffRoll(s) =>     s.start(),
    }
  }
  pub fn update(&mut self) {
    match self {
      SceneContainer::Title(s) =>             s.update(),
      SceneContainer::Main(s) =>               s.update(),
      SceneContainer::FalseEnding(s) => s.update(),
      SceneContainer::TrueEnding(s) =>   s.update(),
      SceneContainer::StaffRoll(s) =>     s.update(),
    }
  }
  pub fn draw(&mut self) {
    match self {
      SceneContainer::Title(s) =>             s.draw(),
      SceneContainer::Main(s) =>               s.draw(),
      SceneContainer::FalseEnding(s) => s.draw(),
      SceneContainer::TrueEnding(s) =>   s.draw(),
      SceneContainer::StaffRoll(s) =>     s.draw(),
    }
  }
  pub fn get_scene_cmd(&self) -> SceneCmd {
    match self {
      SceneContainer::Title(s) =>             s.get_scene_cmd(),
      SceneContainer::Main(s) =>               s.get_scene_cmd(),
      SceneContainer::FalseEnding(s) => s.get_scene_cmd(),
      SceneContainer::TrueEnding(s) =>   s.get_scene_cmd(),
      SceneContainer::StaffRoll(s) =>     s.get_scene_cmd(),
    }
  }
}

// シーンID
pub enum SceneId {
  Title,
  Main,
  EndingFalse,
  EndingTrue,
  StaffRoll,
}
// SceneHandlerに情報を伝えるコマンド
pub enum SceneCmd {
  None,
  Change(SceneId),
}

// シーンに実装するべきトレイト
trait SceneBehavior {
  fn start(&mut self);
  fn update(&mut self);
  fn draw(&mut self);
  fn get_scene_cmd(&self) -> SceneCmd;
}

// -------------------------------
// タイトル
// -------------------------------
pub struct SceneTitle {
  is_start: bool,
}
impl SceneTitle {
  pub fn new() -> Self {
    Self { is_start: false }
  }
}
impl SceneBehavior for SceneTitle {
  fn start(&mut self) {}

  fn update(&mut self) {
    // ボタン押下でスタート
    if is_just_pressed(BTN_Z) { self.is_start = true; }
  }

  fn draw(&mut self) {
    set_drawcolor_idx(0x04);
    text("blood dragon", 28, 60);
    text(b"press \x81 to start", 15, 120);
  }
  fn get_scene_cmd(&self) -> SceneCmd {
    if self.is_start { SceneCmd::Change(SceneId::Main) }
    else             { SceneCmd::None }
  }
}

// -------------------------------
// メイン
// -------------------------------
const CAM_OFFSET_X: i16 = 72;
const CAM_OFFSET_Y: i16 = 76;

pub struct SceneMain {
  dragon: Dragon,
  stage: StageHandler,
  disp_stage_name_clock: Clock,
  dragon_death_clock: Clock,
  fix_cam_pos: Vec2i,
  stage_frag_count: u8,
  total_frag_count: u8,
  now_stage_id: StageID,
  is_start_stage: bool,
  is_clear_all_stage: bool,
  is_fix_cam: bool,
}

impl SceneMain {
  pub fn new() -> Self {
    Self {
      dragon: Dragon::new(),
      stage: StageHandler::new(StageID::Stage1),
      disp_stage_name_clock: Clock::new(90),
      dragon_death_clock: Clock::new(90),
      fix_cam_pos: Vec2i::zero(),
      stage_frag_count: 0,
      total_frag_count: 0,
      now_stage_id: StageID::Stage1,
      is_start_stage: true,
      is_clear_all_stage: false,
      is_fix_cam: false,
    }
  }
}

impl SceneBehavior for SceneMain {
  fn start(&mut self) {
    self.dragon.pos = self.stage.get_start_pos();
  }
  fn update(&mut self) {
    // ステージ開始時のみステージ名を描画、その他の更新処理をパス
    if self.is_start_stage {
      self.disp_stage_name_clock.tick();
      if self.disp_stage_name_clock.is_time_out() {
        self.is_start_stage = false;
        self.disp_stage_name_clock.reset();
      }
      return;
    }

    // プレイヤー死亡時の特別処理、その他の更新処理をパス
    if self.dragon.is_death {
      self.dragon_death_clock.tick();
      // 死亡アニメーションの再生
      if !self.dragon_death_clock.is_time_out() {
        self.is_fix_cam = true;
        self.dragon.update_death();
      // 死亡から一定時間が経過したら復活処理
      } else {
        self.stage_frag_count = 0;
        self.dragon_death_clock.reset();
        self.stage.reload();
        self.dragon.reset(self.stage.get_start_pos());
        self.is_fix_cam = false;
        self.is_start_stage = true;
      }
      return
    }

    let mut tiles: [Option<&Tile>; 4];
    // 横軸の移動処理
    self.dragon.update_x();
    tiles = self.stage.get_tiles_from_pos(self.dragon.pos.x, self.dragon.pos.y, DRAGON_WIDTH, DRAGON_HEIGHT);
    self.dragon.check_collision_x(tiles[0], tiles[1], tiles[2], tiles[3]);
    // 縦軸の移動距離
    self.dragon.update_y();
    tiles = self.stage.get_tiles_from_pos(self.dragon.pos.x, self.dragon.pos.y, DRAGON_WIDTH, DRAGON_HEIGHT);
    self.dragon.check_collision_y(tiles[0], tiles[1], tiles[2], tiles[3]);
    // アイテム等、インタラクティブな物への衝突判定処理
    tiles = self.stage.get_tiles_from_pos(self.dragon.pos.x, self.dragon.pos.y, DRAGON_WIDTH, DRAGON_HEIGHT);
    let interactive_cmds = self.dragon.check_interactive(tiles[0], tiles[1], tiles[2], tiles[3]);
    
    // コマンドの処理
    let mut get_frag = false;
    for i in 0..interactive_cmds.len() {
      match interactive_cmds[i] {
        // かけら取得処理(重複取得を避ける)
        InteractiveCmd::GetFragment(_, _) => {
          if !get_frag { self.stage_frag_count += 1; get_frag = true; } 
        }
        // ステージ更新処理
        InteractiveCmd::ClearStage => {
          // 集めた欠片を集計
          self.total_frag_count += self.stage_frag_count;
          self.stage_frag_count = 0;
          // ステージ移行処理
          if self.is_start_stage { return; } // すでに移行済みならreturn
          match self.now_stage_id {
            StageID::Stage1 | StageID::Stage2 | StageID::Stage3 => {
              match self.now_stage_id {
                StageID::Stage1 => self.now_stage_id = StageID::Stage2,
                StageID::Stage2 => self.now_stage_id = StageID::Stage3,
                StageID::Stage3 => self.now_stage_id = StageID::Stage4,
                _ => return,
              }
              // 次のステージの初期化処理
              self.stage.goto_next_stage(self.now_stage_id);
              self.dragon.reset(self.stage.get_start_pos());
              self.is_start_stage = true;
            }
            // クリア処理
            StageID::Stage4 => self.is_clear_all_stage = true,
            _ => return,
          }
        }
        _ => { }
      }
    }

    // 更新処理
    self.stage.update(interactive_cmds);
    self.dragon.update();
  }

  fn draw(&mut self) {
    // 通常カメラ
    if !self.is_fix_cam {
      let cam_x = -self.dragon.pos.x + CAM_OFFSET_X;
      let cam_y = -self.dragon.pos.y + CAM_OFFSET_Y;
      self.stage.draw(cam_x, cam_y);
      if !self.dragon.is_death { self.dragon.draw(cam_x, cam_y); } 
      else                     { self.dragon.draw_death(cam_x, cam_y); }
      // カメラ座標の保存
      self.fix_cam_pos.x = cam_x;
      self.fix_cam_pos.y = cam_y;
    // 固定カメラ
    } else {
      self.stage.draw(self.fix_cam_pos.x, self.fix_cam_pos.y);
      if !self.dragon.is_death { self.dragon.draw(self.fix_cam_pos.x, self.fix_cam_pos.y); }
      else                     { self.dragon.draw_death(self.fix_cam_pos.x, self.fix_cam_pos.y); }
    }

    // HPゲージの描画
    set_drawcolor(2, 3);
    for i in 0..self.dragon.hp as i32 {
      rect(5 + 6 * i, 145, 5, 10);
    }

    // ステージ開始時のみステージ名を描画
    if self.is_start_stage {
      set_drawcolor(2, 4);
      let stage = String::from("stage ");
      let stage_idx = &(self.now_stage_id as u8 + 1).to_string();
      text_center_x(stage + stage_idx, 50);
    }

    // 欠片の個数の描画
    IMG_ICON_FRAGMENT.draw(140, 1);
    set_drawcolor(2, 4);
    text((self.stage_frag_count + self.total_frag_count).to_string(), 150, 1);

  }
  
  fn get_scene_cmd(&self) -> SceneCmd {
    if self.is_clear_all_stage {
      // エンディング分岐
      if self.total_frag_count == 8 { SceneCmd::Change(SceneId::EndingTrue) }
      else                          { SceneCmd::Change(SceneId::EndingFalse) }
    }
    else { SceneCmd::None }
  }


}

// -------------------------------
// エンディング(偽)
// -------------------------------
const CAM_FIX_POS_X: i16 = -96 + CAM_OFFSET_X;
const CAM_FIX_POS_Y: i16 = -216 + CAM_OFFSET_Y;

// Eventに格納する関数群
fn dragon_walk_r(d: &mut Dragon) { d.evt_walk(1); }
fn dragon_walk_l(d: &mut Dragon) { d.evt_walk(-1); }
fn dragon_stop(d: &mut Dragon)   { d.evt_stop(); }
fn dragon_draw(d: &Dragon) { d.draw(CAM_FIX_POS_X, CAM_FIX_POS_Y); }

const EVENTS_FALSE_ENDING: [Event<Dragon>; 9] = [
  Event::new(60,  Some(dragon_stop),   Some(dragon_draw)),
  Event::new(60,  Some(dragon_walk_r), Some(dragon_draw)),
  Event::new(60,  Some(dragon_stop),   Some(dragon_draw)),
  Event::new(5,   Some(dragon_walk_l), Some(dragon_draw)),
  Event::new(60,  Some(dragon_stop),   Some(dragon_draw)),
  Event::new(5,   Some(dragon_walk_r), Some(dragon_draw)),
  Event::new(120, Some(dragon_stop),   Some(dragon_draw)),
  Event::new(60,  Some(dragon_walk_l), Some(dragon_draw)),
  Event::new(120, Some(dragon_stop),   Some(dragon_draw)),
];

pub struct SceneFalseEnding {
  dragon: Dragon,
  stage: StageHandler,
  timeline: TimeLine<Dragon, 9>,
  is_end_event: bool,
}
impl SceneFalseEnding {
  pub fn new() -> Self {
    Self { 
      dragon: Dragon::new(),
      stage: StageHandler::new(StageID::FalseEnding),
      timeline: TimeLine::new(&EVENTS_FALSE_ENDING),
      is_end_event: false,
    }
  }
}

impl SceneBehavior for SceneFalseEnding {
  fn start(&mut self) {
    self.dragon.pos = self.stage.get_start_pos();
    self.dragon.is_operable = false;
  }
  fn update(&mut self) {
    if self.is_end_event { return; }

    self.timeline.update(&mut self.dragon);
    self.is_end_event = self.timeline.is_end;

    self.dragon.update();

  }
  fn draw(&mut self) {
    self.stage.draw( CAM_FIX_POS_X, CAM_FIX_POS_Y);
    self.timeline.draw(&self.dragon);

    if self.is_end_event {
      set_drawcolor(4, 0);
      text_center_x("THE END...?", 60);
      text_center_x(b"press \x81", 124);
      text_center_x("to return title", 132);
    }
  }
  fn get_scene_cmd(&self) -> SceneCmd {
    if self.is_end_event {
      if is_just_pressed(BTN_Z) { return SceneCmd::Change(SceneId::Title) }
    }
    return SceneCmd::None;
  }
}

// -------------------------------
// エンディング(真)
// -------------------------------
fn frag_update(f: &mut FragmentAnim) { f.frame_count += 1; }
fn frag_gather(f: &mut FragmentAnim) {
  f.frame_count += 1;
  for i in 0..f.frags_pos.len() {
    // 中心までのベクトルを移動に用いる
    let v = -f.frags_pos[i] / 5;
    if f.frame_count & 0b11 != 0b11 { return; }
    f.frags_pos[i] = f.frags_pos[i] + v;
  }
}
fn frag_draw_blinking(f: &FragmentAnim) {
  // 点滅処理
  if f.frame_count & 0b11 == 0b11 { return; }
  frag_draw(f);
}
fn frag_draw(f: &FragmentAnim) {
  for i in 0..f.frags_pos.len() {
    let draw_pos = f.frags_pos[i] + f.offset;
    IMG_ITEM_FRAGMENT.draw(draw_pos.x as i32, draw_pos.y as i32);
  }
}


const EVENTS_TRUE_ENDING_DRAGON: [Event<Dragon>; 9] = [
  Event::new(60,  None,                Some(dragon_draw)),
  Event::new(60,  Some(dragon_walk_r), Some(dragon_draw)),
  Event::new(60,  Some(dragon_stop),   Some(dragon_draw)),
  Event::new(180, None,                Some(dragon_draw)),
  Event::new(1,   None,                Some(dragon_draw)),
  Event::new(60,  None,                Some(dragon_draw)),
  Event::new(35,  Some(dragon_walk_r), Some(dragon_draw)),
  Event::new(1,   Some(dragon_stop),   Some(dragon_draw)),
  Event::new(120, None,                None),
];
const EVENTS_TRUE_ENDING_FRAG: [Event<FragmentAnim>; 5] = [
  Event::new(60,  None,              None),
  Event::new(100, None,              None),
  Event::new(60,  Some(frag_update), Some(frag_draw_blinking)),
  Event::new(80,  Some(frag_gather), Some(frag_draw)),
  Event::new(60,  Some(frag_update), Some(frag_draw_blinking)),
];


pub struct SceneTrueEnding {
  dragon: Dragon,
  stage: StageHandler,
  timeline_dragon: TimeLine<Dragon, 9>,
  timeline_frag: TimeLine<FragmentAnim, 5>,
  frag_anim: FragmentAnim,
  is_end_event: bool,
  is_opened_door: bool,
}
impl SceneTrueEnding {
  pub fn new() -> Self {
    Self { 
      dragon: Dragon::new(),
      stage: StageHandler::new(StageID::TrueEnding),
      timeline_dragon: TimeLine::new(&EVENTS_TRUE_ENDING_DRAGON),
      timeline_frag: TimeLine::new(&EVENTS_TRUE_ENDING_FRAG),
      frag_anim: FragmentAnim::new(),
      is_end_event: false,
      is_opened_door: false,
    }
  }
}
impl SceneBehavior for SceneTrueEnding {
  fn start(&mut self) {
    self.dragon.pos = self.stage.get_start_pos();
    self.dragon.is_operable = false;
  }
  fn update(&mut self) {
    if self.is_end_event { return; }

    self.timeline_dragon.update(&mut self.dragon);
    self.timeline_frag.update(&mut self.frag_anim);

    if self.timeline_dragon.is_end { self.is_end_event = true; }
    if self.timeline_frag.is_end && !self.is_opened_door {
      self.is_opened_door = true;
      let imitation_cmd = [
        InteractiveCmd::GetKey(0, 0),
        InteractiveCmd::None,
        InteractiveCmd::None,
        InteractiveCmd::None
      ];
      self.stage.update(imitation_cmd)
    }

    self.dragon.update();
    
  }
  fn draw(&mut self) {
    self.stage.draw(CAM_FIX_POS_X, CAM_FIX_POS_Y);
    self.timeline_dragon.draw(&self.dragon);
    self.timeline_frag.draw(&self.frag_anim);

  }
  fn get_scene_cmd(&self) -> SceneCmd {
    if !self.is_end_event { return SceneCmd::None; }
    else                  { return SceneCmd::Change(SceneId::StaffRoll) }

  }
}
struct FragmentAnim {
  frame_count: u16,
  frags_pos: [Vec2i; 8],
  offset: Vec2i,
}
impl FragmentAnim {
  pub fn new() -> Self {
    let mut result = Self {
      frame_count: 0,
      frags_pos: [Vec2i::zero(); 8],
      offset: Vec2i::new(-8 + 112, 84),
    };
    result.frags_pos[0] = Vec2i::new( 16,  32);
    result.frags_pos[1] = Vec2i::new( 32,  16);
    result.frags_pos[2] = Vec2i::new( 32, -16);
    result.frags_pos[3] = Vec2i::new( 16, -32);
    result.frags_pos[7] = Vec2i::new(-16,  32);
    result.frags_pos[6] = Vec2i::new(-32,  16);
    result.frags_pos[5] = Vec2i::new(-32, -16);
    result.frags_pos[4] = Vec2i::new(-16, -32);

    return result;
  }
}

// -------------------------------
// スタッフロール
// -------------------------------
pub struct SceneStaffRoll {
}
impl SceneStaffRoll {
  pub fn new() -> Self {
    Self {  }
  }
}
impl SceneBehavior for SceneStaffRoll {
  fn start(&mut self) { }
  fn update(&mut self) { }

  fn draw(&mut self) {
    IMG_ENDING_PHOTO_01.draw(80 - (IMG_ENDING_PHOTO_01.get_width()) as i32 / 2,
                             50 - (IMG_ENDING_PHOTO_01.get_height()) as i32 / 2);
    set_drawcolor(2, 4);
    text_center_x("congratulations!", 88);
    set_drawcolor(4, 0);
    text_center_x("you saved your", 102);
    text_center_x("imprisoned sibling!", 112);
    text_center_x(b"press \x81", 128);
    text_center_x("to return title", 138);
  }
  fn get_scene_cmd(&self) -> SceneCmd {
    if !is_just_pressed(BTN_Z) { SceneCmd::None }
    else                       { SceneCmd::Change(SceneId::Title) }
  }
}