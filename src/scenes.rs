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
    Self { now_scene: SceneData::new(SceneId::Title) }
  }
  pub fn start(&mut self) {
    self.now_scene.start();
  }
  pub fn update(&mut self) {
    self.now_scene.update();
    let cmd = self.now_scene.get_scene_cmd();
    match cmd {
      SceneCmd::Change(sid) => {
        //text((sid as usize).to_string(), 0, 0);
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
}
impl SceneData {
  pub fn new(id: SceneId) -> Self {
    match id {
      SceneId::Title => SceneData::Title(SceneTitle::new()),
      SceneId::Main => SceneData::Main(SceneMain::new())
    }

  }
  pub fn start(&mut self) {
    match self {
      SceneData::Title(s) => s.start(),
      SceneData::Main(s) => s.start(),
    }
  }
  pub fn update(&mut self) {
    match self {
      SceneData::Title(s) => s.update(),
      SceneData::Main(s) => s.update(),
    }
  }
  pub fn draw(&mut self) {
    match self {
      SceneData::Title(s) => s.draw(),
      SceneData::Main(s) => s.draw(),
    }
  }
  pub fn get_scene_cmd(&self) -> SceneCmd {
    match self {
      SceneData::Title(s) => s.get_scene_cmd(),
      SceneData::Main(s) => s.get_scene_cmd(),
    }
  }
}

// シーンID
pub enum SceneId {
  Title,
  Main,
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
    if is_just_pressed(BTN_Z) { self.is_start = true; }
  }
  fn draw(&mut self) {
    set_drawcolor_idx(0x04);
    text("blood dragon", 28, 60);
    text(b"press \x81 to start", 15, 120);
  }
  fn get_scene_cmd(&self) -> SceneCmd {
    if self.is_start {
      return SceneCmd::Change(SceneId::Main)
    }
    else { return SceneCmd::None }
  }
}

// -------------------------------
// メイン
// -------------------------------
const DISP_STAGE_NAME_FRAME: u16 = 90;
pub struct SceneMain {
  dragon: Dragon,
  stage: StageHandler,
  disp_stage_name_clock: Clock,
  dragon_death_clock: Clock,
  fix_cam_pos: Vec2i,
  is_start_stage: bool,
  is_clear_stage: bool,
  is_clear_all_stage: bool,
  is_fix_cam: bool,
}
impl SceneMain {
  pub fn new() -> Self {
    Self {
      dragon: Dragon::new(),
      stage: StageHandler::new(),
      disp_stage_name_clock: Clock::new(DISP_STAGE_NAME_FRAME),
      dragon_death_clock: Clock::new(90),
      fix_cam_pos: Vec2i::zero(),
      is_start_stage: true,
      is_clear_stage: false,
      is_clear_all_stage: false,
      is_fix_cam: false,
    }
  }
}
impl SceneBehavior for SceneMain {
  fn start(&mut self) {
    let start_pos = self.stage.get_start_pos();
    self.dragon.pos.x = start_pos[0];
    self.dragon.pos.y = start_pos[1];
  }
  fn update(&mut self) {
    if self.is_start_stage {
      self.disp_stage_name_clock.tick();
      if self.disp_stage_name_clock.is_time_out() {
        self.is_start_stage = false;
        self.disp_stage_name_clock.reset();
      }
      return
    }

    if self.dragon.is_death {
      self.dragon_death_clock.tick();
      if self.dragon_death_clock.is_time_out() {
        self.dragon_death_clock.reset();
        self.stage.reload();
        let start_pos = self.stage.get_start_pos();
        self.dragon.reset(start_pos[0], start_pos[1]);
        self.is_fix_cam = false;
        self.is_start_stage = true;
      }
      else {
        self.is_fix_cam = true;
        self.dragon.update();
      }

      return
    }

    // 横軸の移動処理
    self.dragon.update_x();
    let tiles = self.stage.get_tiles_from_pos(self.dragon.pos.x, self.dragon.pos.y, DRAGON_WIDTH, DRAGON_HEIGHT);
    self.dragon.check_collision_x(tiles[0], tiles[1], tiles[2], tiles[3]);
    // 縦軸の移動距離
    self.dragon.update_y();
    let tiles = self.stage.get_tiles_from_pos(self.dragon.pos.x, self.dragon.pos.y, DRAGON_WIDTH, DRAGON_HEIGHT);
    self.dragon.check_collision_y(tiles[0], tiles[1], tiles[2], tiles[3]);
    // アイテム等、インタラクティブな物への衝突判定処理
    let tiles = self.stage.get_tiles_from_pos(self.dragon.pos.x, self.dragon.pos.y, DRAGON_WIDTH, DRAGON_HEIGHT);
    let interactive_cmds = self.dragon.check_interactive(tiles[0], tiles[1], tiles[2], tiles[3]);

    // ステージクリア時処理
    if interactive_cmds.contains(&Some(InteractiveCmd::ClearStage)) {
      self.is_clear_stage = true;
    }

    if is_just_pressed(BTN_X) {
      self.dragon.hp -= 1;
    }

    // 更新処理
    self.stage.update(interactive_cmds);
    self.dragon.update();

    // ステージ移行処理
    if self.is_clear_stage {
      let exists_next_stage = self.stage.goto_next_stage();
      if exists_next_stage {
        let pos = self.stage.get_start_pos();
        self.dragon.reset(pos[0], pos[1]);
      } else { self.is_clear_all_stage = true; } // オールクリア
      self.is_start_stage = true;
      self.is_clear_stage = false;
    }


  }

  fn draw(&mut self) {
    if !self.is_fix_cam {
      // ステージ描画
      let cam_x = -self.dragon.pos.x + 70;
      let cam_y = -self.dragon.pos.y + 80;
      self.stage.draw(cam_x, cam_y);
      // プレイヤー描画
      self.dragon.draw(cam_x, cam_y);
      self.fix_cam_pos.x = cam_x;
      self.fix_cam_pos.y = cam_y;
    }
    else {
      self.stage.draw(self.fix_cam_pos.x, self.fix_cam_pos.y);
      self.dragon.draw(self.fix_cam_pos.x, self.fix_cam_pos.y);
    }

    

    // HPゲージの描画
    set_drawcolor(2, 3);
    for i in 0..self.dragon.hp as i32 {
      rect(5 + 6 * i, 145, 5, 10);
    }

    if self.is_start_stage {
      set_drawcolor(2, 4);
      let stage = String::from("stage ");
      let stage_idx = &(self.stage.get_stage_idx() + 1).to_string();
      text_center_x(stage + stage_idx, 50);
    }

  }
  
  fn get_scene_cmd(&self) -> SceneCmd {
    if self.is_clear_all_stage { SceneCmd::Change(SceneId::Title) }
    else { SceneCmd::None }
  }


}






