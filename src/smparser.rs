use crate::note::Note;
use crate::note_pos::col_to_pos;
use std::option::Option;

struct Lines {
    idx: usize,
    lines: Vec<String>,
}

impl Lines {
    fn new(s: &str) -> Self {
        Self {
            idx: 0,
            lines: s.lines().map(|s| s.to_owned()).collect(),
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
            Some(&self.lines[self.idx])
        } else {
            None
        }
    }
    fn consume(&mut self) -> Option<&str> {
        self.skip_bad_lines();
        if self.is_valid() {
            let ret = &self.lines[self.idx];
            self.idx += 1;
            Some(ret)
        } else {
            None
        }
    }
}

#[test]
fn test_lines() {
    let mut lines1 = Lines::new("");
    assert_eq!(lines1.peek(), None);
    assert_eq!(lines1.consume(), None);

    let mut lines2 = Lines::new("a");
    assert_eq!(lines2.peek(), Some("a"));
    assert_eq!(lines2.consume(), Some("a"));
    assert_eq!(lines2.peek(), None);
    assert_eq!(lines2.consume(), None);

    let mut lines3 = Lines::new("a\nbas\nqwe");
    assert_eq!(lines3.peek(), Some("a"));
    assert_eq!(lines3.consume(), Some("a"));
    assert_eq!(lines3.peek(), Some("bas"));
    assert_eq!(lines3.consume(), Some("bas"));
    assert_eq!(lines3.peek(), Some("qwe"));
    assert_eq!(lines3.consume(), Some("qwe"));
    assert_eq!(lines3.peek(), None);
    assert_eq!(lines3.consume(), None);

    let mut lines4 = Lines::new("a\n\n//\nb");
    assert_eq!(lines4.peek(), Some("a"));
    assert_eq!(lines4.consume(), Some("a"));
    assert_eq!(lines4.peek(), Some("b"));
    assert_eq!(lines4.consume(), Some("b"));
    assert_eq!(lines4.peek(), None);
    assert_eq!(lines4.consume(), None);

    let mut lines4 = Lines::new("\n\n//\n");
    assert_eq!(lines4.peek(), None);
    assert_eq!(lines4.consume(), None);
}

pub fn parse_sm(s: &str) -> Option<Vec<Vec<Note>>> {
    let mut ret = Vec::new();
    let mut lines = Lines::new(s);

    while let Some(l) = lines.consume() {
        if l == "#NOTES:" {
            while let Some(p) = lines.peek() {
                if !p.ends_with(":") {
                    break;
                }
                lines.consume();
            }
            let notes = parse_notes(&mut lines)?;
            ret.push(notes);
        } else if line_is_notes(l) {
            return None;
        }
    }

    Some(ret)
}

fn line_is_notes(l: &str) -> bool {
    l.chars().all(|c| c == '0' || c == '1' || c == '2')
}

fn parse_measure(lines: &mut Lines, measure_count: i32) -> Option<Vec<Note>> {
    let mut ret = Vec::new();
    let mut measure = Vec::new();
    loop {
        if !line_is_notes(lines.peek()?) {
            break;
        }
        measure.push(lines.consume()?.to_owned());
    }
    let measure_line_count = measure.len();
    for (line_idx, l) in measure.iter().enumerate() {
        let time = line_idx as f32 / measure_line_count as f32 + measure_count as f32;
        let num_cols = l.len();
        for (col, c) in l.chars().enumerate() {
            if c == '1' || c == '2' {
                ret.push(Note {
                    pos: col_to_pos(col, num_cols),
                    time,
                })
            }
        }
    }
    Some(ret)
}

fn parse_notes(lines: &mut Lines) -> Option<Vec<Note>> {
    let mut ret = Vec::new();
    let mut measure_count = 0;
    if !line_is_notes(lines.peek()?) {
        return None;
    }
    while line_is_notes(lines.peek()?) {
        let measure = parse_measure(lines, measure_count)?;
        ret.extend(measure);
        let separator = lines.consume()?;
        if separator == ";" {
            break;
        } else if separator != "," {
            return None;
        }
        measure_count += 1;
    }
    Some(ret)
}

#[test]
fn test_parse_sm() {
    use crate::note::Pos;

    assert_eq!(parse_sm(""), Some(vec![]));

    assert_eq!(parse_sm("#NOTES:\n"), None);

    assert_eq!(parse_sm("#NOTES:\n;\n"), None);

    assert_eq!(parse_sm("#NOTES:\n0000\n0000\n0000\n0000\n"), None);

    assert_eq!(
        parse_sm("#NOTES:\n0000\n0000\n0000\n0000\n;\n"),
        Some(vec![vec![]])
    );

    assert_eq!(
        parse_sm("#NOTES:\n1000\n0000\n0000\n0000\n;\n"),
        Some(vec![vec![Note {
            pos: Pos { x: 0., y: 1. },
            time: 0.
        }]])
    );

    assert_eq!(
        parse_sm("#NOTES:\n0011\n0000\n0000\n0000\n;\n"),
        Some(vec![vec![
            Note {
                pos: Pos { x: 1., y: 2. },
                time: 0.
            },
            Note {
                pos: Pos { x: 2., y: 1. },
                time: 0.
            }
        ]])
    );

    assert_eq!(
        parse_sm("#NOTES:\n0000\n0000\n0010\n0000\n;\n"),
        Some(vec![vec![Note {
            pos: Pos { x: 1., y: 2. },
            time: 0.5
        }]])
    );

    assert_eq!(
        parse_sm("#NOTES:\n0000\n0000\n0100\n0000\n,\n0000\n0010\n0000\n0000\n;\n"),
        Some(vec![vec![
            Note {
                pos: Pos { x: 1., y: 0. },
                time: 0.5
            },
            Note {
                pos: Pos { x: 1., y: 2. },
                time: 1.25
            }
        ]])
    );

    assert_eq!(
        parse_sm("#NOTES:\n0000\n0000\n0000\n0000\n;\n\n#NOTES:\n0000\n;"),
        Some(vec![vec![], vec![]])
    );

    assert_eq!(
        parse_sm("#NOTES:\n0000\n0000\n0000\n1000\n;\n\n#NOTES:\n1000\n;"),
        Some(vec![
            vec![Note {
                pos: Pos { x: 0., y: 1. },
                time: 0.75
            }],
            vec![Note {
                pos: Pos { x: 0., y: 1. },
                time: 0.
            }]
        ])
    );
}
