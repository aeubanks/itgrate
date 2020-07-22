use criterion::{black_box, criterion_group, criterion_main, Criterion};
use itgrate::note::{Note, Pos};
use itgrate::rate::{rate_notes, state::StepParams};

pub fn benchmark_rate_notes(c: &mut Criterion) {
    let notes = (0..10000)
        .map(|i| Note {
            pos: Pos {
                x: (i as f32 / 4.).fract(),
                y: (i as f32 / 3.3 as f32).fract(),
            },
            time: i as f32,
        })
        .collect::<Vec<Note>>();
    c.bench_function("rate_notes(10000 notes)", |b| {
        b.iter(|| (rate_notes(black_box(&notes), &StepParams::default())))
    });
}

criterion_group!(benches, benchmark_rate_notes);
criterion_main!(benches);
