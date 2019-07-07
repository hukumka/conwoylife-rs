#[macro_use]
extern crate criterion;

extern crate life;

fn bench(b: &mut criterion::Criterion) {
    let inputs: Vec<_> = (1..21).map(|i| i*i*10000).collect();
    b.bench_function_over_inputs("life", |b, &size| {
        let dim = (size as f64).sqrt() as u32;
        let mut l = life::Life::new_random(dim, dim);
        b.iter(|| {
            l.update();
            criterion::black_box(l.value());
        })
    }, inputs);
}

criterion_group!(benches, bench);
criterion_main!(benches);
