use crate::note::Pos;

pub fn col_to_pos(col: usize, num_cols: usize) -> Pos {
    match num_cols {
        // TODO: add doubles, triples, pump, RH
        4 => ddr_singles_pos(col),
        _ => unreachable!("unexpected number of columns"),
    }
}

fn ddr_singles_pos(col: usize) -> Pos {
    match col {
        0 => Pos { x: 0., y: 1. },
        1 => Pos { x: 1., y: 0. },
        2 => Pos { x: 1., y: 2. },
        3 => Pos { x: 2., y: 1. },
        _ => unreachable!("unexpected col for DDR singles"),
    }
}
