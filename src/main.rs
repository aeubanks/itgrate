mod chart;
mod rate;
mod smparser;
mod train;

use chart::Chart;
use clap::{Parser, Subcommand};
use rate::{rate, Params};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    #[arg(help = "Paths of/directories containing .sm files", global = true)]
    inputs: Vec<PathBuf>,

    #[arg(help = "Use preset charts", short = 'p', global = true)]
    use_preset_charts: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Train {
        #[arg(
            help = "Iterations to perform gradient descent",
            short = 'i',
            long = "gradient-descent-iterations",
            default_value = "999"
        )]
        gradient_descent_iterations: i32,
    },
    Graph {
        #[arg(help = "Output graph path", short = 'o')]
        graph_path: PathBuf,
    },
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

fn charts(
    sm_files: &[PathBuf],
    preset_charts: bool,
    only_longest_preset_charts: bool,
) -> Vec<Chart> {
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
        charts.append(&mut Chart::presets(only_longest_preset_charts));
    }
    charts
}

fn graph_fatigues(path: &PathBuf, charts: &[(&Chart, &Vec<(f32, f32)>)]) {
    use gnuplot::*;

    let mut fg = gnuplot::Figure::new();
    let a = fg
        .axes2d()
        .set_x_label("time", &[])
        .set_y_label("fatigue", &[]);
    for (chart, fatigue_times) in charts {
        let times = fatigue_times.iter().map(|(a, _)| *a).collect::<Vec<_>>();
        let fatigues = fatigue_times.iter().map(|(_, a)| *a).collect::<Vec<_>>();
        let caption = chart.description().replace('@', "\\@");
        a.points(times, fatigues, &[PlotOption::Caption(&caption)]);
    }
    fg.save_to_png(path, 1280, 720).unwrap();
    println!("drew fatigue graph to {:?}", path);
}

fn main() {
    let args = Args::parse();

    let sm_files = sm_files(&args.inputs);

    let charts = charts(
        &sm_files,
        args.use_preset_charts,
        matches!(args.command, Command::Graph { graph_path: _ }),
    );

    if charts.is_empty() {
        println!("No simfiles?");
        std::process::exit(1);
    }

    let mut params = Params::new(1.6725047878328008, 22.69176212395888, 0.03094850290834286);

    if let Command::Train {
        gradient_descent_iterations,
    } = args.command
    {
        let err;
        (params, err) = train::train(&charts, params, gradient_descent_iterations);
        println!("-------------");
        println!("params: {:?}", params.to_vec());
        println!("err: {err}");
    }

    let mut ratings = Vec::new();
    for chart in charts {
        let (rating, fatigues) = rate(&chart, params);
        ratings.push((chart, rating.value(), fatigues));
    }
    ratings.sort_by(|(_, r1, _), (_, r2, _)| r1.total_cmp(r2));

    if let Command::Graph { graph_path } = args.command {
        let mapped = ratings.iter().map(|(a, _, c)| (a, c)).collect::<Vec<_>>();
        graph_fatigues(&graph_path, &mapped);
    }

    for (chart, rating, _) in ratings {
        println!(
            "{:>5.2}: {:2}, {:6} notes - {}",
            rating,
            chart.rating,
            chart.notes.len(),
            chart.description(),
        );
    }
}
