use crate::note::Note;
use crate::note_pos::col_to_pos;
use anyhow::{anyhow, Context, Result};
use std::option::Option;

struct Lines {
    idx: usize,
    lines: Vec<String>,
    verbose: bool,
}

impl Lines {
    fn new(s: &str, verbose: bool) -> Self {
        Self {
            idx: 0,
            lines: s
                .lines()
                .map(|s| {
                    let mut s = s.to_owned();
                    if let Some(p) = s.find("//") {
                        s.truncate(p);
                    }
                    s.trim().to_owned()
                })
                .collect(),
            verbose,
        }
    }
    fn is_valid(&self) -> bool {
        self.idx < self.lines.len()
    }
    fn is_bad_line(s: &str) -> bool {
        s.is_empty() || s.starts_with("//")
    }
    fn skip_bad_lines(&mut self) {
        while self.is_valid() {
            if !Lines::is_bad_line(&self.lines[self.idx]) {
                break;
            }
            self.idx += 1;
        }
    }
    fn peek(&mut self) -> Option<&str> {
        self.skip_bad_lines();
        if self.is_valid() {
            let ret = &self.lines[self.idx];
            if self.verbose {
                println!("peeking at line: {}", ret);
            }
            Some(ret)
        } else {
            None
        }
    }
    fn peek_result(&mut self) -> Result<&str> {
        self.peek()
            .map_or_else(|| Err(anyhow!("peek_expect when empty")), Ok)
    }
    fn consume(&mut self) -> Option<&str> {
        self.skip_bad_lines();
        if self.is_valid() {
            let ret = &self.lines[self.idx];
            if self.verbose {
                println!("consuming line: {}", ret);
            }
            self.idx += 1;
            Some(ret)
        } else {
            None
        }
    }
    fn consume_result(&mut self) -> Result<&str> {
        self.consume()
            .map_or_else(|| Err(anyhow!("consume_expect when empty")), Ok)
    }
}

#[test]
fn test_lines() {
    let mut lines0 = Lines::new("a", false);
    assert!(lines0.peek_result().is_ok());
    assert!(lines0.consume_result().is_ok());
    assert!(lines0.peek_result().is_err());
    assert!(lines0.consume_result().is_err());

    let mut lines1 = Lines::new("", false);
    assert_eq!(lines1.peek(), None);
    assert_eq!(lines1.consume(), None);

    let mut lines2 = Lines::new("a", false);
    assert_eq!(lines2.peek(), Some("a"));
    assert_eq!(lines2.consume(), Some("a"));
    assert_eq!(lines2.peek(), None);
    assert_eq!(lines2.consume(), None);

    let mut lines3 = Lines::new("a\nbas\nqwe", false);
    assert_eq!(lines3.peek(), Some("a"));
    assert_eq!(lines3.consume(), Some("a"));
    assert_eq!(lines3.peek(), Some("bas"));
    assert_eq!(lines3.consume(), Some("bas"));
    assert_eq!(lines3.peek(), Some("qwe"));
    assert_eq!(lines3.consume(), Some("qwe"));
    assert_eq!(lines3.peek(), None);
    assert_eq!(lines3.consume(), None);

    let mut lines4 = Lines::new("a\n\n//\nb", false);
    assert_eq!(lines4.peek(), Some("a"));
    assert_eq!(lines4.consume(), Some("a"));
    assert_eq!(lines4.peek(), Some("b"));
    assert_eq!(lines4.consume(), Some("b"));
    assert_eq!(lines4.peek(), None);
    assert_eq!(lines4.consume(), None);

    let mut lines5 = Lines::new("\n// hihihi\n // \n //\n// \n", false);
    assert_eq!(lines5.peek(), None);
    assert_eq!(lines5.consume(), None);

    let mut lines6 = Lines::new("a //b\nc// d", false);
    assert_eq!(lines6.peek(), Some("a"));
    assert_eq!(lines6.consume(), Some("a"));
    assert_eq!(lines6.peek(), Some("c"));
    assert_eq!(lines6.consume(), Some("c"));
    assert_eq!(lines6.peek(), None);
    assert_eq!(lines6.consume(), None);
}

struct BPMs {
    // (beat, bpm)
    bpm_changes: Vec<(f32, f32)>,
    // TODO: stops?
}

impl BPMs {
    fn interval_time(bpm: f32, beats: f32) -> f32 {
        60. / bpm * beats
    }

    fn beat_to_time(&self, beat: f32) -> f32 {
        // there's probably a faster way to do this
        let mut last_change_beat = 0.;
        let mut last_bpm = 1.;
        let mut ret = 0.;
        for bpm_change in &self.bpm_changes {
            if beat <= bpm_change.0 {
                break;
            }
            let beats_since_last_change = bpm_change.0 - last_change_beat;
            ret += BPMs::interval_time(last_bpm, beats_since_last_change);
            last_change_beat = bpm_change.0;
            last_bpm = bpm_change.1;
        }
        ret += BPMs::interval_time(last_bpm, beat - last_change_beat);
        ret
    }

    fn measure_to_time(&self, measure: f32) -> f32 {
        self.beat_to_time(measure * 4.)
    }
}

#[test]
fn test_bpm_measure_to_time() {
    {
        let b = BPMs {
            bpm_changes: vec![(0., 240.)],
        };
        assert_eq!(b.measure_to_time(0.), 0.);
        assert_eq!(b.measure_to_time(0.5), 0.5);
        assert_eq!(b.measure_to_time(1.), 1.);
        assert_eq!(b.measure_to_time(10.), 10.);
    }
    {
        let b = BPMs {
            bpm_changes: vec![(0., 60.)],
        };
        assert_eq!(b.measure_to_time(0.), 0.);
        assert_eq!(b.measure_to_time(0.5), 2.);
        assert_eq!(b.measure_to_time(2.), 8.);
    }
    {
        let b = BPMs {
            bpm_changes: vec![(0., 60.), (4., 240.), (8., 60.)],
        };
        assert_eq!(b.measure_to_time(0.), 0.);
        assert_eq!(b.measure_to_time(0.5), 2.);
        assert_eq!(b.measure_to_time(1.), 4.);
        assert_eq!(b.measure_to_time(1.5), 4.5);
        assert_eq!(b.measure_to_time(2.), 5.);
        assert_eq!(b.measure_to_time(2.5), 7.);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct SMChart {
    pub mode: String,
    pub author: String,
    pub difficulty: String,
    pub level: i32,
    pub notes: Vec<Note>,
}

#[derive(Debug, Clone)]
pub struct SMResult {
    pub title: String,
    pub charts: Vec<SMChart>,
}

pub fn parse_sm(s: &str, verbose: bool) -> Result<SMResult> {
    let mut lines = Lines::new(s, verbose);

    let title = parse_title(&mut lines).context("finding #TITLE:")?;
    let bpms = parse_bpm(&mut lines).context("finding #BPMS:")?;
    let charts = parse_charts(&mut lines, &bpms)?;

    Ok(SMResult { title, charts })
}

fn parse_title(lines: &mut Lines) -> Result<String> {
    loop {
        let l = lines.consume_result()?;
        if !l.starts_with("#TITLE:") {
            continue;
        }
        let mut title = l[7..].to_owned();
        if title.pop().map_or(true, |c| c != ';') {
            return Err(anyhow!("Didn't find ';' in {}", l));
        }
        return Ok(title);
    }
}

fn parse_bpm(lines: &mut Lines) -> Result<BPMs> {
    let mut bpms = Vec::new();

    let mut bpm_lines = String::new();

    loop {
        let l = lines.consume_result()?;
        if !l.starts_with("#BPMS:") {
            continue;
        }
        bpm_lines.push_str(&l[6..]);
        loop {
            if bpm_lines.ends_with(';') {
                break;
            }
            bpm_lines.push_str(lines.consume_result()?);
        }
        break;
    }

    bpm_lines.pop();

    for bpm_change in bpm_lines.split(',') {
        let equals = match bpm_change.find('=') {
            Some(f) => f,
            None => {
                return Err(anyhow!("didn't find '=' in {}", bpm_change));
            }
        };
        let time = bpm_change[..equals]
            .parse::<f32>()
            .with_context(|| format!("parsing BPM string: {}", bpm_change))?;
        let bpm = bpm_change[(equals + 1)..]
            .parse::<f32>()
            .with_context(|| format!("parsing BPM string: {}", bpm_change))?;
        bpms.push((time, bpm))
    }

    Ok(BPMs { bpm_changes: bpms })
}

fn parse_charts(lines: &mut Lines, bpms: &BPMs) -> Result<Vec<SMChart>> {
    let mut ret = Vec::new();
    while let Some(l) = lines.consume() {
        if l == "#NOTES:" {
            let mut mode = lines.consume_result()?.to_owned();
            if mode.pop() != Some(':') {
                return Err(anyhow!("mode doesn't end in ':'"));
            }
            let mut author = lines.consume_result()?.to_owned();
            if author.pop() != Some(':') {
                return Err(anyhow!("author doesn't end in ':'"));
            }
            let mut difficulty = lines.consume_result()?.to_owned();
            if difficulty.pop() != Some(':') {
                return Err(anyhow!("difficulty doesn't end in ':'"));
            }
            let mut level = lines.consume_result()?.to_owned();
            if level.pop() != Some(':') {
                return Err(anyhow!("level doesn't end in ':'"));
            }
            let mut groove = lines.consume_result()?.to_owned();
            if groove.pop() != Some(':') {
                return Err(anyhow!("groove doesn't end in ':'"));
            }
            let notes = parse_chart(lines, bpms)?;
            ret.push(SMChart {
                mode,
                author,
                difficulty,
                level: level.parse()?,
                notes,
            });
        } else if line_is_notes(l) {
            return Err(anyhow!("Unexpected notes while not in #NOTES section"));
        }
    }
    Ok(ret)
}

fn line_is_notes(l: &str) -> bool {
    l.chars().all(|c| match c {
        '0' | '1' | '2' | '3' | '4' | 'M' | 'L' | 'F' => true,
        _ => false,
    })
}

fn parse_measure(lines: &mut Lines, measure_count: i32, bpms: &BPMs) -> Result<Vec<Note>> {
    let mut ret = Vec::new();
    let mut measure_lines = Vec::new();
    loop {
        if !line_is_notes(lines.peek_result()?) {
            // Done with notes
            break;
        }
        measure_lines.push(lines.consume_result()?.to_owned());
    }
    let measure_line_count = measure_lines.len();
    for (line_idx, l) in measure_lines.iter().enumerate() {
        let measure = line_idx as f32 / measure_line_count as f32 + measure_count as f32;
        let time = bpms.measure_to_time(measure);
        let num_cols = l.len();
        for (col, c) in l.chars().enumerate() {
            if c == '1' || c == '2' || c == '4' || c == 'L' {
                ret.push(Note {
                    pos: col_to_pos(col, num_cols),
                    time,
                })
            }
        }
    }
    Ok(ret)
}

fn parse_chart(lines: &mut Lines, bpms: &BPMs) -> Result<Vec<Note>> {
    let mut ret = Vec::new();
    let mut measure_count = 0;
    if !line_is_notes(lines.peek_result()?) {
        return Err(anyhow!("expected notes, got {:?}", lines.peek_result()));
    }
    while line_is_notes(lines.peek_result()?) {
        let measure = parse_measure(lines, measure_count, bpms)?;
        ret.extend(measure);
        let separator = lines.consume_result()?;
        if separator == ";" {
            break;
        } else if separator != "," {
            return Err(anyhow!("expected ',' separator, got {}", separator));
        }
        measure_count += 1;
    }
    Ok(ret)
}

#[test]
fn test_parse_sm() {
    use crate::note::Pos;

    assert!(parse_sm("", false).is_err());

    assert!(parse_sm("#TITLE:;\na:\nb:\nc:\n1:\ne:\n#BPMS:0.0=240.0", false).is_err());

    assert!(parse_sm("#TITLE:\n#BPMS:0.0=240.0;", false).is_err());

    assert!(parse_sm("#TITLE:a;\n#BPMS:0.0=240.0\n", false).is_err());

    assert!(parse_sm("#TITLE:a;\n#BPMS:0.0240.0\n;", false).is_err());

    assert_eq!(
        parse_sm("#TITLE:a;\n#BPMS:0.0=240.0;", false)
            .unwrap()
            .title,
        "a"
    );

    assert!(parse_sm("#TITLE:a;\n#BPMS:0.0=240.0;", false)
        .unwrap()
        .charts
        .is_empty(),);

    assert!(parse_sm("#TITLE:a;\n#BPMS:0.0=240.0\n;", false)
        .unwrap()
        .charts
        .is_empty(),);

    assert!(parse_sm("#TITLE:a;\n#BPMS:0.0=240.0,10.0=250.0\n;", false)
        .unwrap()
        .charts
        .is_empty(),);

    assert!(parse_sm("#TITLE:a;\n#BPMS:0.0=240.0\n,10.0=250.0;", false)
        .unwrap()
        .charts
        .is_empty(),);

    assert!(
        parse_sm("#TITLE:a;\n#BPMS:0.0=240.0,\n10.0=250.0;\n", false)
            .unwrap()
            .charts
            .is_empty(),
    );

    assert!(
        parse_sm("#TITLE:a;\n#BPMS:0.0=240.0\n,10.0=250.0\n;", false)
            .unwrap()
            .charts
            .is_empty(),
    );

    assert!(parse_sm("#TITLE:a;\n#BPMS:0.0=240.0\n;\n", false)
        .unwrap()
        .charts
        .is_empty(),);

    assert!(parse_sm("#TITLE:a;\n#BPMS:0.0=240.0\n;\n#NOTES:\n", false).is_err());

    assert!(parse_sm(
        "#TITLE:a;\n#BPMS:0.0=240.0\n;\n#NOTES:\na:\nb:\nc:\n1:\ne:\n",
        false
    )
    .is_err());

    assert!(parse_sm(
        "#TITLE:a;\n#BPMS:0.0=240.0\n;\n#NOTES:\na:\nb:\nc:\n1:\ne:\n;\n",
        false
    )
    .is_err());

    assert!(parse_sm(
        "#TITLE:a;\n#BPMS:0.0=240.0\n;\n
        #NOTES:\na:\nb:\nc:\n1:\ne:\n0000\n0000\n0000\n0000\n",
        false
    )
    .is_err());

    assert!(parse_sm(
        "#TITLE:a;\n#BPMS:0.0=240.0\n;\n
        #NOTES:\na:\nb:\nc:\nd:\ne:\n0000\n0000\n0000\n0000\n;",
        false
    )
    .is_err());

    assert_eq!(
        parse_sm(
            "#TITLE:a;\n#BPMS:0.0=240.0\n;
            \n#NOTES:\na:\nb:\nc:\n1:\ne:\n0000\n0000\n0000\n0000\n;\n",
            false
        )
        .unwrap()
        .charts
        .len(),
        1
    );

    assert!(parse_sm(
        "#TITLE:a;\n#BPMS:0.0=240.0\n;\n
        #NOTES:\na:\nb:\nc:\n1:\ne:\n0000\n0000\n0000\n0000\n;\n",
        false
    )
    .unwrap()
    .charts[0]
        .notes
        .is_empty(),);

    assert_eq!(
        parse_sm(
            "#TITLE:a;\n#BPMS:0.0=240.0\n;
            \n#NOTES:\na:\nb:\nc:\n1:\ne:\n1000\n0000\n0000\n0000\n;\n",
            false
        )
        .unwrap()
        .charts[0]
            .notes,
        vec![Note {
            pos: Pos { x: 0., y: 1. },
            time: 0.
        }]
    );

    assert_eq!(
        parse_sm(
            "#TITLE:a;\n#BPMS:0.0=240.0\n;\n#NOTES:\na:\nb:\nc:\n1:\ne:\n
            0011\n0000\n0000\n0000\n;\n",
            false
        )
        .unwrap()
        .charts[0]
            .notes,
        vec![
            Note {
                pos: Pos { x: 1., y: 2. },
                time: 0.
            },
            Note {
                pos: Pos { x: 2., y: 1. },
                time: 0.
            }
        ]
    );

    assert_eq!(
        parse_sm(
            "#TITLE:a;\n#BPMS:0.0=240.0\n;\n
            #NOTES:\na:\nb:\nc:\n1:\ne:\n0000\n0000\n0010\n0000\n;\n",
            false
        )
        .unwrap()
        .charts[0]
            .notes,
        vec![Note {
            pos: Pos { x: 1., y: 2. },
            time: 0.5
        }]
    );

    assert_eq!(
        parse_sm(
            "#TITLE:a;\n#BPMS:0.0=240.0\n;
            \n#NOTES:\na:\nb:\nc:\n1:\ne:\n\n0000\n0000\n0100\n0000\n,
            \n0000\n0010\n0000\n0000\n;\n",
            false
        )
        .unwrap()
        .charts[0]
            .notes,
        vec![
            Note {
                pos: Pos { x: 1., y: 0. },
                time: 0.5
            },
            Note {
                pos: Pos { x: 1., y: 2. },
                time: 1.25
            }
        ]
    );

    assert_eq!(
        parse_sm(
            "#TITLE:a;\n#BPMS:0.0=240.0\n;
            \n#NOTES:\na:\na:\na:\n1:\na:\n0000\n0000\n0000\n0000\n;
            \n\n#NOTES:\nz:\nz:\nz:\n2:\nz:\n0000\n;",
            false
        )
        .unwrap()
        .charts,
        vec![
            SMChart {
                mode: "a".to_owned(),
                author: "a".to_owned(),
                difficulty: "a".to_owned(),
                level: 1,
                notes: vec![]
            },
            SMChart {
                difficulty: "z".to_owned(),
                mode: "z".to_owned(),
                author: "z".to_owned(),
                level: 2,
                notes: vec![]
            }
        ]
    );

    assert_eq!(
        parse_sm(
            "#TITLE:a;\n#BPMS:0.0=240.0\n;\n#NOTES:\nz:\ny:\nx:\n2:\nv:
            \n\n0000\n0000\n0000\n1000\n;\n\n#NOTES:\na:\nb:\nc:\n3:\ne:\n1000\n;",
            false
        )
        .unwrap()
        .charts,
        vec![
            SMChart {
                mode: "z".to_owned(),
                author: "y".to_owned(),
                difficulty: "x".to_owned(),
                level: 2,
                notes: vec![Note {
                    pos: Pos { x: 0., y: 1. },
                    time: 0.75
                }]
            },
            SMChart {
                mode: "a".to_owned(),
                author: "b".to_owned(),
                difficulty: "c".to_owned(),
                level: 3,
                notes: vec![Note {
                    pos: Pos { x: 0., y: 1. },
                    time: 0.
                }]
            }
        ]
    );

    assert_eq!(
        parse_sm(
            "#TITLE:a;\n#BPMS:0.0=240.0\n;
            \n#NOTES:\na:\nb:\nc:\n1:\ne:\n1234\n0000\nLFM0\n0000\n;\n",
            false
        )
        .unwrap()
        .charts[0]
            .notes,
        vec![
            Note {
                pos: Pos { x: 0., y: 1. },
                time: 0.
            },
            Note {
                pos: Pos { x: 1., y: 0. },
                time: 0.
            },
            Note {
                pos: Pos { x: 2., y: 1. },
                time: 0.
            },
            Note {
                pos: Pos { x: 0., y: 1. },
                time: 0.5
            }
        ]
    );

    assert_eq!(
        parse_sm(
            "#TITLE:a;\n#BPMS:0.0=240.0,2.0=60.0;
            \n#NOTES:\na:\nb:\nc:\n1:\ne:\n1000\n1000\n1000\n1000\n;",
            false
        )
        .unwrap()
        .charts[0]
            .notes,
        vec![
            Note {
                pos: Pos { x: 0., y: 1. },
                time: 0.
            },
            Note {
                pos: Pos { x: 0., y: 1. },
                time: 0.25
            },
            Note {
                pos: Pos { x: 0., y: 1. },
                time: 0.5
            },
            Note {
                pos: Pos { x: 0., y: 1. },
                time: 1.5
            }
        ]
    );
    assert_eq!(
        parse_sm(
            "#TITLE:a;\n#BPMS:0.0=240.0\n,2.0=60.0\n;
            \n#NOTES:\na:\nb:\nc:\n1:\ne:\n1000\n1000\n1000\n1000\n;",
            false
        )
        .unwrap()
        .charts[0]
            .notes,
        vec![
            Note {
                pos: Pos { x: 0., y: 1. },
                time: 0.
            },
            Note {
                pos: Pos { x: 0., y: 1. },
                time: 0.25
            },
            Note {
                pos: Pos { x: 0., y: 1. },
                time: 0.5
            },
            Note {
                pos: Pos { x: 0., y: 1. },
                time: 1.5
            }
        ]
    );
}
