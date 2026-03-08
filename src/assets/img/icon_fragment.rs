// icon_fragment
use crate::wasm4::*;
use crate::utils::RawImage;

const ICON_FRAGMENT_WIDTH: u32 = 8;
const ICON_FRAGMENT_HEIGHT: u32 = 8;
const ICON_FRAGMENT_FLAGS: u32 = BLIT_2BPP;
const ICON_FRAGMENT: [u8; 16] = [ 0x00,0x54,0x01,0xbd,0x06,0xd5,0x1b,0x59,0x6d,0x64,0x76,0x90,0x5a,0x40,0x15,0x00 ];

pub const RAWIMG_ICON_FRAGMENT: RawImage = RawImage {
    width: ICON_FRAGMENT_WIDTH,
    height: ICON_FRAGMENT_HEIGHT,
    flags: ICON_FRAGMENT_FLAGS,
    data: &ICON_FRAGMENT,
};

