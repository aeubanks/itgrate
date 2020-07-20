use itgrate::note::{Note, Pos};

pub fn create_notes<F1: Fn(f32) -> Pos, F2: Fn(f32) -> f32>(
    count: usize,
    pos_fn: F1,
    time_fn: F2,
) -> Vec<Note> {
    (0..count)
        .map(|i| Note {
            pos: pos_fn(i as f32),
            time: time_fn(i as f32),
        })
        .collect::<Vec<Note>>()
}

#[test]
fn test_create_notes() {
    assert_eq!(
        create_notes(
            3,
            |f| Pos {
                x: f * 2.0,
                y: f * 4.0
            },
            |f| f / 2.
        ),
        vec![
            Note {
                pos: Pos { x: 0., y: 0. },
                time: 0.
            },
            Note {
                pos: Pos { x: 2., y: 4. },
                time: 0.5
            },
            Note {
                pos: Pos { x: 4., y: 8. },
                time: 1.
            }
        ]
    );
}
