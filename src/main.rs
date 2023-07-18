mod chart;
mod rate;
mod smparser;

use chart::Chart;
use clap::Parser;
use rate::{rate, Params, Ratio};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    #[structopt(help = "Paths of/directories containing .sm files")]
    inputs: Vec<PathBuf>,

    #[structopt(help = "Use preset charts", short = 'p')]
    use_preset_charts: bool,

    #[structopt(help = "Output graph path", short)]
    graph_path: Option<PathBuf>,

    #[structopt(
        help = "Iterations to hill climb",
        short = 'i',
        long = "hill-climb-iterations",
        default_value = "9999"
    )]
    hill_climb_iterations: i32,
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

fn charts(sm_files: &[PathBuf], preset_charts: bool) -> Vec<Chart> {
    let mut charts = Vec::new();
    for sm_file in sm_files {
        println!("Reading {:?}", sm_file);
        let buf = std::fs::read(sm_file).unwrap();
        let str = std::str::from_utf8(&buf).unwrap();
        for chart in smparser::parse(str) {
            charts.push(chart);
        }
    }
    if preset_charts {
        charts.push(Chart::from_unbroken(170., 96, 15));
        charts.push(Chart::from_unbroken(170., 128, 15));
        charts.push(Chart::from_unbroken(170., 192, 16));
        charts.push(Chart::from_unbroken(170., 256, 16));
        charts.push(Chart::from_unbroken(170., 384, 17));
        charts.push(Chart::from_unbroken(170., 512, 17));

        charts.push(Chart::from_unbroken(180., 64, 15));
        charts.push(Chart::from_unbroken(180., 96, 15));
        charts.push(Chart::from_unbroken(180., 128, 16));
        charts.push(Chart::from_unbroken(180., 192, 16));
        charts.push(Chart::from_unbroken(180., 256, 17));
        charts.push(Chart::from_unbroken(180., 384, 17));
        charts.push(Chart::from_unbroken(180., 512, 18));

        charts.push(Chart::from_unbroken(190., 48, 15));
        charts.push(Chart::from_unbroken(190., 64, 15));
        charts.push(Chart::from_unbroken(190., 96, 16));
        charts.push(Chart::from_unbroken(190., 128, 17));
        charts.push(Chart::from_unbroken(190., 192, 17));
        charts.push(Chart::from_unbroken(190., 256, 18));
        charts.push(Chart::from_unbroken(190., 384, 18));
        charts.push(Chart::from_unbroken(190., 512, 19));

        charts.push(Chart::from_unbroken(200., 32, 15));
        charts.push(Chart::from_unbroken(200., 48, 15));
        charts.push(Chart::from_unbroken(200., 64, 16));
        charts.push(Chart::from_unbroken(200., 96, 17));
        charts.push(Chart::from_unbroken(200., 128, 17));
        charts.push(Chart::from_unbroken(200., 192, 18));
        charts.push(Chart::from_unbroken(200., 256, 19));
        charts.push(Chart::from_unbroken(200., 384, 19));
        charts.push(Chart::from_unbroken(200., 512, 20));

        charts.push(Chart::from_unbroken(210., 32, 15));
        charts.push(Chart::from_unbroken(210., 48, 16));
        charts.push(Chart::from_unbroken(210., 64, 17));
        charts.push(Chart::from_unbroken(210., 96, 18));
        charts.push(Chart::from_unbroken(210., 128, 18));
        charts.push(Chart::from_unbroken(210., 192, 19));
        charts.push(Chart::from_unbroken(210., 256, 20));
        charts.push(Chart::from_unbroken(210., 384, 20));
        charts.push(Chart::from_unbroken(210., 512, 21));

        charts.push(Chart::from_unbroken(220., 32, 16));
        charts.push(Chart::from_unbroken(220., 48, 17));
        charts.push(Chart::from_unbroken(220., 64, 18));
        charts.push(Chart::from_unbroken(220., 96, 19));
        charts.push(Chart::from_unbroken(220., 128, 19));
        charts.push(Chart::from_unbroken(220., 192, 20));
        charts.push(Chart::from_unbroken(220., 256, 21));
        charts.push(Chart::from_unbroken(220., 384, 22));
        charts.push(Chart::from_unbroken(220., 512, 22));

        charts.push(Chart::from_unbroken(230., 32, 17));
        charts.push(Chart::from_unbroken(230., 48, 18));
        charts.push(Chart::from_unbroken(230., 64, 19));
        charts.push(Chart::from_unbroken(230., 96, 20));
        charts.push(Chart::from_unbroken(230., 128, 20));
        charts.push(Chart::from_unbroken(230., 192, 21));
        charts.push(Chart::from_unbroken(230., 256, 22));
        charts.push(Chart::from_unbroken(230., 384, 22));
        charts.push(Chart::from_unbroken(230., 512, 23));
    }
    charts
}

fn error(charts: &[Chart], params: Params) -> f32 {
    let mut error = 0.;
    for chart in charts {
        let (rating, _) = rate(chart, params);
        let dr = rating - (chart.rating as f32 + 0.5);
        error += dr * dr;
    }
    error / charts.len() as f32
}

fn graph_fatigues(path: &PathBuf, charts: &Vec<(Chart, f32, Vec<(f32, f32)>)>) {
    use gnuplot::*;

    let mut fg = gnuplot::Figure::new();
    let a = fg
        .axes2d()
        .set_x_label("time", &[])
        .set_y_label("fatigue", &[]);
    for (chart, _, fatigue_times) in charts {
        let times = fatigue_times.iter().map(|(a, _)| *a).collect::<Vec<_>>();
        let fatigues = fatigue_times.iter().map(|(_, a)| *a).collect::<Vec<_>>();
        let caption = format!("{} ({})", chart.title, chart.difficulty);
        a.points(times, fatigues, &[PlotOption::Caption(&caption)]);
    }
    fg.save_to_png(path, 1280, 720).unwrap();
    println!("drew fatigue graph to {:?}", path);
}

fn main() {
    let args = Args::parse();

    let sm_files = sm_files(&args.inputs);

    let charts = charts(&sm_files, args.use_preset_charts);

    if charts.is_empty() {
        println!("No simfiles?");
        std::process::exit(1);
    }

    let mut params = Params {
        step_1: 0.065,
        step_2: -0.05,
        dt_const: 0.0023,
        ratio: Ratio::Linear(0.0422),
    };
    let mut err = error(&charts, params);

    let mut rng = rand::thread_rng();

    for i in 0..args.hill_climb_iterations {
        let mut new_params = params;
        new_params.rand(&mut rng);
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
        let (rating, fatigues) = rate(&chart, params);
        ratings.push((chart, rating, fatigues));
    }
    ratings.sort_by(|(_, r1, _), (_, r2, _)| r1.total_cmp(r2));

    if let Some(graph_path) = args.graph_path {
        graph_fatigues(&graph_path, &ratings);
    }

    for (chart, rating, _) in ratings {
        println!(
            "{:>5.2}: {:2}, {:6} notes - {} ({})",
            rating,
            chart.rating,
            chart.notes.len(),
            chart.title,
            chart.difficulty
        );
    }
}
