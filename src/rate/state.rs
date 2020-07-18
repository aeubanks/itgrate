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
    pub fn with_max_fatigue(mf: f32) -> Self {
        let mut ret = Self::default();
        ret.max_fatigue = mf;
        ret
    }
}

impl State {
    pub fn step(&self, foot: Foot, note: Note) -> State {
        for foot in &self.feet {
            if let Some(last) = foot.last_hit {
                assert!(last.time <= note.time, "stepping an earlier note!");
            }
        }
        let mut copy = *self;

        let mut foot = &mut copy.feet[foot as usize];
        foot.fatigue += 0.5;
        foot.last_hit = Some(note);
        copy.fatigue += 0.5;
        copy.max_fatigue = self.max_fatigue.max(copy.fatigue);
        copy
    }

    pub fn max_fatigue(&self) -> f32 {
        self.max_fatigue
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
        assert_eq!(s.feet[Foot::Left as usize].fatigue, 0.5);
        assert_eq!(s.feet[Foot::Left as usize].last_hit, Some(note1));
        assert_eq!(s.feet[Foot::Right as usize].fatigue, 0.);
        assert_eq!(s.feet[Foot::Right as usize].last_hit, None);
        assert_eq!(s.fatigue, 0.5);
        assert_eq!(s.max_fatigue, 0.5);

        s = s.step(Foot::Right, note2);
        assert_eq!(s.feet[Foot::Left as usize].fatigue, 0.5);
        assert_eq!(s.feet[Foot::Left as usize].last_hit, Some(note1));
        assert_eq!(s.feet[Foot::Right as usize].fatigue, 0.5);
        assert_eq!(s.feet[Foot::Right as usize].last_hit, Some(note2));
        assert_eq!(s.fatigue, 1.);
        assert_eq!(s.max_fatigue, 1.);
    }
}
