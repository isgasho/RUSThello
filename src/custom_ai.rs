use {Result};
use rand::thread_rng;
use rand::distributions::{IndependentSample, Range};
use ai_player::{AiPlayer, Score};
use reversi::{board, turn, Side, ReversiError};
use reversi::board::Coord;

const RANDOMNESS: f64 = 0.05f64;

const ENDGAME_LENGTH: usize = 17;

fn coord_to_string(c: Coord) -> String {
    let (r, c) = c.get_row_col();
    format!("{}{}", char::from((0x61 + c) as u8), r + 1)
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
    // If everything is alright, turn shouldn't be ended
    let side = turn.get_state()
        .ok_or_else(|| ReversiError::EndedGame(*turn))?;
    
    // Finds all possible legal moves and records their coordinates
    let moves: Vec<Coord> = all_possible_moves(turn);
    
    match moves.len() {
        0 => unreachable!("Game is not ended!"), // Game can't be ended
        // 1 => Ok(moves[0]), // If there is only one possible move, there's no point in evaluating it.
        _num_moves => {
            let tempo = turn.get_tempo();
            let left = (64 - tempo) as usize;
            let mut moves_and_scores = Vec::new();
            if left > ENDGAME_LENGTH {
                // use iterative deepening
                let mut depth = 1;
                while depth <= 5 {
                    ai_eval_with_depth(turn, depth, &moves,
                                       &mut moves_and_scores, side);
                    depth += 1;
                }
            } else {
                ai_eval_till_end(turn, &moves,
                                   &mut moves_and_scores, side);
            }
            let best_move_and_score = match side {
                Side::Dark =>
                    moves_and_scores.into_iter().min_by_key(|&(_, score)| score),
                Side::Light =>
                    moves_and_scores.into_iter().max_by_key(|&(_, score)| score),
            }
            .expect("No best move found!");
            Ok(best_move_and_score.0)
        }
    }
}

fn ai_eval_with_depth(turn: &turn::Turn, depth: usize, moves: &[Coord],
                      moves_and_scores: &mut Vec<(Coord, Score)>, side: Side) {
    let mut moves_scores_lines = Vec::new();
    moves_and_scores.clear();
    for &mv in moves.iter() {
        let mut turn_after_move = *turn;
        turn_after_move
            .make_move(mv)
            .expect("The move was checked, but something went wrong!");
        let (score, line) =
            ai_eval_iddfs(&turn_after_move, depth)
            .expect("Something went wrong with `ai_eval_iddfs`!");
        moves_scores_lines.push((mv, score, line));
    }
    moves_scores_lines.sort_by_key(|&(_, score, _)| score);
    if side == Side::Light {
        moves_scores_lines.reverse();
    }
    eprintln!("evals[depth = {}]:", depth);
    for i in 0 .. ::std::cmp::min(4, moves_scores_lines.len()) {
        let (mv, score, line) = moves_scores_lines[i].clone();
        eprintln!("{:?}: {}{}", score, coord_to_string(mv),
                  line_to_string(&line));
    }
    *moves_and_scores = moves_scores_lines.into_iter()
        .map(|(mv, score, _)| (mv, score)).collect();
}

fn ai_eval_iddfs(turn: &turn::Turn, depth: usize)
                 -> Result<(Score, Vec<Coord>)> {
    let (mut score, mut line) = try!(ai_eval_iddfs_internal(turn, depth));
    // Add some randomness
    let between = Range::new(-RANDOMNESS, RANDOMNESS);
    let mut rng = thread_rng();
    score = match score {
        Score::Running(val) => Score::Running(val * (1.0 + between.ind_sample(&mut rng))),
        _ => score,
    };
    // Done, return
    line.reverse(); // the last move is pushed first
    Ok((score, line))
}

fn ai_eval_iddfs_internal(turn: &turn::Turn, depth: usize)
                          -> Result<(Score, Vec<Coord>)> {
    match turn.get_state() {
        None => return Ok((Score::Ended(turn.get_score_diff()), Vec::new())),
        Some(_) if depth == 0 =>
            return Ok(
                (Score::Running(try!(my_board_eval(&turn))), Vec::new())
            ),
        _ => (),
    }

    // Finds all possible legal moves and records their coordinates
    let mut moves: Vec<Coord>;
    {
        moves = all_possible_moves(turn);
    }
    
    // If everything is alright, turn shouldn't be ended
    // assert!(!turn.is_endgame());
    
    let mut scores: Vec<(Vec<Coord>, Score)> = Vec::new();
    
    while let Some(coord) = moves.pop() {
        let mut turn_after_move = *turn;
        turn_after_move.make_move(coord)?;
        scores.push(match turn_after_move.get_state() {
            _ => {
                let (new_score, mut line) = try!(ai_eval_iddfs_internal(&turn_after_move, depth - 1));
                line.push(coord);
                (line, new_score)
            }
        });
    }
    
    let (line, score) = match turn.get_state() {
        Some(Side::Dark) => scores.into_iter().min_by_key(|&(_, score)| score).expect("Why should this fail?"),
        Some(Side::Light) => scores.into_iter().max_by_key(|&(_, score)| score).expect("Why should this fail?"),
        None => unreachable!("turn is ended but it should not be"),
    };
    Ok((score, line))
}

fn ai_eval_till_end(turn: &turn::Turn, moves: &[Coord],
                      moves_and_scores: &mut Vec<(Coord, Score)>, side: Side) {
    let mut moves_scores_lines = Vec::new();
    moves_and_scores.clear();
    for &mv in moves.iter() {
        let mut turn_after_move = *turn;
        turn_after_move
            .make_move(mv)
            .expect("The move was checked, but something went wrong!");
        let (score, line) =
            ai_eval_till_end_internal(&turn_after_move)
            .expect("Something went wrong with `ai_eval_till_end_internal`!");
        moves_scores_lines.push((mv, score, line));
        let prunable =
            match turn.get_state() {
                Some(Side::Dark) =>
                    score < 0,
                Some(Side::Light) =>
                    score > 0,
                None => unreachable!(),
            };
        if prunable {
            break;
        }
    }
    moves_scores_lines.sort_by_key(|&(_, score, _)| score);
    if side == Side::Light {
        moves_scores_lines.reverse();
    }
    eprintln!("evals[depth = {} (full)]:", 63 - turn.get_tempo());
    for i in 0 .. ::std::cmp::min(4, moves_scores_lines.len()) {
        let (mv, score, line) = moves_scores_lines[i].clone();
        eprintln!("{:?}: {}{}", score, coord_to_string(mv),
                  line_to_string(&line));
    }
    *moves_and_scores = moves_scores_lines.into_iter()
        .map(|(mv, score, _)| (mv, Score::Ended(score))).collect();
}



// Check only if it's winning or not
fn ai_eval_till_end_internal(turn: &turn::Turn)
                    -> Result<(i16, Vec<Coord>)> {
    match turn.get_state() {
        None => return Ok((turn.get_score_diff(), Vec::new())),
        _ => (),
    }

    // Finds all possible legal moves and records their coordinates
    let mut moves: Vec<Coord>;
    {
        moves = all_possible_moves(turn);
    }
    
    // If everything is alright, turn shouldn't be ended
    // assert!(!turn.is_endgame());
    
    let mut scores: Vec<(Vec<Coord>, i16)> = Vec::new();
    
    while let Some(coord) = moves.pop() {
        let mut turn_after_move = *turn;
        turn_after_move.make_move(coord)?;
        let (newline, new_score) = match turn_after_move.get_state() {
            _ => {
                let (new_score, mut line) = try!(ai_eval_till_end_internal(&turn_after_move));
                line.push(coord);
                (line, new_score)
            }
        };
        scores.push((newline, new_score));
        let prunable =
            match turn.get_state() {
                Some(Side::Dark) =>
                    new_score < 0,
                Some(Side::Light) => new_score > 0,
                None => unreachable!(),
            };
        if prunable {
            break;
        }
    }
    
    let (line, score) = match turn.get_state() {
        Some(Side::Dark) => scores.into_iter().min_by_key(|&(_, score)| score).expect("Why should this fail?"),
        Some(Side::Light) => scores.into_iter().max_by_key(|&(_, score)| score).expect("Why should this fail?"),
        None => unreachable!("turn is ended but it should not be"),
    };
    Ok((score, line))
}

fn my_board_eval(turn: &turn::Turn) -> Result<f64> {
    assert!(!turn.is_end_state());
    let side = turn.get_state().unwrap();

    let turns = turn.get_tempo();
    let mut val = AiPlayer::heavy_eval(turn)?;
    if turns <= 35 {
        val = 0.0;
        let mylegit = all_possible_moves(turn).len();
        if side == Side::Light {
            val += mylegit as f64 / 2.0;
        } else {
            val -= mylegit as f64 / 2.0;
        }
        let edges = [(0, 0, 1, 0), (7, 0, 0, 1), (0, 0, 0, 1), (0, 7, 1, 0)];
        for &(sx, sy, dx, dy) in edges.iter() {
            let mut white = 0;
            let mut black = 0;
            for i in 0 .. 8 {
                let x = sx + i * dx;
                let y = sy + i * dy;
                match *(turn.get_cell(Coord::new(x, y))?) {
                    None => (),
                    Some(disk) => match disk.get_side() {
                        Side::Light => white |= 1 << i,
                        Side::Dark => black |= 1 << i,
                    },
                };
            }
            val += eval_edge(white) - eval_edge(black);
        }
    }
    Ok(val)
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


fn all_possible_moves(turn: &turn::Turn) -> Vec<Coord> {
    let mut moves: Vec<Coord> = Vec::new();
    for row in 0..board::BOARD_SIZE {
        for col in 0..board::BOARD_SIZE {
            let coord = board::Coord::new(row, col);
            if turn.check_move(coord).is_ok() {
                moves.push(coord);
            }
        }
    }
    moves
}
