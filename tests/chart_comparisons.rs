mod common;

use common::create_notes;
use itgrate::note::Pos;
use itgrate::rate::rate_notes;

#[test]
#[ignore = "currently failing?"]
fn two_measures_of_240_is_harder_than_eight_measures_of_120() {
    let notes_240 = create_notes(32, |_| Pos::default(), |f| f / 16.);
    let notes_120 = create_notes(128, |_| Pos::default(), |f| f / 8.);
    let rating_240 = rate_notes(&notes_240);
    let rating_120 = rate_notes(&notes_120);
    assert!(rating_240 > rating_120, "{} {}", rating_240, rating_120);
}
