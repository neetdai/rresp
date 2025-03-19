use criterion::{
    black_box, criterion_group, criterion_main, Bencher, BenchmarkId, Criterion, Throughput,
};
use lexical::to_string;
use rresp::{decode, v3::V3};

fn build_bulk(len: usize) -> Vec<u8> {
    let mut buf = Vec::new();
    let len_str = to_string(len);
    buf.push(b'$');
    buf.extend_from_slice(len_str.as_bytes());
    buf.extend_from_slice(b"\r\n");
    buf.extend_from_slice(vec![b'1'; len].as_slice());
    buf.extend_from_slice(b"\r\n");
    buf
}

fn build_array<F>(len: usize, func: F) -> Vec<u8>
where
    F: Fn() -> Vec<u8>,
{
    let len_str = to_string(len);
    let mut buf = Vec::with_capacity(len);
    buf.push(b'*');
    buf.extend_from_slice(len_str.as_bytes());
    buf.extend_from_slice(b"\r\n");
    for _ in 0..len {
        let content = func();
        buf.extend_from_slice(content.as_slice());
    }
    buf
}

struct DecodeBulkParams(Vec<(Vec<u8>, usize)>);

impl DecodeBulkParams {
    fn new() -> Self {
        let params = vec![
            (build_bulk(16), 16),
            (build_bulk(1024), 1024),
            (build_bulk(10240), 10240),
            (build_bulk(102400), 102400),
        ];
        Self(params)
    }
}

struct DecodeArrayParams(Vec<(Vec<u8>, usize)>);

impl DecodeArrayParams {
    fn new() -> Self {
        let params = vec![
            (build_array(10, || build_bulk(16)), 10),
            (build_array(100, || build_bulk(16)), 100),
            (build_array(1000, || build_bulk(16)), 1000),
            (build_array(10000, || build_bulk(16)), 10000),
        ];
        Self(params)
    }
}

fn build_array_tree(len: usize) -> Vec<u8> {
    let mut buf = Vec::with_capacity(len * 4 + 3);
    for _ in 0..len {
        buf.extend_from_slice(b"*1\r\n");
    }

    buf.extend_from_slice(b"_\r\n");

    buf
}

struct DecodeArrayTreeParams(Vec<(Vec<u8>, usize)>);

impl DecodeArrayTreeParams {
    fn new() -> Self {
        let params = vec![
            (build_array_tree(10), 10),
            (build_array_tree(100), 100),
            (build_array_tree(1000), 1000),
            // (build_array_tree(10000), 10000),
        ];
        Self(params)
    }
}

fn v3_decode(c: &mut Criterion) {
    let bulk_params = DecodeBulkParams::new();
    let array_params = DecodeArrayParams::new();
    let array_tree_params = DecodeArrayTreeParams::new();

    let mut group = c.benchmark_group("v3_decode");

    for (bluk, len) in bulk_params.0 {
        group.throughput(Throughput::Elements(len as u64));
        group.bench_with_input(BenchmarkId::new("decode_bulk", len), &bluk, |b, i| {
            b.iter(|| decode::<V3>(black_box(i)).unwrap().unwrap());
        });
    }

    for (array, len) in array_params.0 {
        group.throughput(Throughput::Elements(len as u64));
        group.bench_with_input(BenchmarkId::new("decode_array", len), &array, |b, i| {
            b.iter(|| decode::<V3>(black_box(i)).unwrap().unwrap());
        });
    }

    for (array_tree, len) in array_tree_params.0 {
        group.throughput(Throughput::Elements(len as u64));
        group.bench_with_input(
            BenchmarkId::new("decode_array_tree", len),
            &array_tree,
            |b, i| {
                b.iter(|| decode::<V3>(black_box(i)).unwrap().unwrap());
            },
        );
    }
}

criterion_group!(benches, v3_decode);
criterion_main!(benches);
