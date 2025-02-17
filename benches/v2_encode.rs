use criterion::{
    black_box, criterion_group, criterion_main, Bencher, BenchmarkId, Criterion, Throughput,
};
use lexical::to_string;
use rand::random;
use rresp::{encode, Frame, V2};

fn v2_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("v2_encode");

    group.throughput(Throughput::Elements(18));
    group.bench_function(BenchmarkId::new("encode_blob", 18),  |b| {
        b.iter(|| {
            let blob_frame = Frame::BlobString(b"hello world");
            encode::<V2>(black_box(blob_frame)).unwrap();
        });
    });
}

criterion_group!(benches, v2_encode);
criterion_main!(benches);
