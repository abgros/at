use at::At;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn at_index(s: &[u64], idx: usize) {
	black_box(s.at(idx));
}

fn at_index_i64(s: &[u64], idx: i64) {
	black_box(s.at(idx));
}

fn at_index_i128(s: &[u64], idx: i128) {
	black_box(s.at(idx));
}

fn normal_index(s: &[u64], idx: usize) {
	black_box(s[idx]);
}

fn criterion_benchmark(c: &mut Criterion) {
	let s = &[1, 2, 3];

	c.bench_function("at_index", |b| {
		b.iter(|| at_index(black_box(s), black_box(2)))
	});

	c.bench_function("at_index_i64", |b| {
		b.iter(|| at_index_i64(black_box(s), black_box(2)))
	});

	c.bench_function("at_index_i128", |b| {
		b.iter(|| at_index_i128(black_box(s), black_box(2)))
	});

	c.bench_function("normal_index", |b| {
		b.iter(|| normal_index(black_box(s), black_box(2)))
	});
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
