#![crate_name = "rusthello_lib"]
#![crate_type = "lib"]


// External crates
extern crate rand;
extern crate rayon;
extern crate reversi;
extern crate termion;
extern crate smallvec;

// Modules
pub mod interface;
pub mod human_player;
pub mod ai_player;
pub mod custom_ai;
pub mod bit_board;

use reversi::{ReversiError};
use reversi::game::{PlayerAction};
use std::result;

pub enum OtherAction {
    Help,
    Quit,
}

pub type Action = PlayerAction<OtherAction>;
pub type Result<T> = result::Result<T, ReversiError>;
