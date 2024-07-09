use criterion::{criterion_group, criterion_main, Criterion};
use grafton_server::add;

pub fn bench_add(c: &mut Criterion) {
    c.bench_function("add", |b| b.iter(|| add(2, 3)));
}

criterion_group!(benches, bench_add);
criterion_main!(benches);
