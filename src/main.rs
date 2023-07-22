mod chart;
mod rate;
mod smparser;
mod train;

use chart::Chart;
use clap::{Parser, Subcommand};
use rate::{rate, Params, Ratio};
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
            help = "Iterations to hill climb",
            short = 'i',
            long = "hill-climb-iterations",
            default_value = "9999"
        )]
        hill_climb_iterations: i32,
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
        let caption = chart.description().replace("@", "\\@");
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

    let mut params = Params {
        step_1: 0.065,
        step_2: -0.05,
        dt_const: 0.0023,
        ratio: Ratio::Linear(0.0422),
    };

    if let Command::Train {
        hill_climb_iterations,
    } = args.command
    {
        params = train::train(&charts, params, hill_climb_iterations);
    }

    let mut ratings = Vec::new();
    for chart in charts {
        let (rating, fatigues) = rate(&chart, params);
        ratings.push((chart, rating, fatigues));
    }
    ratings.sort_by(|(_, r1, _), (_, r2, _)| r1.total_cmp(r2));

    if let Command::Graph { graph_path } = args.command {
        graph_fatigues(&graph_path, &ratings);
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
