use crate::note::Note;

#[derive(Debug, PartialEq, Copy, Clone, Default)]
pub struct FootStatus {
    last_hit: Option<Note>,
    last_fatigue: f32,
}

impl FootStatus {
    const FATIGUE_PER_STEP: f32 = 2.;
    const FATIGUE_DIST_RATIO: f32 = 0.1;
    const FATIGUE_DECAY_RATE: f32 = 0.05;

    fn fatigue_after_rest(prev_fatigue: f32, rest_time: f32) -> f32 {
        prev_fatigue * (-rest_time * FootStatus::FATIGUE_DECAY_RATE).exp()
    }

    fn fatigue_after_rest_and_step(last_fatigue: f32, rest_time: f32, distance: f32) -> f32 {
        // Recover from fatigue exponentially.
        // Add fatigue based on rest time and distance.
        FootStatus::fatigue_after_rest(last_fatigue, rest_time)
            + (FootStatus::FATIGUE_PER_STEP + distance * FootStatus::FATIGUE_DIST_RATIO)
                / (1. + rest_time)
    }

    fn step(&mut self, note: &Note) {
        self.last_fatigue = FootStatus::fatigue_after_rest_and_step(
            self.last_fatigue,
            self.last_hit.map_or(0., |lh| note.time - lh.time),
            self.last_hit.map_or(0., |lh| note.pos.distance(&lh.pos)),
        );
        self.last_hit = Some(*note);
    }

    fn fatigue(&self, time: f32) -> f32 {
        self.last_hit.map_or(0., |lh| {
            FootStatus::fatigue_after_rest(self.last_fatigue, time - lh.time)
        })
    }
}

#[derive(Copy, Clone)]
pub enum Foot {
    Left = 0,
    Right = 1,
}

#[derive(Debug, PartialEq, Copy, Clone, Default)]
pub struct State {
    feet: [FootStatus; 2],
    cur_fatigue: f32,
    max_fatigue: f32,
}

#[cfg(test)]
impl State {
    pub fn with_fatigue(f: f32) -> Self {
        let mut ret = Self::default();
        ret.cur_fatigue = f;
        ret
    }
}

impl State {
    pub fn step(&self, foot: Foot, note: &Note) -> State {
        for foot in &self.feet {
            if let Some(last) = foot.last_hit {
                assert!(last.time <= note.time, "stepping an earlier note!");
            }
        }
        let mut copy = *self;
        copy.step_impl(foot, note);
        copy
    }

    fn step_impl(&mut self, foot: Foot, note: &Note) {
        let step_foot = &mut self.feet[foot as usize];
        step_foot.step(note);
        self.cur_fatigue = self.feet.iter().map(|f| f.fatigue(note.time)).sum();
        self.max_fatigue = self.max_fatigue.max(self.fatigue());
    }

    pub fn max_fatigue(&self) -> f32 {
        self.max_fatigue
    }

    pub fn fatigue(&self) -> f32 {
        self.cur_fatigue
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
        let note3 = Note {
            pos: Pos { x: 0., y: 0. },
            time: 1000.,
        };
        let mut s = State::default();
        s = s.step(Foot::Left, &note1);
        assert_ne!(s.feet[Foot::Left as usize].last_fatigue, 0.);
        assert_eq!(s.feet[Foot::Left as usize].last_hit, Some(note1));
        assert_eq!(s.feet[Foot::Right as usize].last_fatigue, 0.);
        assert_eq!(s.feet[Foot::Right as usize].last_hit, None);
        assert_eq!(s.max_fatigue(), s.fatigue());
        assert_eq!(
            s.max_fatigue(),
            s.feet[0].fatigue(note1.time) + s.feet[1].fatigue(note1.time)
        );

        s = s.step(Foot::Right, &note2);
        assert_ne!(s.feet[Foot::Left as usize].last_fatigue, 0.);
        assert_eq!(s.feet[Foot::Left as usize].last_hit, Some(note1));
        assert_ne!(s.feet[Foot::Right as usize].last_fatigue, 0.);
        assert_eq!(s.feet[Foot::Right as usize].last_hit, Some(note2));
        assert_eq!(s.max_fatigue(), s.fatigue());
        assert_eq!(
            s.max_fatigue(),
            s.feet[0].fatigue(note2.time) + s.feet[1].fatigue(note2.time)
        );

        s = s.step(Foot::Left, &note3);
        assert_ne!(s.fatigue(), s.max_fatigue());
    }
}
