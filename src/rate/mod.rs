use crate::note::Note;
use state::{Foot, State};

mod state;

pub fn rate_notes(notes: &[Note]) -> f32 {
    let mut state = State::new();
    let mut foot = Foot::Left;
    for n in notes {
        state = state.step(foot, *n);
        foot = foot.other_foot();
    }
    state.fatigue()
}
