use {Result};
use rand::thread_rng;
use rand::distributions::{IndependentSample, Range};
use ai_player::{Score};

use reversi::{board, turn, Side, ReversiError};
use reversi::board::{Coord};
use bit_board;
use bit_board::BitBoard;

use std::cmp::max;

use smallvec::SmallVec;

type SVec<T> = SmallVec<[T; 16]>;

const RANDOMNESS: f64 = 0.05f64;

const USUAL_DEPTH: usize = 5;
const ENDGAME_LENGTH: usize = 17;

fn coord_to_string(c: Coord) -> String {
    let (r, c) = c.get_row_col();
    if r >= 8 || c >= 8 {
        "Pass".to_string()
    } else {
        format!("{}{}", char::from((0x61 + c) as u8), r + 1)
    }
}

fn line_to_string(line: &[Coord]) -> String {
    if line.len() == 0 {
        "".to_string()
    } else {
        let mut ret = " -".to_string();
        for &mv in line.iter() {
            ret = format!("{} {}", ret, coord_to_string(mv));
        }
        ret
    }
}

pub fn find_best_move_custom(turn: &turn::Turn) -> Result<board::Coord> {
    let mut bl = 0;
    let mut wh = 0;
    for row in 0 .. 8 {
        for col in 0 .. 8 {
            let idx = row * 8 + col;
            let cell = turn.get_cell(Coord::new(row, col))?;
            match *cell {
                Some(disk) => match disk.get_side() {
                    Side::Dark => bl |= 1 << idx,
                    Side::Light => wh |= 1 << idx,
                },
                None => (),
            }
        }
    }
    let is_black = turn.get_state() == Some(Side::Dark);
    match find_best_move_bit_board(BitBoard(bl, wh, is_black)) {
        Some(v) => Ok(v),
        None => Err(ReversiError::EndedGame(*turn)),
    }
}
pub fn find_best_move_bit_board(BitBoard(bl, wh, turn): BitBoard)
                                           -> Option<board::Coord> {
    // Finds all possible legal moves and records their coordinates
    let my = if turn { bl } else { wh };
    let opp = if turn { wh } else { bl };
    let moves = bit_board::valid_moves_set(my, opp);
    let opp_moves = bit_board::valid_moves_set(opp, my);
    if moves == 0 && opp_moves == 0 {
        // Game has ended.
        return None;
    }
    
    match moves.count_ones() {
        0 => return None,
        _num_moves => {
            let tempo = bit_board::get_tempo(my, opp);
            let left = (64 - tempo) as usize;
            let mut moves_and_scores = Vec::new();
            if left > ENDGAME_LENGTH {
                // use iterative deepening
                let mut depth = 1;
                while depth <= USUAL_DEPTH {
                    ai_eval_with_depth(my, opp, depth, moves,
                                       &mut moves_and_scores);
                    depth += 1;
                }
            } else {
                let mut nnodes = 0;
                ai_eval_till_end(my, opp, moves,
                                 &mut moves_and_scores, true, &mut nnodes);
            }
            let best_move_and_score =
                moves_and_scores.into_iter().min_by_key(|&(_, score)| score)
                .expect("No best move found!");
            Some(best_move_and_score.0)
        }
    }
}

pub fn ai_eval_with_depth(my: u64, opp: u64, depth: usize, moves: u64,
                      moves_and_scores: &mut Vec<(Coord, Score)>) {
    let mut moves_scores_lines = SVec::new();
    moves_and_scores.clear();
    let mut restmoves = moves;
    while restmoves != 0 {
        let disk = 1u64 << restmoves.trailing_zeros();
        let (nmy, nopp) = bit_board::move_bit_board(my, opp, disk);
        restmoves ^= disk;
        let (score, line) =
            ai_eval_iddfs(nopp, nmy, depth);
        moves_scores_lines.push((disk_to_coord(disk), score, line));
    }
    moves_scores_lines.sort_unstable_by_key(|&(_, score, _)| score);
    eprintln!("evals[depth = {}]:", depth);
    for i in 0 .. ::std::cmp::min(4, moves_scores_lines.len()) {
        let (mv, score, line) = moves_scores_lines[i].clone();
        eprintln!("{:?}: {}{}", negate_score(score), coord_to_string(mv),
                  line_to_string(&line));
    }
    *moves_and_scores = moves_scores_lines.into_iter()
        .map(|(mv, score, _)| (mv, score)).collect();
}

fn ai_eval_iddfs(my: u64, opp: u64, depth: usize)
                 -> (Score, SVec<Coord>) {
    let (mut score, mut line) = ai_eval_iddfs_internal(my, opp, depth);
    // Add some randomness
    let between = Range::new(-RANDOMNESS, RANDOMNESS);
    let mut rng = thread_rng();
    score = match score {
        Score::Running(val) => Score::Running(val * (1.0 + between.ind_sample(&mut rng))),
        _ => score,
    };
    // Done, return
    line.reverse(); // the last move is pushed first
    (score, line)
}

fn ai_eval_iddfs_internal(my: u64, opp: u64, depth: usize)
                          -> (Score, SVec<Coord>) {
    let mut moves = bit_board::valid_moves_set(my, opp);
    let oppmoves = bit_board::valid_moves_set(opp, my);
    if moves == 0 && oppmoves == 0 {
        return (Score::Ended(get_score_diff(my, opp)), SVec::new());
    }
    if depth == 0 {
        return
            (Score::Running(my_board_eval(my, opp)), SVec::new());
    }

    if moves == 0 {
        let (score, mut line) = ai_eval_iddfs_internal(opp, my, depth);
        line.push(Coord::new(8, 8)); // Pass
        return (negate_score(score), line);
    }
    // If everything is alright, turn shouldn't be ended
    // assert!(!turn.is_endgame());
    
    let mut scores: SVec<(SVec<Coord>, Score)> = SVec::new();
    
    while moves != 0 {
        let disk = 1u64 << moves.trailing_zeros();
        moves ^= disk;
        let (nmy, nopp) = bit_board::move_bit_board(my, opp, disk);
        let (new_score, mut line)
            = ai_eval_iddfs_internal(nopp, nmy, depth - 1);
        line.push(disk_to_coord(disk));
        scores.push((line, new_score));
    }
    
    let (line, score) = scores.into_iter().min_by_key(|&(_, score)| score).expect("Why should this fail?");
    (negate_score(score), line)
}

pub fn ai_eval_till_end(my: u64, opp: u64, moves: u64,
                        moves_and_scores: &mut Vec<(Coord, Score)>,
                        pruning: bool,
                        nnodes: &mut u64) {
    let mut moves_scores_lines = SVec::new();
    moves_and_scores.clear();
    let mut moves = moves;
    let mut disks = SVec::new();
    while moves != 0 {
        let disk = 1u64 << moves.trailing_zeros();
        moves ^= disk;
        let (nmy, nopp) = bit_board::move_bit_board(my, opp, disk);
        let opp_moves = bit_board::valid_moves_set(nopp, nmy);
        disks.push((opp_moves.count_ones(), disk));
    }
    disks.sort_unstable();
    let mut ma = -1i16 << 10;
    for (_, disk) in disks {
        let mut nnodes_this = 0;
        let (nmy, nopp) = bit_board::move_bit_board(my, opp, disk);
        let (score, mut line, defunct) =
            ai_eval_till_end_internal(nopp, nmy, -1 << 10, -ma, pruning,
                                      &mut nnodes_this);
        *nnodes += nnodes_this;
        println!("Move: {} #nodes = {}", coord_to_string(disk_to_coord(disk)),
                 nnodes_this);
        ma = max(ma, -score);
        if !defunct {
            line.reverse();
            moves_scores_lines.push((disk_to_coord(disk), score, line));
        }
        if pruning && score < 0 {
            break;
        }
    }
    moves_scores_lines.sort_unstable_by_key(|&(_, score, _)| score);
    eprintln!("evals[depth = {} ({})]:", 63 - bit_board::get_tempo(my, opp),
              if pruning { "lock" } else { "full" });
    for i in 0 .. ::std::cmp::min(4, moves_scores_lines.len()) {
        let (mv, score, line) = moves_scores_lines[i].clone();
        let line: SVec<_> = line.into_iter().map(|x| disk_to_coord(x)).collect();
        eprintln!("{:?}: {}{}", -score, coord_to_string(mv),
                  line_to_string(&line));
    }
    *moves_and_scores = moves_scores_lines.into_iter()
        .map(|(mv, score, _)| (mv, Score::Ended(score))).collect();
}



// Check only if it's winning or not
fn ai_eval_till_end_internal(my: u64, opp: u64, alpha: i16, beta: i16,
                             pruning: bool,
                             nnodes: &mut u64)
                             -> (i16, SVec<u64>, bool) {
    *nnodes += 1;
    let mut moves = bit_board::valid_moves_set(my, opp);
    if moves == 0 && bit_board::valid_moves_set(opp, my) == 0 {
        let score = get_score_diff(my, opp);
        return (score, SVec::new(), false);
    }

    if moves == 0 {
        let (score, mut line, defunct) = ai_eval_till_end_internal(opp, my,
        -beta, -alpha, pruning, nnodes);
        if defunct {
            return (-score, SVec::new(), true);
        }
        line.push(0); // Pass
        return (-score, line, false);
    }

    let mut disks = SVec::new();
    while moves != 0 {
        let disk = 1u64 << moves.trailing_zeros();
        moves ^= disk;
        let (nmy, nopp) = bit_board::move_bit_board(my, opp, disk);
        let opp_moves = bit_board::valid_moves_set(nopp, nmy);
        disks.push((opp_moves.count_ones(), disk));
    }
    disks.sort_unstable();
    let mut ma = alpha;
    let mut line = SVec::new();
    let mut found = false;
    for (_, disk) in disks {
        let (nmy, nopp) = bit_board::move_bit_board(my, opp, disk);
        let (new_score, mut newline, defunct) =
            ai_eval_till_end_internal(nopp, nmy, -beta, -ma, pruning,
                                      nnodes);
        if ma < -new_score {
            ma = -new_score;
            if !defunct {
                newline.push(disk);
                line = newline;
                found = true;
            }
        }
        if ma >= beta {
            return (ma, SVec::new(), true);
        }
        // Opponent always lose, no need to search more in lock mode
        if pruning && new_score < 0 {
            break;
        }
    }
    (ma, line, !found)
}

fn my_board_eval(my: u64, opp: u64) -> f64 {
    let mut val = 0.0;
    let mylegit = bit_board::valid_moves_set(my, opp).count_ones();
    val += mylegit as f64 / 2.0;
    let edges = [(0, 1), (0, 8), (7, 8), (56, 1)];
    for &(s, d) in edges.iter() {
        let mut white = 0;
        let mut black = 0;
        for i in 0 .. 8 {
            let idx = s + i * d;
            if (my & 1u64 << idx) != 0 {
                black |= 1u8 << i;
            }
            if (opp & 1u64 << idx) != 0 {
                white |= 1u8 << i;
            }
        }
        val += eval_edge(black) - eval_edge(white);
    }
    val
}

fn eval_edge(pat: u8) -> f64 {
    match pat & 0x81 {
        0x00 => -(pat.count_ones() as f64) - 2.0 * ((pat & 0x42).count_ones() as f64),
        0x01 => pat.count_ones() as f64 * 4.0,
        0x80 => pat.count_ones() as f64 * 4.0,
        0x81 => pat.count_ones() as f64 * 8.0,
        _ => unreachable!(),
    }
}


fn get_score_diff(my: u64, opp: u64) -> i16 {
    my.count_ones() as i16 - opp.count_ones() as i16
}

fn negate_score(s: Score) -> Score {
    match s {
        Score::Running(f) => Score::Running(-f),
        Score::Ended(d) => Score::Ended(-d),
    }
}

fn disk_to_coord(disk: u64) -> Coord {
    if disk == 0 {
        return Coord::new(8, 8);
    }
    assert_eq!(disk.count_ones(), 1);
    let idx = disk.trailing_zeros();
    Coord::new((idx / 8) as usize, (idx % 8) as usize)
}
