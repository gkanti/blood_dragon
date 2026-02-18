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
  pub fn draw(&self) {
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
  pub fn draw(&self) {
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
  fn draw(&self);
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
    if is_just_pressed(BTN_2) { self.is_start = true; }
  }
  fn draw(&self) {
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
pub struct SceneMain {
  dragon: Dragon,
  stage: StageHandler,
  is_stage_clear: bool,
}
impl SceneMain {
  pub fn new() -> Self {
    Self {
      dragon: Dragon::new(),
      stage: StageHandler::new(),
      is_stage_clear: false,
    }
  }
}
impl SceneBehavior for SceneMain {
  fn start(&mut self) {
    let start_pos = self.stage.get_start_pos();
    self.dragon.pos.x = start_pos.0;
    self.dragon.pos.y = start_pos.1;
  }
  fn update(&mut self) {
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
      self.is_stage_clear = true;
    }
    text(self.is_stage_clear.to_string(), 0, 0);
    // ステージ移行処理
    if self.is_stage_clear {
      let is_exists_next_stage = self.stage.goto_next_stage();
      if is_exists_next_stage {
        let pos = self.stage.get_start_pos();
        self.dragon.reset(pos.0, pos.1);
        self.is_stage_clear = false;
      }
    }

    // 更新処理
    self.stage.update(interactive_cmds);
    self.dragon.update();
  }

  fn draw(&self) {
    // ステージ描画
    let cam_x = -self.dragon.pos.x + 70;
    let cam_y = -self.dragon.pos.y + 80;
    self.stage.draw(cam_x, cam_y);
    // プレイヤー描画
    self.dragon.draw(cam_x, cam_y);
    // HPゲージの描画
    set_drawcolor(2, 3);
    for i in 0..self.dragon.hp as i32 {
      rect(5 + 6 * i, 145, 5, 10);
    }
  }
  
  fn get_scene_cmd(&self) -> SceneCmd {
    SceneCmd::None
  }


}






