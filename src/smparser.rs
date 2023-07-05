use std::collections::HashMap;

#[derive(PartialEq, Debug)]
pub struct Note {
    pub time: f32,
}

pub struct Chart {
    pub title: String,
    pub difficulty: String,
    pub notes: Vec<Note>,
    pub rating: i32,
}

fn parse_msd(buf: &str) -> Option<HashMap<String, Vec<String>>> {
    let mut map = HashMap::<String, Vec<String>>::new();
    let mut str = String::new();
    for mut line in buf.lines() {
        if let Some(comment) = line.find("//") {
            line = &line[0..comment];
        }
        assert!(!line.contains('\\'), "found \\, can't handle escaping yet");
        str.push_str(line);
        str.push('\n');
    }
    if !str.is_empty() {
        str.pop();
    }

    for split in str.split(';') {
        let split = split.trim_start();
        if split.is_empty() {
            continue;
        }
        let c = split.chars().next().unwrap();
        if c != '#' {
            println!("No #: {}", split);
            return None;
        }
        let colon = match split.find(':') {
            Some(c) => c,
            None => {
                println!("No :: {}", split);
                return None;
            }
        };

        let key = &split[1..colon];
        let val = &split[(colon + 1)..split.len()];

        map.entry(key.to_owned()).or_default().push(val.to_owned());
    }
    Some(map)
}

#[test]
fn test_parse_msd() {
    assert_eq!(parse_msd(""), Some(HashMap::new()));
    assert_eq!(
        parse_msd("#A:"),
        Some(HashMap::from([("A".into(), vec!["".into()])]))
    );
    assert_eq!(
        parse_msd("// whoa\n#A:"),
        Some(HashMap::from([("A".into(), vec!["".into()])]))
    );
    assert_eq!(
        parse_msd("  // whoa\n#A:"),
        Some(HashMap::from([("A".into(), vec!["".into()])]))
    );
    assert_eq!(
        parse_msd("  // whoa\n#A:// hello\n"),
        Some(HashMap::from([("A".into(), vec!["".into()])]))
    );
    assert_eq!(
        parse_msd("\n#A:"),
        Some(HashMap::from([("A".into(), vec!["".into()])]))
    );
    assert_eq!(
        parse_msd("#A:;"),
        Some(HashMap::from([("A".into(), vec!["".into()])]))
    );
    assert_eq!(
        parse_msd("#A:\n;"),
        Some(HashMap::from([("A".into(), vec!["\n".into()])]))
    );
    assert_eq!(parse_msd("#A"), None);
    assert_eq!(parse_msd("#A;"), None);
    assert_eq!(parse_msd("A:;"), None);
    assert_eq!(
        parse_msd("#A:;#B:asdf;#CC:  hihi whoa "),
        Some(HashMap::from([
            ("A".into(), vec!["".into()]),
            ("B".into(), vec!["asdf".into()]),
            ("CC".into(), vec!["  hihi whoa ".into()])
        ]))
    );
    assert_eq!(
        parse_msd("#A:\n;#B:\nasdf\n;"),
        Some(HashMap::from([
            ("A".into(), vec!["\n".into()]),
            ("B".into(), vec!["\nasdf\n".into()]),
        ]))
    );
    assert_eq!(
        parse_msd("#A:;#A:asdf;"),
        Some(HashMap::from([(
            "A".into(),
            vec!["".into(), "asdf".into()]
        ),]))
    );
    assert_eq!(
        parse_msd("#A:;#B:hi;#A:asdf;"),
        Some(HashMap::from([
            ("A".into(), vec!["".into(), "asdf".into()]),
            ("B".into(), vec!["hi".into()]),
        ]))
    );
}

fn split_notes(buf: &str) -> Option<(String, String, i32, String)> {
    let mut split = buf.split(':');
    let style = split.next()?.trim().to_owned();
    split.next()?;
    let difficulty = split.next()?.to_owned();
    let rating = match split.next()?.trim().parse::<i32>() {
        Ok(r) => r,
        Err(_) => {
            return None;
        }
    };
    split.next()?;
    let steps = split.next()?.to_owned();
    if split.next().is_some() {
        return None;
    }

    Some((style, difficulty, rating, steps))
}

#[test]
fn test_split_notes() {
    assert_eq!(split_notes(""), None);
    assert_eq!(
        split_notes("a:b:c:0:d:e"),
        Some(("a".into(), "c".into(), 0, "e".into()))
    );
    assert_eq!(
        split_notes("a:b:c: 2 :d:e"),
        Some(("a".into(), "c".into(), 2, "e".into()))
    );
    assert_eq!(split_notes("a:b:c:0:d"), None);
    assert_eq!(split_notes("a:b:c:z:d:e"), None);
}

#[derive(Default, PartialEq, Debug)]
struct BPMs {
    // (beat, bpm)
    bpm_changes: Vec<(f32, f32)>,
    // TODO: stops
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

fn parse_bpms(buf: &str) -> Option<BPMs> {
    let mut bpms = BPMs::default();
    for change in buf.split(',') {
        let change = change.trim();
        let equal = change.find('=')?;

        let time = match change[0..equal].parse::<f32>() {
            Ok(c) => c,
            Err(_) => {
                return None;
            }
        };
        let bpm = match change[(equal + 1)..change.len()].parse::<f32>() {
            Ok(c) => c,
            Err(_) => {
                return None;
            }
        };
        bpms.bpm_changes.push((time, bpm));
    }
    if bpms.bpm_changes.is_empty() {
        return None;
    }
    Some(bpms)
}

#[test]
fn test_parse_bpms() {
    assert_eq!(parse_bpms(""), None);
    assert_eq!(
        parse_bpms("0=2"),
        Some(BPMs {
            bpm_changes: vec![(0., 2.)]
        })
    );
    assert_eq!(
        parse_bpms(" 0=2  "),
        Some(BPMs {
            bpm_changes: vec![(0., 2.)]
        })
    );
    assert_eq!(
        parse_bpms("0=2,4.0=8.0"),
        Some(BPMs {
            bpm_changes: vec![(0., 2.), (4., 8.)]
        })
    );
    assert_eq!(parse_bpms("0=2:4.0=8.0"), None);
}

fn parse_steps(buf: &str, bpms: &BPMs) -> Option<Vec<Note>> {
    let mut steps = Vec::new();

    for (measure_num, measure) in buf.split(',').enumerate() {
        let lines = measure
            .lines()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        let measures_per_line = 1.0 / lines.len() as f32;
        for (line_num, line) in lines.iter().enumerate() {
            for c in line.trim().chars() {
                match c {
                    '1' | '2' | '4' => {
                        let cur_measure = line_num as f32 * measures_per_line + measure_num as f32;
                        steps.push(Note {
                            time: bpms.measure_to_time(cur_measure),
                        });
                    }
                    '0' | '3' | 'M' | 'L' | 'F' => {}
                    a => {
                        println!("Unexpected {} in {}", a, line);
                        return None;
                    }
                }
            }
        }
    }

    Some(steps)
}

#[test]
fn test_parse_steps() {
    let bpms = BPMs {
        bpm_changes: vec![(0., 60.), (8., 120.)],
    };
    assert_eq!(parse_steps("", &bpms), Some(vec![]));
    assert_eq!(parse_steps("00", &bpms), Some(vec![]));
    assert_eq!(parse_steps(" 00  ", &bpms), Some(vec![]));
    assert_eq!(parse_steps("3M", &bpms), Some(vec![]));
    assert_eq!(parse_steps("05", &bpms), None);
    assert_eq!(parse_steps("10", &bpms), Some(vec![Note { time: 0. }]));
    assert_eq!(parse_steps("00\n10", &bpms), Some(vec![Note { time: 2. }]));
    assert_eq!(
        parse_steps("11", &bpms),
        Some(vec![Note { time: 0. }, Note { time: 0. }])
    );
    assert_eq!(
        parse_steps("00\n,\n10", &bpms),
        Some(vec![Note { time: 4. }])
    );
    assert_eq!(
        parse_steps("00\n,\n10\n10", &bpms),
        Some(vec![Note { time: 4. }, Note { time: 6. }])
    );
    assert_eq!(
        parse_steps("00\n,\n10\n,\n10\n,\n10\n", &bpms),
        Some(vec![
            Note { time: 4. },
            Note { time: 8. },
            Note { time: 10. }
        ])
    );
}

pub fn parse(buf: &str) -> Vec<Chart> {
    let mut charts = Vec::new();
    // Strip BOM
    let buf = buf.trim_start_matches('\u{feff}');
    let msd = parse_msd(buf).unwrap();
    if let Some(stops) = msd.get("STOPS") {
        for stop in stops {
            if !stop.trim().is_empty() {
                return vec![];
            }
        }
    }
    let title = &msd.get("TITLE").unwrap()[0];
    let bpms = {
        let bpms = &msd.get("BPMS").unwrap();
        assert_eq!(bpms.len(), 1);
        parse_bpms(&bpms[0]).unwrap()
    };
    if let Some(all_notes) = msd.get("NOTES") {
        for notes in all_notes {
            let (style, difficulty, rating, steps) = split_notes(notes).unwrap();
            if style != "dance-single" {
                continue;
            }
            charts.push(Chart {
                title: title.clone(),
                difficulty,
                notes: parse_steps(&steps, &bpms).unwrap(),
                rating,
            });
        }
    }
    charts
}
