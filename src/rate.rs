use crate::smparser::Chart;
use rand::distributions::{Distribution, Uniform};
use rand::Rng;

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Ratio {
    Linear(f32),
    Exp(f32, f32),
    Recip(f32, f32),
    Sigmoid(f32, f32),
    Tanh(f32),
}

#[derive(Clone, Copy, Debug)]
pub struct Params {
    pub step_1: f32,
    pub step_2: f32,
    pub dt_const: f32,
    pub ratio: Ratio,
}

impl Params {
    fn ratio(&self, dt: f32) -> f32 {
        use Ratio::*;
        // ratio = 1 when dt = 0, ratio -> 0 as dt -> inf
        match self.ratio {
            // max(1-ax,0)
            Linear(a) => (1.0 - a * dt).max(0.0),
            // a^(-bx)
            Exp(a, b) => a.powf(-b * dt),
            // 1/(1+ax)^b
            Recip(a, b) => 1.0 / (1.0 + a * dt).powf(b),
            // 2-2/(1+a^(-bx))
            Sigmoid(a, b) => 2.0 - 2.0 / (1.0 + a.powf(-b * dt)),
            // 1-tanh(ax)
            Tanh(a) => 1.0 - (a * dt).tanh(),
        }
    }

    pub fn rand<R>(&mut self, rng: &mut R)
    where
        R: Rng + ?Sized,
    {
        let range = Uniform::from(0.9..1.1);
        if rng.gen() {
            self.step_1 *= range.sample(rng);
        }
        if rng.gen() {
            self.step_2 *= range.sample(rng);
        }
        if rng.gen() {
            self.dt_const *= range.sample(rng);
        }
        use Ratio::*;
        // ratio = 1 when dt = 0, ratio -> 0 as dt -> inf
        match &mut self.ratio {
            Linear(a) => {
                if rng.gen() {
                    *a *= range.sample(rng);
                }
            }
            Exp(a, b) => {
                if rng.gen() {
                    *a *= range.sample(rng);
                }
                if rng.gen() {
                    *b *= range.sample(rng);
                }
            }
            Recip(a, b) => {
                if rng.gen() {
                    *a *= range.sample(rng);
                }
                if rng.gen() {
                    *b *= range.sample(rng);
                }
            }
            Sigmoid(a, b) => {
                if rng.gen() {
                    *a *= range.sample(rng);
                }
                if rng.gen() {
                    *b *= range.sample(rng);
                }
            }
            Tanh(a) => {
                if rng.gen() {
                    *a *= range.sample(rng);
                }
            }
        }
    }
}

struct State {
    cur_fatigue: f32,
    max_fatigue: f32,
    last_time: f32,
    params: Params,
}

impl State {
    fn with_params(params: Params) -> Self {
        Self {
            cur_fatigue: 0.,
            max_fatigue: 0.,
            last_time: 0.,
            params,
        }
    }
    fn step(&mut self, time: f32) {
        let dt = time - self.last_time + self.params.dt_const;
        assert!(dt >= 0.);

        let ratio = self.params.ratio(dt);

        assert!(ratio >= 0.0);
        assert!(ratio <= 1.0);

        self.cur_fatigue *= ratio;
        self.cur_fatigue += self.params.step_1 + dt * self.params.step_2;

        if self.cur_fatigue > self.max_fatigue {
            self.max_fatigue = self.cur_fatigue;
        }

        self.last_time = time;
    }
}

pub fn rate(chart: &Chart, params: Params) -> (f32, Vec<(f32, f32)>) {
    let mut fatigue = State::with_params(params);
    let mut fatigues = Vec::with_capacity(chart.notes.len());
    fatigues.push((0.0, 0.0));
    for note in &chart.notes {
        fatigue.step(note.time);
        fatigues.push((note.time, fatigue.cur_fatigue))
    }
    (fatigue.max_fatigue, fatigues)
}
