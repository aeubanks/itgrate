use crate::note::Note;

#[derive(Debug, PartialEq, Clone)]
pub struct StepParams {
    pub base_fatigue_per_step: f32,
    pub fatigue_per_step_ratio: f32,
    pub fatigue_dist_ratio: f32,
    pub fatigue_decay_rate: f32,
    pub rest_time_add_constant: f32,
}

impl Default for StepParams {
    fn default() -> Self {
        Self {
            base_fatigue_per_step: 4.8647046,
            fatigue_per_step_ratio: 32.294823,
            fatigue_dist_ratio: 1.6507491,
            fatigue_decay_rate: 0.020347526,
            rest_time_add_constant: 0.5731187,
        }
    }
}

impl StepParams {
    pub fn from_params(params: &[f32]) -> Self {
        assert_eq!(params.len(), 5);
        Self {
            base_fatigue_per_step: params[0],
            fatigue_per_step_ratio: params[1],
            fatigue_dist_ratio: params[2],
            fatigue_decay_rate: params[3],
            rest_time_add_constant: params[4],
        }
    }

    pub fn to_params(&self) -> Vec<f32> {
        vec![
            self.base_fatigue_per_step,
            self.fatigue_per_step_ratio,
            self.fatigue_dist_ratio,
            self.fatigue_decay_rate,
            self.rest_time_add_constant,
        ]
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Default)]
pub struct FootStatus {
    last_hit: Option<Note>,
    last_fatigue: f32,
}

impl FootStatus {
    fn fatigue_after_rest(prev_fatigue: f32, rest_time: f32, step_params: &StepParams) -> f32 {
        prev_fatigue * (-rest_time * step_params.fatigue_decay_rate).exp()
    }

    fn fatigue_after_rest_and_step(
        last_fatigue: f32,
        rest_time: f32,
        distance: f32,
        step_params: &StepParams,
    ) -> f32 {
        // Recover from fatigue exponentially.
        // Add fatigue based on rest time and distance.
        FootStatus::fatigue_after_rest(last_fatigue, rest_time, step_params)
            + step_params.fatigue_per_step_ratio
                * (step_params.base_fatigue_per_step + distance * step_params.fatigue_dist_ratio)
                / (step_params.rest_time_add_constant + rest_time)
    }

    fn step(&mut self, note: &Note, step_params: &StepParams) {
        self.last_fatigue = FootStatus::fatigue_after_rest_and_step(
            self.last_fatigue,
            self.last_hit.map_or(0., |lh| note.time - lh.time),
            self.last_hit.map_or(0., |lh| note.pos.distance(&lh.pos)),
            step_params,
        );
        self.last_hit = Some(*note);
    }

    fn fatigue(&self, time: f32, step_params: &StepParams) -> f32 {
        self.last_hit.map_or(0., |lh| {
            FootStatus::fatigue_after_rest(self.last_fatigue, time - lh.time, step_params)
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
    pub fn step(&self, foot: Foot, note: &Note, step_params: &StepParams) -> State {
        for foot in &self.feet {
            if let Some(last) = foot.last_hit {
                assert!(last.time <= note.time, "stepping an earlier note!");
            }
        }
        let mut copy = *self;
        copy.step_impl(foot, note, step_params);
        copy
    }

    fn step_impl(&mut self, foot: Foot, note: &Note, step_params: &StepParams) {
        let step_foot = &mut self.feet[foot as usize];
        step_foot.step(note, step_params);
        self.cur_fatigue = self
            .feet
            .iter()
            .map(|f| f.fatigue(note.time, step_params))
            .sum();
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
        let step_params = StepParams::default();
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
        s = s.step(Foot::Left, &note1, &step_params);
        assert_ne!(s.feet[Foot::Left as usize].last_fatigue, 0.);
        assert_eq!(s.feet[Foot::Left as usize].last_hit, Some(note1));
        assert_eq!(s.feet[Foot::Right as usize].last_fatigue, 0.);
        assert_eq!(s.feet[Foot::Right as usize].last_hit, None);
        assert_eq!(s.max_fatigue(), s.fatigue());
        assert_eq!(
            s.max_fatigue(),
            s.feet[0].fatigue(note1.time, &step_params)
                + s.feet[1].fatigue(note1.time, &step_params)
        );

        s = s.step(Foot::Right, &note2, &step_params);
        assert_ne!(s.feet[Foot::Left as usize].last_fatigue, 0.);
        assert_eq!(s.feet[Foot::Left as usize].last_hit, Some(note1));
        assert_ne!(s.feet[Foot::Right as usize].last_fatigue, 0.);
        assert_eq!(s.feet[Foot::Right as usize].last_hit, Some(note2));
        assert_eq!(s.max_fatigue(), s.fatigue());
        assert_eq!(
            s.max_fatigue(),
            s.feet[0].fatigue(note2.time, &step_params)
                + s.feet[1].fatigue(note2.time, &step_params)
        );

        s = s.step(Foot::Left, &note3, &step_params);
        assert_ne!(s.fatigue(), s.max_fatigue());
    }
}
