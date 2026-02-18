use crate::utils::*;
pub mod error;
pub mod dragon_idle_01;
pub mod dragon_walk_01;
pub mod dragon_walk_02;
pub mod dragon_walk_03;
pub mod dragon_jump_01;
pub mod dragon_fly_01;
pub mod dragon_fly_02;
pub mod dragon_fall_01;
pub mod tile_wall;
pub mod tile_needle;
pub mod tile_gate_open;
pub mod tile_gate_close;
pub mod item_key;
pub mod item_fragment;
use error::*;
use dragon_idle_01::*;
use dragon_walk_01::*;
use dragon_walk_02::*;
use dragon_walk_03::*;
use dragon_jump_01::*;
use dragon_fly_01::*;
use dragon_fly_02::*;
use dragon_fall_01::*;
use tile_wall::*;
use tile_needle::*;
use tile_gate_open::*;
use tile_gate_close::*;
use item_key::*;
use item_fragment::*;

pub const IMG_ERROR:             Image = Image::new( 0x0432, &RAWIMG_ERROR);
pub const IMG_DRAGON_IDLE_01:    Image = Image::new( 0x0432, &RAWIMG_DRAGON_IDLE_01);
pub const IMG_DRAGON_WALK_01:    Image = Image::new( 0x0432, &RAWIMG_DRAGON_WALK_01);
pub const IMG_DRAGON_WALK_02:    Image = Image::new( 0x0432, &RAWIMG_DRAGON_WALK_02);
pub const IMG_DRAGON_WALK_03:    Image = Image::new( 0x0432, &RAWIMG_DRAGON_WALK_03);
pub const IMG_DRAGON_JUMP_01:    Image = Image::new( 0x0432, &RAWIMG_DRAGON_JUMP_01);
pub const IMG_DRAGON_FLY_01:     Image = Image::new( 0x0432, &RAWIMG_DRAGON_FLY_01);
pub const IMG_DRAGON_FLY_02:     Image = Image::new( 0x0432, &RAWIMG_DRAGON_FLY_02);
pub const IMG_DRAGON_FALL_01:    Image = Image::new( 0x0432, &RAWIMG_DRAGON_FALL_01);
pub const IMG_TILE_WALL:         Image = Image::new( 0x1243, &RAWIMG_TILE_WALL);
pub const IMG_TILE_NEEDLE_UP:    Image = Image::newf(0x4320, &RAWIMG_TILE_NEEDLE, false, false, false);
pub const IMG_TILE_NEEDLE_RIGHT: Image = Image::newf(0x4320, &RAWIMG_TILE_NEEDLE, false, true, true);
pub const IMG_TILE_NEEDLE_LEFT:  Image = Image::newf(0x4320, &RAWIMG_TILE_NEEDLE, false, false, true);
pub const IMG_TILE_NEEDLE_DOWN:  Image = Image::newf(0x4320, &RAWIMG_TILE_NEEDLE, false, true, false);
pub const IMG_TILE_GATE_OPEN:    Image = Image::new( 0x0040, &RAWIMG_TILE_GATE_OPEN);
pub const IMG_TILE_GATE_CLOSE:   Image = Image::new( 0x4320, &RAWIMG_TILE_GATE_CLOSE);
pub const IMG_ITEM_KEY:          Image = Image::new( 0x4320, &RAWIMG_ITEM_KEY);
pub const IMG_ITEM_FRAGMENT:     Image = Image::new( 0x4320, &RAWIMG_ITEM_FRAGMENT);

pub static TILE_STAGE: &[&'static Image; 11] = &[
  &IMG_ERROR,            // Empty
  &IMG_ERROR,            // Start
  &IMG_TILE_WALL,        // Wall
  &IMG_TILE_NEEDLE_UP,   // Needle Up
  &IMG_TILE_NEEDLE_RIGHT,// Needle Right
  &IMG_TILE_NEEDLE_LEFT, // Needle Left
  &IMG_TILE_NEEDLE_DOWN, // Needle Down
  &IMG_TILE_GATE_CLOSE,  // Goal(closed)
  &IMG_TILE_GATE_OPEN,   // Goal(opened)
  &IMG_ITEM_KEY,         // Key
  &IMG_ITEM_FRAGMENT,    // Fragment
];

pub static ANIM_DRAGON_IDLE: &[&'static Image; 1] = &[&IMG_DRAGON_IDLE_01];
pub static ANIM_DRAGON_WALK: &[&'static Image; 3] = &[&IMG_DRAGON_WALK_01, &IMG_DRAGON_WALK_02, &IMG_DRAGON_WALK_03];
pub static ANIM_DRAGON_JUMP: &[&'static Image; 1] = &[&IMG_DRAGON_JUMP_01];
pub static ANIM_DRAGON_FLY:  &[&'static Image; 2] = &[&IMG_DRAGON_FLY_01, &IMG_DRAGON_FLY_02];
pub static ANIM_DRAGON_FALL: &[&'static Image; 1] = &[&IMG_DRAGON_FALL_01];
