use criterion::{
    black_box, criterion_group, criterion_main, Bencher, BenchmarkId, Criterion, Throughput,
};
use lexical::to_string;
use rand::random;
use rresp::{decode, v2::V2};

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

fn build_null() -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(b"$-1\r\n");
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

struct DecodeArrayHalfNullParams(Vec<(Vec<u8>, usize)>);

impl DecodeArrayHalfNullParams {
    fn new() -> Self {
        let params = vec![
            (
                build_array(10, || {
                    let is_null = random::<bool>();
                    if is_null {
                        build_null()
                    } else {
                        build_bulk(16)
                    }
                }),
                10,
            ),
            (
                build_array(100, || {
                    let is_null = random::<bool>();
                    if is_null {
                        build_null()
                    } else {
                        build_bulk(16)
                    }
                }),
                100,
            ),
            (
                build_array(1000, || {
                    let is_null = random::<bool>();
                    if is_null {
                        build_null()
                    } else {
                        build_bulk(16)
                    }
                }),
                1000,
            ),
            (
                build_array(10000, || {
                    let is_null = random::<bool>();
                    if is_null {
                        build_null()
                    } else {
                        build_bulk(16)
                    }
                }),
                10000,
            ),
        ];
        Self(params)
    }
}

fn v2_decode(c: &mut Criterion) {
    let bulk_params = DecodeBulkParams::new();
    let array_params = DecodeArrayParams::new();
    let array_half_null_params = DecodeArrayHalfNullParams::new();
    let mut group = c.benchmark_group("v2_decode");

    for (bulk, len) in bulk_params.0 {
        group.throughput(Throughput::Elements(len as u64));
        group.bench_with_input(BenchmarkId::new("decode_bulk", len), &bulk, |b, i| {
            b.iter(|| decode::<V2>(black_box(i)).unwrap().unwrap());
        });
    }

    for (array, len) in array_params.0 {
        group.throughput(Throughput::Elements(len as u64));
        group.bench_with_input(BenchmarkId::new("decode_array", len), &array, |b, i| {
            b.iter(|| decode::<V2>(black_box(i)).unwrap().unwrap());
        });
    }

    for (array, len) in array_half_null_params.0 {
        group.throughput(Throughput::Elements(len as u64));
        group.bench_with_input(
            BenchmarkId::new("decode_array_half_null", len),
            &array,
            |b, i| {
                b.iter(|| decode::<V2>(black_box(i)).unwrap().unwrap());
            },
        );
    }
}

criterion_group!(benches, v2_decode);
criterion_main!(benches);
