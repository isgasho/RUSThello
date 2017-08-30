/**
 * The implementation is done in reference to https://github.com/koba-e964/othello-ai/blob/master/CBoard.hs, which uses routines that are originally in edax.
 */

// is_dark_turn: bool
#[derive(Clone, Copy)]
pub struct BitBoard(pub u64, pub u64, pub bool);

pub fn get_score_diff(light: u64, dark: u64) -> i16 {
    light.count_ones() as i16 - dark.count_ones() as i16
}

pub fn get_tempo(light: u64, dark: u64) -> i16 {
    light.count_ones() as i16 + dark.count_ones() as i16
}

/// Port from https://github.com/koba-e964/othello-ai/blob/master/CBoard.hs
/// set of valid moves represented by Places
/// reference : http://code.google.com/p/edax-reversi/source/browse/src/board.c
pub fn valid_moves_set(bl: u64, wh: u64) -> u64 {
    let mask = wh & 0x7e7e7e7e7e7e7e7e;
    let r1 = vmSubMO(bl, mask, 1);
    let r2 = vmSubMO(bl, wh, 8);
    let r3 = vmSubMO(bl, mask, 7);
    let r4 = vmSubMO(bl, mask, 9);
    (r1 | r2 | r3 | r4) & !(bl | wh)
}


fn vmSubMO(my: u64, mask: u64, dir: usize) -> u64 {
    let dir2 = dir + dir;
    let fl1 = mask & (my << dir);
    let fr1 = mask & (my >> dir);
    let fl2 = fl1 | (mask & (fl1 << dir));
    let fr2 = fr1 | (mask & (fr1 >> dir));
    let maskl = mask & (mask << dir);
    let maskr = mask & (mask >> dir);
    let fl3 = fl2 | maskl & (fl2 << dir2);
    let fr3 = fr2 | maskr & (fr2 >> dir2);
    let fl4 = fl3 | maskl & (fl3 << dir2);
    let fr4 = fr3 | maskr & (fr3 >> dir2);
    fl4 << dir | fr4 >> dir
}

#[derive(Clone, Copy)]
enum Dir {
    Left,
    Right,
}

#[derive(Clone, Copy)]
struct Transfer(Dir, u64, usize);


const transfers: [Transfer; 8] =
    [Transfer(Dir::Right, 0xffffffffffffff00, 8), // up
     Transfer(Dir::Right, 0xfefefefefefefe00, 9), // up left
     Transfer(Dir::Right, 0x7f7f7f7f7f7f7f00, 7), // up right
     Transfer(Dir::Right, 0xfefefefefefefefe, 1), // left
     Transfer(Dir::Left, 0x7f7f7f7f7f7f7f7f, 1), // right
     Transfer(Dir::Left, 0x00fefefefefefefe, 7), // down left
     Transfer(Dir::Left, 0x007f7f7f7f7f7f7f, 9), // down right
     Transfer(Dir::Left, 0x00ffffffffffffff, 8)]; // down


fn trans_op(trans: Transfer, x: u64) -> u64 {
    let Transfer(dir, mask, sh) = trans;
    match dir {
        Dir::Left =>
            (x & mask) << sh,
        Dir::Right =>
            (x & mask) >> sh,
    }
}

fn reversibleSetInDir(trans: Transfer, my: u64, opp: u64) -> u64 {
    let vacant = !(my | opp);
    let mut opps: [u64; 6] = [0; 6];
    let mut cur = trans_op(trans, opp);
    for i in 0 .. 6 {
        opps[i] = cur;
        cur = trans_op(trans, cur);
    }
    let mut mys: [u64; 6] = [0; 6];
    cur = trans_op(trans, trans_op(trans, my));
    for i in 0 .. 6 {
        mys[i] = cur;
        cur = trans_op(trans, cur);
    }
    cur = 0;
    for i in 0 .. 6 {
        cur |= opps[i] & mys[i];
    }
    vacant & cur
}

/// disk must be a singleton
fn flippableIndiceSet(my: u64, opp: u64, dist: u64) -> u64 {
    let mut cur = 0;
    for &trans in transfers.iter() {
        cur |= flippableIndicesInDir(trans, my, opp, dist);
    }
    cur
}

/// reference: http://ja.wikipedia.org/wiki/%E3%82%AA%E3%82%BB%E3%83%AD%E3%81%AB%E3%81%8A%E3%81%91%E3%82%8B%E3%83%93%E3%83%83%E3%83%88%E3%83%9C%E3%83%BC%E3%83%89
fn flippableIndicesInDir(trans: Transfer, my: u64, opp: u64, disk: u64)
    -> u64 {
    let ma = trans_op(trans, disk);
    let mut rev = 0;
    let mut mask = ma;
    while mask != 0 {
        if (mask & opp) == 0 {
            break;
        }
        rev |= mask;
        mask = trans_op(trans, mask);
    }
    if (mask & my) != 0 {
        rev
    } else {
        0
    }
}
    
const BLACK: usize = 0;
const WHITE: usize = 1;

fn countC(cb: BitBoard, color: usize) -> i16 {
    let BitBoard(bl, wh, turn) = cb;
    (if color == BLACK {
        bl.count_ones()
    } else {
        wh.count_ones()
    }) as i16
}    


pub fn show_bit_board(board: BitBoard) -> String {
    let BitBoard(bl, wh, turn) = board;
    let mut ret = 
    (" |A B C D E F G H \n-+----------------\n").to_string();
    for row in 1 .. 9 {
        let mut board_line = format!("{}|", row);
        for col in 1 .. 9 {
            let pos = (row - 1) * 8 + col - 1; // row first
            let mask = 1u64 << pos;
            let c = if (bl & mask) != 0 {
                'X'
            } else if (wh & mask) != 0 {
                'O'
            } else {
                ' '
            };
            board_line = format!("{}{} ", board_line, c);
        }
        ret = format!("{}{}\n", ret, board_line);
    }
    ret = format!("{}  (X: Black,  O: White)\n", ret);
    ret = format!("{} {} to move", ret, if turn { "Black" } else { "White" });
    ret
}
