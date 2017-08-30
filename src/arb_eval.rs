//! `RUSThello`
//! A simple Reversi game written in Rust with love.
//! Based on `reversi` library (by the same author).
//! Released under MIT license.
//! by Enrico Ghiorzi

#![crate_name = "arbeval"]
#![crate_type = "bin"]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate rusthello_lib;
extern crate reversi;

use reversi::{ReversiError, Side};
use reversi::game::{PlayerAction, IsPlayer, Game};
use reversi::turn::Turn;
use reversi::board::Board;
use rusthello_lib::{OtherAction, Result};
use rusthello_lib::{interface, human_player, ai_player, custom_ai, bit_board};
use rusthello_lib::bit_board::BitBoard;
use rusthello_lib::interface::{UserCommand};
use std::cmp::Ordering;
use std::time::{Instant};

fn read_board() -> bit_board::BitBoard {
    let stdin = ::std::io::stdin();
    let mut config: String = "".to_string();
    stdin.read_line(&mut config);
    let mut bl = 0;
    let mut wh = 0;
    if config.len() < 64 {
        panic!("config must be of length 64");
    }
    let config_v: Vec<char> = config.chars().collect();
    for i in 0 .. 64 {
        match config_v[i] {
            'O' => wh |= 1u64 << i,
            'X' => bl |= 1u64 << i,
            '-' => (),
            _ => panic!("invalid disk"),
        }
    }
    let mut turn_config = "".to_string();
    stdin.read_line(&mut turn_config);
    let turn = turn_config.starts_with("Black")
        || turn_config.starts_with("black");
    BitBoard(bl, wh, turn)
}

fn main() {
    // Main intro
    println!("Evaluation by custom ai");
    let board = read_board();
    println!("{}", bit_board::show_bit_board(board));
    let BitBoard(bl, wh, turn) = board;
    let valid_moves = bit_board::valid_moves_set(bl, wh);
    println!("{}", bit_board::show_bit_board(BitBoard(valid_moves, wh, false)));
    // custom_ai::find_best_move_custom(&turn).unwrap();
}
