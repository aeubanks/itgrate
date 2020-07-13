use crate::note::Pos;

const GAME_TYPES: &[(usize, fn(usize) -> Pos)] = &[(4, itg_pos), (5, pump_pos), (9, rh_pos)];

pub fn col_to_pos(col: usize, num_cols: usize) -> Pos {
    for (cols_per_pad, pad_pos_f) in GAME_TYPES {
        if num_cols % cols_per_pad == 0 {
            let pad = col / cols_per_pad;
            let panel = col % cols_per_pad;
            let panel_pos = pad_pos_f(panel);
            return Pos {
                x: 3. * pad as f32 + panel_pos.x,
                y: panel_pos.y,
            };
        }
    }
    unreachable!("unexpected number of columns");
}

fn itg_pos(col: usize) -> Pos {
    match col {
        0 => Pos { x: 0., y: 1. },
        1 => Pos { x: 1., y: 0. },
        2 => Pos { x: 1., y: 2. },
        3 => Pos { x: 2., y: 1. },
        _ => unreachable!("unexpected col for ITG"),
    }
}

fn pump_pos(col: usize) -> Pos {
    match col {
        0 => Pos { x: 0., y: 0. },
        1 => Pos { x: 0., y: 2. },
        2 => Pos { x: 1., y: 1. },
        3 => Pos { x: 2., y: 2. },
        4 => Pos { x: 2., y: 0. },
        _ => unreachable!("unexpected col for Pump"),
    }
}

fn rh_pos(col: usize) -> Pos {
    match col {
        0 => Pos { x: 0., y: 0. },
        1 => Pos { x: 0., y: 1. },
        2 => Pos { x: 0., y: 2. },
        3 => Pos { x: 1., y: 0. },
        4 => Pos { x: 1., y: 1. },
        5 => Pos { x: 1., y: 2. },
        6 => Pos { x: 2., y: 2. },
        7 => Pos { x: 2., y: 1. },
        8 => Pos { x: 2., y: 0. },
        _ => unreachable!("unexpected col for RH"),
    }
}

#[test]
fn test_col_to_pos() {
    assert_eq!(col_to_pos(0, 4), Pos { x: 0., y: 1. });
    assert_eq!(col_to_pos(7, 8), Pos { x: 5., y: 1. });
    assert_eq!(col_to_pos(9, 10), Pos { x: 5., y: 0. });
    assert_eq!(col_to_pos(10, 18), Pos { x: 3., y: 1. });
}
