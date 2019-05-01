#[macro_use]
extern crate criterion;

extern crate life;

fn bench(b: &mut criterion::Criterion){
    b.bench_function("life", |b|{
        let mut l = life::Life::new(200, 200);
        b.iter(|| {
            l.update();
            criterion::black_box(l.value());
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);