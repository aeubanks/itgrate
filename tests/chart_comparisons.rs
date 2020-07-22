mod common;

use common::create_notes;
use itgrate::note::Pos;
use itgrate::rate::{rate_notes, state::StepParams};

#[test]
#[ignore = "currently failing?"]
fn two_measures_of_240_is_harder_than_eight_measures_of_120() {
    let notes_240 = create_notes(32, |_| Pos::default(), |f| f / 16.);
    let notes_120 = create_notes(128, |_| Pos::default(), |f| f / 8.);
    let step_params = StepParams::default();
    let rating_240 = rate_notes(&notes_240, &step_params);
    let rating_120 = rate_notes(&notes_120, &step_params);
    assert!(rating_240 > rating_120, "{} {}", rating_240, rating_120);
}
