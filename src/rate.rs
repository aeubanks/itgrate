use crate::chart::Chart;
use autodiff::{Float, F1};

#[derive(Clone, Copy, Debug)]
pub struct Params {
    pub step_1: F1,
    pub step_2: F1,
    pub dt_const: F1,
    pub ratio_1: F1,
    pub ratio_2: F1,
}

impl Params {
    pub fn new(step_1: f64, step_2: f64, dt_const: f64, ratio_1: f64, ratio_2: f64) -> Self {
        Self {
            step_1: F1::cst(step_1),
            step_2: F1::cst(step_2),
            dt_const: F1::cst(dt_const),
            ratio_1: F1::cst(ratio_1),
            ratio_2: F1::cst(ratio_2),
        }
    }

    pub fn to_vec(&self) -> Vec<f64> {
        vec![
            self.step_1.value(),
            self.step_2.value(),
            self.dt_const.value(),
            self.ratio_1.value(),
            self.ratio_2.value(),
        ]
    }

    pub fn from_vec(v: &[f64]) -> Self {
        Self::new(v[0], v[1], v[2], v[3], v[4])
    }
}

struct State {
    cur_fatigue: F1,
    max_fatigue: F1,
    last_time: f32,
    params: Params,
}

impl State {
    fn with_params(params: Params) -> Self {
        Self {
            cur_fatigue: F1::cst(0.0),
            max_fatigue: F1::cst(0.0),
            last_time: 0.,
            params,
        }
    }
    fn step(&mut self, time: f32) {
        let dt = F1::cst(time - self.last_time + 1.0).ln() + self.params.dt_const;
        assert!(dt.value() >= 0.);

        let ratio =
            F1::cst(1.0) / (F1::cst(1.0) + self.params.ratio_1 * dt).powf(self.params.ratio_2);

        if ratio.value() < 0.0 || ratio.value() > 1.0 {
            panic!("unexpected ratio: {}, dt {}", ratio, dt);
        }

        self.cur_fatigue *= ratio;
        self.cur_fatigue += self.params.step_1 + dt * self.params.step_2;

        if self.cur_fatigue > self.max_fatigue {
            self.max_fatigue = self.cur_fatigue;
        }

        self.last_time = time;
    }
}

pub fn rate(chart: &Chart, params: Params) -> (F1, Vec<(f32, f32)>) {
    let mut fatigue = State::with_params(params);
    let mut fatigues = Vec::with_capacity(chart.notes.len());
    fatigues.push((0.0, 0.0));
    for note in &chart.notes {
        fatigue.step(note.time);
        fatigues.push((note.time, fatigue.cur_fatigue.value() as f32))
    }
    (fatigue.max_fatigue, fatigues)
}
