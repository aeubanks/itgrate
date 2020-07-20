mod note;
mod note_pos;
mod rate;
mod smparser;

use anyhow::Result;
use gnuplot::*;
use rate::{fatigues_at_notes, rate_notes};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt()]
struct Opts {
    #[structopt(parse(from_os_str))]
    inputs: Vec<PathBuf>,

    #[structopt(short = "v")]
    verbose: bool,

    #[structopt(parse(from_os_str), short = "g")]
    graph: Option<PathBuf>,
}

fn graph(path: &PathBuf, vals: Vec<(String, Vec<f32>, Vec<f32>)>) {
    let mut fg = gnuplot::Figure::new();
    let a = fg
        .axes2d()
        .set_x_label("time", &[])
        .set_y_label("fatigue", &[]);
    for (name, times, fatigues) in vals {
        a.points(times, fatigues, &[PlotOption::Caption(&name)]);
    }

    fg.save_to_png(path, 1280, 720).unwrap();
}

fn main() -> Result<()> {
    let opts = Opts::from_args();
    let mut all_fatigues_over_time = Vec::new();
    for input in opts.inputs {
        let s = std::fs::read_to_string(input)?;
        let smresult = smparser::parse_sm(&s, opts.verbose)?;
        for (difficulty, notes) in smresult.charts {
            let name = format!("{} ({})", smresult.title, difficulty);
            println!("{}", name);
            let rating = rate_notes(&notes);
            println!("{}", rating);
            if opts.graph.is_some() {
                let fatigues = fatigues_at_notes(&notes);
                all_fatigues_over_time.push((
                    name,
                    notes.iter().map(|n| n.time).collect(),
                    fatigues,
                ));
            }
        }
    }
    if let Some(graph_path) = opts.graph {
        graph(&graph_path, all_fatigues_over_time);
    }
    Ok(())
}
