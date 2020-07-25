use crate::rate::rate_notes;
use crate::rate::state::StepParams;
use crate::smparser::SMChart;
use anyhow::Result;
use oxigen::prelude::*;
use rand::prelude::*;
use rand::rngs::SmallRng;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::slice::Iter;
use std::vec::IntoIter;

#[derive(Clone)]
struct RaterGenotype<'a> {
    params: Vec<f32>,
    charts: &'a [SMChart],
}

impl Display for RaterGenotype<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        Debug::fmt(&self.params, f)
    }
}

impl<'a> Genotype<f32> for RaterGenotype<'a> {
    type ProblemSize = &'a [SMChart];

    fn into_iter(self) -> IntoIter<f32> {
        self.params.into_iter()
    }

    fn iter(&self) -> Iter<f32> {
        self.params.iter()
    }

    fn from_iter<I: Iterator<Item = f32>>(&mut self, i: I) {
        self.params = i.into_iter().collect();
    }

    fn generate(charts: &Self::ProblemSize) -> Self {
        let default = StepParams::default();
        Self {
            params: default.to_params(),
            charts,
        }
    }

    fn fitness(&self) -> f64 {
        let step_params = StepParams::from_params(&self.params);
        let mut errors = Vec::with_capacity(self.charts.len());
        for chart in self.charts {
            let rating = rate_notes(&chart.notes, &step_params);
            errors.push(rating - (chart.level as f32 + 0.5));
        }
        -errors.iter().map(|f| f * f).sum::<f32>().sqrt() as f64
    }

    fn mutate(&mut self, rgen: &mut SmallRng, _index: usize) {
        for f in &mut self.params {
            *f *= rgen.gen_range(0.5, 2.);
        }
    }

    fn is_solution(&self, _fitness: f64) -> bool {
        false
    }

    fn fix(&mut self) -> bool {
        let mut changed = false;
        for p in &mut self.params {
            if *p <= 0. {
                *p = 0.0001;
                changed = true;
            }
        }
        changed
    }

    fn distance(&self, other: &Self) -> f64 {
        self.params
            .iter()
            .zip(other.iter())
            .map(|(a, b)| a - b)
            .map(|f| f * f)
            .sum::<f32>()
            .sqrt() as f64
    }
}

pub fn optimize(charts: &[SMChart], generations: u64) -> Result<StepParams> {
    assert!(!charts.is_empty());

    let progress_log = File::create("/dev/stdout")?;

    let (_solutions, _generation, _progress, population) =
        GeneticExecution::<f32, RaterGenotype>::new()
            .stop_criterion(Box::new(StopCriteria::Generation(generations)))
            .population_size(32)
            .genotype_size(charts)
            .progress_log(1, progress_log)
            .run();
    let mut best = &population[0].ind;
    let mut best_fitness = std::f32::NEG_INFINITY;
    for p in population.iter().skip(1) {
        let f = p.ind.fitness() as f32;
        if f > best_fitness {
            best_fitness = f;
            best = &p.ind;
        }
    }
    Ok(StepParams::from_params(&best.params))
}

#[test]
fn sanity() -> Result<()> {
    let chart = SMChart {
        title: "".to_owned(),
        mode: "".to_owned(),
        author: "".to_owned(),
        difficulty: "".to_owned(),
        level: 1,
        notes: vec![crate::note::Note {
            pos: crate::note::Pos { x: 0., y: 1. },
            time: 1.,
        }],
    };
    optimize(&[chart], 2)?;
    Ok(())
}
