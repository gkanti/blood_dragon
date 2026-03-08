use crate::assets::img::IMG_ICON_FRAGMENT;
use crate::assets::img::IMG_ITEM_FRAGMENT;
use crate::wasm4::*;
use crate::utils::*;
use crate::dragon::*;
use crate::stage::*;

// シーン管理用
pub struct SceneHandler {
  now_scene: SceneData,
}

impl SceneHandler {
  pub fn new() -> Self {
    Self { now_scene: SceneData::new(SceneId::EndingTrue) }
  }
  pub fn start(&mut self) {
    self.now_scene.start();
  }
  pub fn update(&mut self) {
    self.now_scene.update();
    let cmd = self.now_scene.get_scene_cmd();
    match cmd {
      SceneCmd::Change(sid) => {
        self.now_scene = SceneData::new(sid);
        self.now_scene.start();
      },
      _ => {}
    }
  }
  pub fn draw(&mut self) {
    self.now_scene.draw();
  }
}


// シーンEnum
enum SceneData {
  Title(SceneTitle),
  Main(SceneMain),
  FalseEnding(SceneFalseEnding),
  TrueEnding(SceneTrueEnding),
  StaffRoll(SceneStaffRoll),
}
impl SceneData {
  pub fn new(id: SceneId) -> Self {
    match id {
      SceneId::Title =>       SceneData::Title(SceneTitle::new()),
      SceneId::Main =>        SceneData::Main(SceneMain::new()),
      SceneId::EndingFalse => SceneData::FalseEnding(SceneFalseEnding::new()),
      SceneId::EndingTrue =>  SceneData::TrueEnding(SceneTrueEnding::new()),
      SceneId::StaffRoll =>   SceneData::StaffRoll(SceneStaffRoll::new())
    }
  }
  pub fn start(&mut self) {
    match self {
      SceneData::Title(s) =>             s.start(),
      SceneData::Main(s) =>               s.start(),
      SceneData::FalseEnding(s) => s.start(),
      SceneData::TrueEnding(s) =>   s.start(),
      SceneData::StaffRoll(s) =>     s.start(),
    }
  }
  pub fn update(&mut self) {
    match self {
      SceneData::Title(s) =>             s.update(),
      SceneData::Main(s) =>               s.update(),
      SceneData::FalseEnding(s) => s.update(),
      SceneData::TrueEnding(s) =>   s.update(),
      SceneData::StaffRoll(s) =>     s.update(),
    }
  }
  pub fn draw(&mut self) {
    match self {
      SceneData::Title(s) =>             s.draw(),
      SceneData::Main(s) =>               s.draw(),
      SceneData::FalseEnding(s) => s.draw(),
      SceneData::TrueEnding(s) =>   s.draw(),
      SceneData::StaffRoll(s) =>     s.draw(),
    }
  }
  pub fn get_scene_cmd(&self) -> SceneCmd {
    match self {
      SceneData::Title(s) =>             s.get_scene_cmd(),
      SceneData::Main(s) =>               s.get_scene_cmd(),
      SceneData::FalseEnding(s) => s.get_scene_cmd(),
      SceneData::TrueEnding(s) =>   s.get_scene_cmd(),
      SceneData::StaffRoll(s) =>     s.get_scene_cmd(),
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
pub struct SceneMain {
  dragon: Dragon,
  stage: StageHandler,

  disp_stage_name_clock: Clock,
  dragon_death_clock: Clock,

  fix_cam_pos: Vec2i,

  total_frag_count: u8,
  now_stage_id: StageID,

  is_start_stage: bool,
  is_clear_all_stage: bool,
  is_fix_cam: bool,
}
const INIT_STAGE: StageID = StageID::Stage1;
impl SceneMain {
  pub fn new() -> Self {
    Self {
      dragon: Dragon::new(),
      stage: StageHandler::new(INIT_STAGE),
      disp_stage_name_clock: Clock::new(90),
      dragon_death_clock: Clock::new(90),
      fix_cam_pos: Vec2i::zero(),
      total_frag_count: 0,
      now_stage_id: INIT_STAGE,
      is_start_stage: true,
      is_clear_all_stage: false,
      is_fix_cam: false,
    }
  }
}

const CAM_OFFSET_X: i16 = 72;
const CAM_OFFSET_Y: i16 = 76;

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
      return
    }

    // プレイヤー死亡時の特別処理、その他の更新処理をパス
    if self.dragon.is_death {
      self.dragon_death_clock.tick();
      // 死亡から一定時間が経過したら復活処理
      if self.dragon_death_clock.is_time_out() {
        self.dragon_death_clock.reset();
        self.stage.reload();
        self.dragon.reset(self.stage.get_start_pos());
        self.is_fix_cam = false;
        self.is_start_stage = true;
      }
      // 死亡アニメーションの再生
      else {
        self.is_fix_cam = true;
        self.dragon.update_death();
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
    
    // ステージクリア時処理
    if interactive_cmds.contains(&InteractiveCmd::ClearStage) {
      // 集めた欠片を集計
      self.total_frag_count += self.stage.fragment_count;
      // ステージ移行処理
      match self.now_stage_id {
        // 移行処理
        StageID::Stage1 | StageID::Stage2 | StageID::Stage3 => {
          match self.now_stage_id {
            StageID::Stage1 => self.now_stage_id = StageID::Stage2,
            StageID::Stage2 => self.now_stage_id = StageID::Stage3,
            StageID::Stage3 => self.now_stage_id = StageID::Stage4,
            _ => return,
          }
          self.stage.goto_next_stage(self.now_stage_id);
          self.dragon.reset(self.stage.get_start_pos());
          self.is_start_stage = true;
        }
        // クリア処理
        StageID::Stage4 => self.is_clear_all_stage = true,
        _ => return,
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
    }
    // 固定カメラ
    else {
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
    let frag_count = self.stage.fragment_count + self.total_frag_count;
    text(frag_count.to_string(), 150, 1);

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


pub struct SceneFalseEnding {
  dragon: Dragon,
  stage: StageHandler,
  frame_count: u16,
  is_end_event: bool,
}

impl SceneFalseEnding {
  pub fn new() -> Self {
    Self { 
      dragon: Dragon::new(),
      stage: StageHandler::new(StageID::FalseEnding),
      frame_count: 0,
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
    // ゴ ミ カ ス 処 理
    if self.is_end_event { return; }

    self.frame_count += 1;
    if      self.frame_count <= 60        { self.dragon.evt_stop(); }
    else if self.frame_count <= 60  + 60  { self.dragon.evt_walk(1); }
    else if self.frame_count <= 120 + 60  { self.dragon.evt_stop(); }
    else if self.frame_count <= 180 + 10  { self.dragon.evt_walk(-1); }
    else if self.frame_count <= 190 + 60  { self.dragon.evt_stop(); }
    else if self.frame_count <= 250 + 10  { self.dragon.evt_walk(1); }
    else if self.frame_count <= 260 + 120 { self.dragon.evt_stop(); }
    else if self.frame_count <= 380 + 60  { self.dragon.evt_walk(-1); }
    else if self.frame_count >= 440 + 120 { self.is_end_event = true; }

    self.dragon.update();

  }
  fn draw(&mut self) {
    let cam_x = -96 + CAM_OFFSET_X;
    let cam_y = -216 + CAM_OFFSET_Y;
    self.stage.draw(cam_x, cam_y);
    self.dragon.draw(cam_x, cam_y);

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

pub struct SceneTrueEnding {
  dragon: Dragon,
  stage: StageHandler,
  frag_anim: FragmentAnim,
  frame_count: u16,
  is_end_event: bool,
  is_enter_door: bool,
}
struct FragmentAnim {
  frame_count: u16,
  frags_pos: [Vec2i; 8],
  offset: Vec2i,
  is_end: bool,
}

impl SceneTrueEnding {
  pub fn new() -> Self {
    Self { 
      dragon: Dragon::new(),
      stage: StageHandler::new(StageID::TrueEnding),
      frag_anim: FragmentAnim::new(),
      frame_count: 0,
      is_end_event: false,
      is_enter_door: false,
    }
  }
}
impl FragmentAnim {
  pub fn new() -> Self {
    let mut result = Self {
      frame_count: 0,
      frags_pos: [Vec2i::zero(); 8],
      offset: Vec2i::new(-8 + 112, 80),
      is_end: false,
    };

    result.frags_pos[0] = Vec2i::new(16, 32);
    result.frags_pos[1] = Vec2i::new(32, 16);
    result.frags_pos[2] = Vec2i::new(32, -16);
    result.frags_pos[3] = Vec2i::new(16, -32);
    result.frags_pos[7] = Vec2i::new(-16, 32);
    result.frags_pos[6] = Vec2i::new(-32, 16);
    result.frags_pos[5] = Vec2i::new(-32, -16);
    result.frags_pos[4] = Vec2i::new(-16, -32);

    return result;
  }
  pub fn play(&mut self) {
    if self.is_end { return; }

    self.frame_count += 1;

    if      self.frame_count <= 120       { return; }
    else if self.frame_count <= 120 + 60 {
      for i in 0..self.frags_pos.len() {
        let v = -self.frags_pos[i] / 5;
        if self.frame_count & 0b11 != 0b11 { return; }
        self.frags_pos[i] = self.frags_pos[i] + v;
      }
    }
    else if self.frame_count >= 200 { self.is_end = true; }
  }

  pub fn draw(&self) {
    if self.is_end { return; }
    // 点滅処理
    if self.frame_count <= 60 {
      if self.frame_count & 0b11 == 0b11 { return; }
    }

    for i in 0..self.frags_pos.len() {
      let draw_pos = self.frags_pos[i] + self.offset;
      IMG_ITEM_FRAGMENT.draw(draw_pos.x as i32, draw_pos.y as i32);
    }
  }
}
impl SceneBehavior for SceneTrueEnding {
  fn start(&mut self) {
    self.dragon.pos = self.stage.get_start_pos();
    self.dragon.is_operable = false;
  }
  fn update(&mut self) {
    // ゴ ミ カ ス 処 理
    if self.is_end_event { return; }

    self.frame_count += 1;
    if      self.frame_count <= 60        { self.dragon.evt_stop(); }
    else if self.frame_count <= 60  + 60  { self.dragon.evt_walk(1); }
    else if self.frame_count <= 120 + 60  { self.dragon.evt_stop(); }
    else if self.frame_count <= 180 + 200 { self.frag_anim.play(); }
    else if self.frame_count <= 380 + 1   { self.stage.update([InteractiveCmd::GetKey(0, 0), InteractiveCmd::None, InteractiveCmd::None, InteractiveCmd::None]); }
    else if self.frame_count <= 381 + 59  { self.dragon.evt_stop(); }
    else if self.frame_count <= 440 + 35  { self.dragon.evt_walk(1); }
    else if self.frame_count <= 475 + 1   { self.is_enter_door = true; }
    else if self.frame_count >= 476 + 120  { self.is_end_event = true; }

    self.dragon.update();
    
  }
  fn draw(&mut self) {
    let cam_x = -96 + CAM_OFFSET_X;
    let cam_y = -216 + CAM_OFFSET_Y;
    self.stage.draw(cam_x, cam_y);
    if !self.is_enter_door {
      self.dragon.draw(cam_x, cam_y);
    }

    if self.frame_count >= 180 { self.frag_anim.draw(); }

  }
  fn get_scene_cmd(&self) -> SceneCmd {
    if !self.is_end_event { return SceneCmd::None; }
    else                  { return SceneCmd::Change(SceneId::StaffRoll) }

  }
}

// -------------------------------
// タイトル
// -------------------------------
pub struct SceneStaffRoll {
  is_end: bool
}
impl SceneStaffRoll {
  pub fn new() -> Self {
    Self { is_end: false }
  }
}
impl SceneBehavior for SceneStaffRoll {
  fn start(&mut self) {}

  fn update(&mut self) {

  }

  fn draw(&mut self) {

  }
  fn get_scene_cmd(&self) -> SceneCmd {
    if !self.is_end { SceneCmd::None }
    else            { SceneCmd::Change(SceneId::Title) }
  }
}