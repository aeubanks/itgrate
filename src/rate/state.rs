use crate::note::Note;

#[derive(Debug, PartialEq, Copy, Clone, Default)]
pub struct FootStatus {
    last_hit: Option<Note>,
    fatigue: f32,
}

#[derive(Copy, Clone)]
pub enum Foot {
    Left = 0,
    Right = 1,
}

#[derive(Debug, PartialEq, Copy, Clone, Default)]
pub struct State {
    feet: [FootStatus; 2],
    fatigue: f32,
    max_fatigue: f32,
}

#[cfg(test)]
impl State {
    pub fn with_fatigue(f: f32) -> Self {
        let mut ret = Self::default();
        ret.fatigue = f;
        ret
    }
}

impl State {
    const FATIGUE_PER_STEP: f32 = 1.;
    const FATIGUE_DIST_RATIO: f32 = 0.01;
    const FATIGUE_DECAY_RATE: f32 = 0.1;
    const FOOT_TO_BODY_FATIGUE_RATIO: f32 = 0.1;

    fn fatigue_after_rest_and_step(prev_fatigue: f32, rest_time: f32, distance: f32) -> f32 {
        // Recover from fatigue exponentially, and add a constant fatigue per step.
        prev_fatigue * (-rest_time * State::FATIGUE_DECAY_RATE).exp()
            + distance / (rest_time + 1.) * State::FATIGUE_DIST_RATIO
            + State::FATIGUE_PER_STEP
    }

    pub fn step(&self, foot: Foot, note: Note) -> State {
        for foot in &self.feet {
            if let Some(last) = foot.last_hit {
                assert!(last.time <= note.time, "stepping an earlier note!");
            }
        }
        let mut copy = *self;

        let mut foot = &mut copy.feet[foot as usize];
        foot.fatigue = State::fatigue_after_rest_and_step(
            foot.fatigue,
            foot.last_hit.map_or(0., |lh| note.time - lh.time),
            foot.last_hit.map_or(0., |lh| note.pos.distance(&lh.pos)),
        );
        foot.last_hit = Some(note);
        // Add the foot's fatigue (scaled by some factor) to overall fatigue.
        copy.fatigue += foot.fatigue * State::FOOT_TO_BODY_FATIGUE_RATIO;
        copy.max_fatigue = self.max_fatigue.max(copy.fatigue);
        copy
    }

    pub fn max_fatigue(&self) -> f32 {
        self.max_fatigue
    }

    pub fn fatigue(&self) -> f32 {
        self.fatigue
    }
}

#[test]
fn test_state_step() {
    use crate::note::Pos;
    {
        let note1 = Note {
            pos: Pos { x: 0., y: 0. },
            time: 0.0,
        };
        let note2 = Note {
            pos: Pos { x: 0., y: 0. },
            time: 0.1,
        };
        let mut s = State::default();
        s = s.step(Foot::Left, note1);
        assert_ne!(s.feet[Foot::Left as usize].fatigue, 0.);
        assert_eq!(s.feet[Foot::Left as usize].last_hit, Some(note1));
        assert_eq!(s.feet[Foot::Right as usize].fatigue, 0.);
        assert_eq!(s.feet[Foot::Right as usize].last_hit, None);
        assert_eq!(s.max_fatigue, s.fatigue);

        s = s.step(Foot::Right, note2);
        assert_ne!(s.feet[Foot::Left as usize].fatigue, 0.);
        assert_eq!(s.feet[Foot::Left as usize].last_hit, Some(note1));
        assert_ne!(s.feet[Foot::Right as usize].fatigue, 0.);
        assert_eq!(s.feet[Foot::Right as usize].last_hit, Some(note2));
    }
}
