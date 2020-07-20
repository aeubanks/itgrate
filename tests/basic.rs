mod common;

use common::create_notes;
use itgrate::note::{Note, Pos};
use itgrate::rate::{rate_notes, ratings_at_notes};

#[test]
fn empty() {
    assert_eq!(rate_notes(&[]), 0.);
}

#[test]
fn faster_is_harder() {
    // [rate_notes(50 notes 1s apart), rate_notes(50 notes 0.5s apart), ...]
    let ratings = (1..10)
        .map(|i| {
            create_notes(
                50,
                |f| Pos {
                    x: (f / 4.0).fract(),
                    y: (f / 5.0).fract(),
                },
                |n| n / i as f32,
            )
        })
        .map(|notes| rate_notes(&notes))
        .collect::<Vec<f32>>();
    for w in ratings.windows(2) {
        // Ratings should increase with faster notes
        assert!(w[0] < w[1], "{} {}", w[0], w[1]);
    }
}

#[test]
fn more_is_harder() {
    // [rate_notes(0 notes), rate_notes(1 note), ...]
    let ratings = (1..10)
        .map(|i| {
            create_notes(
                i,
                |f| Pos {
                    x: (f / 4.0).fract(),
                    y: (f / 5.0).fract(),
                },
                |n| n,
            )
        })
        .map(|notes| rate_notes(&notes))
        .collect::<Vec<f32>>();
    for w in ratings.windows(2) {
        // Ratings should increase with more notes
        assert!(w[0] < w[1], "{} {}", w[0], w[1]);
    }
}

#[test]
fn farther_is_harder() {
    let notes1 = (0..100)
        .flat_map(|n| {
            vec![
                Note {
                    pos: Pos { x: 0., y: 0. },
                    time: n as f32,
                },
                Note {
                    pos: Pos { x: 1., y: 0. },
                    time: n as f32,
                },
                Note {
                    pos: Pos { x: 0., y: 1. },
                    time: n as f32,
                },
                Note {
                    pos: Pos { x: 1., y: 1. },
                    time: n as f32,
                },
            ]
        })
        .map(|n| n)
        .collect::<Vec<Note>>();
    let notes2 = (0..100)
        .flat_map(|n| {
            vec![
                Note {
                    pos: Pos { x: 0., y: 0. },
                    time: n as f32,
                },
                Note {
                    pos: Pos { x: 1., y: 0. },
                    time: n as f32,
                },
                Note {
                    pos: Pos { x: 0., y: 2. },
                    time: n as f32,
                },
                Note {
                    pos: Pos { x: 1., y: 2. },
                    time: n as f32,
                },
            ]
        })
        .map(|n| n)
        .collect::<Vec<Note>>();

    let r1 = rate_notes(&notes1);
    let r2 = rate_notes(&notes2);
    assert!(r1 < r2);
}

#[test]
fn one_note_after_break_doesnt_affect_difficulty() {
    let mut notes = create_notes(100, |_| Pos::default(), |f| f / 10.);
    let r1 = rate_notes(&notes);
    notes.push(Note {
        pos: Pos::default(),
        time: 100.,
    });
    let r2 = rate_notes(&notes);
    assert_eq!(r1, r2);
}

#[test]
fn rating_at_notes_same_len_as_notes() {
    let notes = create_notes(100, |_| Pos::default(), |f| f / 10.);
    let ratings = ratings_at_notes(&notes);
    assert_eq!(notes.len(), ratings.len());
}

#[test]
fn fatigue_keeps_increasing_in_unbroken_stream() {
    let notes = create_notes(100, |_| Pos::default(), |f| f / 10.);
    let ratings = ratings_at_notes(&notes);
    for w in ratings.windows(2) {
        assert!(w[0] < w[1], "{} {}", w[0], w[1]);
    }
}
