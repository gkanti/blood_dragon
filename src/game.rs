
use crate::{scenes::*, utils::update_btn};
use crate::wasm4::*;
pub struct Game {
  scene: SceneHandler,
}

impl Game {
  pub fn new() -> Self {
    Self { scene: SceneHandler::new() }
  }
  pub fn start(&mut self) {
    self.scene.start()
  }

  pub fn update(&mut self) {
    update_btn();
    self.scene.update();
    self.scene.draw();

  }
  
}