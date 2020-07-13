use anyhow::Result;
use rate::rate_notes;
use std::path::PathBuf;
use structopt::StructOpt;

mod note;
mod note_pos;
mod rate;
mod smparser;

#[derive(Debug, StructOpt)]
#[structopt()]
struct Opts {
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    #[structopt(short = "v")]
    verbose: bool,
}

fn main() -> Result<()> {
    let opts = Opts::from_args();
    let s = std::fs::read_to_string(opts.input)?;
    let notes = smparser::parse_sm(&s, opts.verbose)?;
    let ratings = notes.iter().map(|n| rate_notes(n));
    for r in ratings {
        println!("{}", r);
    }
    Ok(())
}
