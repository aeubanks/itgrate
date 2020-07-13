use crate::note::Note;
use crate::note_pos::col_to_pos;
use anyhow::{anyhow, Result};
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
                println!("consuming line:  {}", ret);
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

pub fn parse_sm(s: &str, verbose: bool) -> Result<Vec<Vec<Note>>> {
    let mut ret = Vec::new();
    let mut lines = Lines::new(s, verbose);

    while let Some(l) = lines.consume() {
        if l == "#NOTES:" {
            loop {
                let p = lines.peek_result()?;
                if !p.ends_with(':') {
                    break;
                }
                lines.consume();
            }
            let notes = parse_notes(&mut lines)?;
            ret.push(notes);
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

fn parse_measure(lines: &mut Lines, measure_count: i32) -> Result<Vec<Note>> {
    let mut ret = Vec::new();
    let mut measure = Vec::new();
    loop {
        if !line_is_notes(lines.peek_result()?) {
            // Done with notes
            break;
        }
        measure.push(lines.consume_result()?.to_owned());
    }
    let measure_line_count = measure.len();
    for (line_idx, l) in measure.iter().enumerate() {
        let time = line_idx as f32 / measure_line_count as f32 + measure_count as f32;
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

fn parse_notes(lines: &mut Lines) -> Result<Vec<Note>> {
    let mut ret = Vec::new();
    let mut measure_count = 0;
    if !line_is_notes(lines.peek_result()?) {
        return Err(anyhow!("expected notes, got {:?}", lines.peek_result()));
    }
    while line_is_notes(lines.peek_result()?) {
        let measure = parse_measure(lines, measure_count)?;
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

    assert_eq!(parse_sm("", false).unwrap(), vec![] as Vec<Vec<Note>>);

    assert!(parse_sm("#NOTES:\n", false).is_err());

    assert!(parse_sm("#NOTES:\n;\n", false).is_err());

    assert!(parse_sm("#NOTES:\n0000\n0000\n0000\n0000\n", false).is_err());

    assert_eq!(
        parse_sm("#NOTES:\n0000\n0000\n0000\n0000\n;\n", false).unwrap(),
        vec![vec![]]
    );

    assert_eq!(
        parse_sm("#NOTES:\n1000\n0000\n0000\n0000\n;\n", false).unwrap(),
        vec![vec![Note {
            pos: Pos { x: 0., y: 1. },
            time: 0.
        }]]
    );

    assert_eq!(
        parse_sm("#NOTES:\n0011\n0000\n0000\n0000\n;\n", false).unwrap(),
        vec![vec![
            Note {
                pos: Pos { x: 1., y: 2. },
                time: 0.
            },
            Note {
                pos: Pos { x: 2., y: 1. },
                time: 0.
            }
        ]]
    );

    assert_eq!(
        parse_sm("#NOTES:\n0000\n0000\n0010\n0000\n;\n", false).unwrap(),
        vec![vec![Note {
            pos: Pos { x: 1., y: 2. },
            time: 0.5
        }]]
    );

    assert_eq!(
        parse_sm(
            "#NOTES:\n0000\n0000\n0100\n0000\n,\n0000\n0010\n0000\n0000\n;\n",
            false
        )
        .unwrap(),
        vec![vec![
            Note {
                pos: Pos { x: 1., y: 0. },
                time: 0.5
            },
            Note {
                pos: Pos { x: 1., y: 2. },
                time: 1.25
            }
        ]]
    );

    assert_eq!(
        parse_sm(
            "#NOTES:\n0000\n0000\n0000\n0000\n;\n\n#NOTES:\n0000\n;",
            false
        )
        .unwrap(),
        vec![vec![], vec![]]
    );

    assert_eq!(
        parse_sm(
            "#NOTES:\n0000\n0000\n0000\n1000\n;\n\n#NOTES:\n1000\n;",
            false
        )
        .unwrap(),
        vec![
            vec![Note {
                pos: Pos { x: 0., y: 1. },
                time: 0.75
            }],
            vec![Note {
                pos: Pos { x: 0., y: 1. },
                time: 0.
            }]
        ]
    );

    assert_eq!(
        parse_sm("#NOTES:\n1234\n0000\nLFM0\n0000\n;\n", false).unwrap(),
        vec![vec![
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
        ]]
    );
}
