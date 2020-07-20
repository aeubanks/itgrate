mod note;
mod note_pos;
mod rate;
mod smparser;

use anyhow::Result;
use rate::rate_notes;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt()]
struct Opts {
    #[structopt(parse(from_os_str))]
    inputs: Vec<PathBuf>,

    #[structopt(short = "v")]
    verbose: bool,
}

fn main() -> Result<()> {
    let opts = Opts::from_args();
    for input in opts.inputs {
        let s = std::fs::read_to_string(input)?;
        let smresult = smparser::parse_sm(&s, opts.verbose)?;
        for (difficulty, notes) in smresult.charts {
            println!("{} {}", smresult.title, difficulty);
            let rating = rate_notes(&notes);
            println!("{}", rating);
        }
    }
    Ok(())
}
