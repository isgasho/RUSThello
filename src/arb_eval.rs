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

use rusthello_lib::{custom_ai, bit_board};
use rusthello_lib::bit_board::BitBoard;
use std::time::{Instant};

fn read_board() -> bit_board::BitBoard {
    let stdin = ::std::io::stdin();
    let mut config: String = "".to_string();
    stdin.read_line(&mut config).unwrap();
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
    stdin.read_line(&mut turn_config).unwrap();
    let turn = turn_config.starts_with("Black")
        || turn_config.starts_with("black");
    BitBoard(bl, wh, turn)
}

fn main() {
    // Main intro
    println!("Evaluation by custom ai");
    let board = read_board();
    let BitBoard(bl, wh, turn) = board;
    let my = if turn { bl } else { wh };
    let opp = if turn { wh } else { bl };
    println!("{}", bit_board::show_bit_board(board));
    let start = Instant::now();
    custom_ai::find_best_move_bit_board(board);
    let end = start.elapsed();
    let end = end.as_secs() as f64 +
        end.subsec_nanos() as f64 * 1e-9;
    println!("Analysis: {}sec", end);
    // full analysis
    if bit_board::get_tempo(my, opp) >= 40 {
        let depth = 63 - bit_board::get_tempo(my, opp) as usize;
        let mut moves_and_scores = Vec::new();
        let moves = bit_board::valid_moves_set(my, opp);
        let start = Instant::now();
        custom_ai::ai_eval_till_end(my, opp, moves, &mut moves_and_scores,
                                    true);
        let end = start.elapsed();
        let end = end.as_secs() as f64 +
            end.subsec_nanos() as f64 * 1e-9;
        println!("Lock analysis: {}sec", end);
        if true {
            let start = Instant::now();
            custom_ai::ai_eval_till_end(my, opp, moves, &mut moves_and_scores,
                                        false);
            let end = start.elapsed();
            let end = end.as_secs() as f64 +
                end.subsec_nanos() as f64 * 1e-9;
            println!("Full analysis: {}sec", end);
        }
    }
    // custom_ai::find_best_move_custom(&turn).unwrap();
}
