use {Result};
use rand::thread_rng;
use rand::distributions::{IndependentSample, Range};
use ai_player::{AiPlayer, Score};
use reversi::{board, turn, Side, ReversiError};
use reversi::board::Coord;

const RANDOMNESS: f64 = 0.05f64;

fn coord_to_string(c: Coord) -> String {
    let (r, c) = c.get_row_col();
    format!("{}{}", char::from((0x61 + c) as u8), r + 1)
}
pub fn find_best_move_custom(turn: &turn::Turn) -> Result<board::Coord> {
    // stupid AI who always attempts to move a1
    // If everything is alright, turn shouldn't be ended
    let side = turn.get_state()
        .ok_or_else(|| ReversiError::EndedGame(*turn))?;
    
    // Finds all possible legal moves and records their coordinates
    let mut moves: Vec<Coord> = Vec::new();
    for row in 0..board::BOARD_SIZE {
        for col in 0..board::BOARD_SIZE {
            let coord = board::Coord::new(row, col);
            if turn.check_move(coord).is_ok() {
                moves.push(coord);
                }
        }
    }
    
    match moves.len() {
        0 => unreachable!("Game is not ended!"), // Game can't be ended
        1 => Ok(moves[0]), // If there is only one possible move, there's no point in evaluating it.
        _num_moves => {
            // use iterative deepening
            let mut depth = 1;
            let mut moves_and_scores = Vec::new();
            while depth <= 5 {
                moves_and_scores.clear();
                for &mv in moves.iter() {
                    let mut turn_after_move = *turn;
                    turn_after_move
                        .make_move(mv)
                        .expect("The move was checked, but something went wrong!");
                    let score =
                        ai_eval_iddfs(&turn_after_move, depth)
                        .expect("Something went wrong with `ai_eval_iddfs`!");
                    moves_and_scores.push((mv, score));
                }
                moves_and_scores.sort_by_key(|&(_, score)| score);
                if side == Side::Light {
                    moves_and_scores.reverse();
                }
                eprintln!("evals[depth = {}]:", depth);
                for i in 0 .. ::std::cmp::min(4, moves_and_scores.len()) {
                    let (mv, score) = moves_and_scores[i];
                    eprintln!("{:?}: {}", score, coord_to_string(mv));
                }
                depth += 1;
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
fn ai_eval_iddfs(turn: &turn::Turn, depth: u32) -> Result<Score> {
    if turn.get_state().is_none() {
        Ok(Score::Ended(turn.get_score_diff()))
    } else {
        let mut score = try!(ai_eval_with_leftover_iddfs(turn, depth));
        // Add some randomness
        let between = Range::new(-RANDOMNESS, RANDOMNESS);
        let mut rng = thread_rng();
        score = match score {
            Score::Running(val) => Score::Running(val * (1.0 + between.ind_sample(&mut rng))),
            _ => score,
        };
        // Done, return
        Ok(score)
    }
}

fn ai_eval_with_leftover_iddfs(turn: &turn::Turn, depth: u32) -> Result<Score> {
    
    // If everything is alright, turn shouldn't be ended
    // assert!(!this_turn.is_endgame());
    
    match turn.get_state() {
        None => return Ok(Score::Ended(turn.get_score_diff())),
        Some(_) if depth == 0 =>
            return Ok(Score::Running(try!(AiPlayer::heavy_eval(&turn)))),
        _ => (),
    }
    
    
    // Finds all possible legal moves and records their coordinates
    let mut moves: Vec<Coord>;
    {
        let mut turn = *turn;
        moves = Vec::new();
        for row in 0..board::BOARD_SIZE {
            for col in 0..board::BOARD_SIZE {
                let coord = board::Coord::new(row, col);
                if turn.check_move(coord).is_ok() {
                    moves.push(coord);
                }
            }
        }
        match moves.len() {
            0 => unreachable!("Endgame should have been detected earlier: here it's a waste of computations!"),
            1 => {
                turn.make_move(moves[0])?; //.expect("There is one move and it should be legit");
                if turn.get_state().is_none() {
                    return Ok(Score::Ended(turn.get_score_diff()));
                }
            }
            _num_moves => (),
        }
    }
    
    // If everything is alright, turn shouldn't be ended
    // assert!(!turn.is_endgame());
    
    let mut scores: Vec<Score> = Vec::new();
    
    while let Some(coord) = moves.pop() {
        let mut turn_after_move = *turn;
        turn_after_move.make_move(coord)?;
        scores.push(match turn_after_move.get_state() {
            _ => {
                let new_score = try!(ai_eval_with_leftover_iddfs(&turn_after_move, depth - 1));
                new_score
            }
        });
    }
    
    Ok(match turn.get_state() {
        Some(Side::Dark) => scores.into_iter().min().expect("Why should this fail?"),
        Some(Side::Light) => scores.into_iter().max().expect("Why should this fail?"),
        None => unreachable!("turn is ended but it should not be"),
    }
    )
}
