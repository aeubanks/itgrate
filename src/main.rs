mod rate;
mod smparser;

use clap::Parser;
use rand::distributions::{Distribution, Uniform};
use rand::Rng;
use rate::{rate, Params};
use smparser::Chart;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    #[structopt(help = "Paths of/directories containing .sm files")]
    inputs: Vec<PathBuf>,
}

fn sm_files_impl(path: &PathBuf, set: &mut HashSet<PathBuf>) {
    let metadata = std::fs::metadata(path).expect("couldn't get metadata for path");
    if metadata.is_file() {
        if let Some(Some(ext)) = path.extension().map(|e| e.to_str()) {
            if ext.to_lowercase() == "sm" {
                set.insert(path.clone());
            }
        }
    } else if metadata.is_dir() {
        for de in std::fs::read_dir(path).expect("couldn't read entries in directory") {
            sm_files_impl(&de.unwrap().path(), set);
        }
    }
}

fn sm_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut set = HashSet::new();
    for path in paths {
        sm_files_impl(path, &mut set);
    }
    set.into_iter().collect()
}

fn charts(sm_files: &[PathBuf]) -> Vec<smparser::Chart> {
    let mut charts = Vec::new();
    for sm_file in sm_files {
        println!("Reading {:?}", sm_file);
        let buf = std::fs::read(sm_file).unwrap();
        let str = std::str::from_utf8(&buf).unwrap();
        for chart in smparser::parse(str) {
            charts.push(chart);
        }
    }
    charts
}

fn error(charts: &[Chart], params: Params) -> f32 {
    let mut error = 0.;
    for chart in charts {
        let rating = rate(chart, params);
        let dr = rating - (chart.rating as f32 + 0.5);
        error += dr * dr;
    }
    error / charts.len() as f32
}

fn main() {
    let args = Args::parse();

    let sm_files = sm_files(&args.inputs);

    if sm_files.is_empty() {
        println!("Didn't find any sm files");
        std::process::exit(1);
    }

    let charts = charts(&sm_files);

    let mut params = Params {
        step_1: 0.05,
        step_2: 0.05,
        linear: 0.1,
        exp_1: 2.0,
        exp_2: 2.0,
        recip_1: 2.0,
        recip_2: 2.0,
        sigmoid_1: 2.0,
        sigmoid_2: 2.0,
        tanh_1: 2.0,
    };
    let mut err = error(&charts, params);

    let mut rng = rand::thread_rng();
    let range = Uniform::from(0.9..1.1);

    for i in 0..9999 {
        let mut new_params = params;
        if rng.gen() {
            new_params.step_1 *= range.sample(&mut rng);
        }
        if rng.gen() {
            new_params.step_2 *= range.sample(&mut rng);
        }
        if rng.gen() {
            new_params.linear *= range.sample(&mut rng);
        }
        let new_err = error(&charts, new_params);
        if new_err < err {
            params = new_params;
            err = new_err;
            println!("iteration {}", i);
            println!("better params: {:?}, err {}", params, err);
        }
    }

    println!("params: {:?}, err {}", params, err);

    let mut ratings = Vec::new();
    for chart in charts {
        let r = rate(&chart, params);
        ratings.push((r, chart));
    }
    ratings.sort_by(|(r1, _), (r2, _)| r1.total_cmp(r2));

    for (r, c) in ratings {
        println!(
            "{:>5.2}: {:2}, {:6} notes - {} ({})",
            r,
            c.rating,
            c.notes.len(),
            c.title,
            c.difficulty
        );
    }
}
