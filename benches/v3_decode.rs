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
        let lens = [16usize, 1024, 10240, 102400];

        let mut params = Vec::with_capacity(lens.len());
        for &len in lens.iter() {
            let p = build_bulk(len);
            let p_len = p.len();
            params.push((p, p_len));
        }
        Self(params)
    }
}

struct DecodeArrayParams(Vec<(Vec<u8>, usize)>);

impl DecodeArrayParams {
    fn new() -> Self {
        let lens = [10usize, 100, 1000, 10000];
        let mut params = Vec::with_capacity(lens.len());
        for &len in lens.iter() {
            let p = build_array(len, || build_bulk(16));
            let p_len = p.len();
            params.push((p, p_len));
        }
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
        let lens = [10usize, 100, 1000];
        let mut params = Vec::with_capacity(lens.len());
        for &len in lens.iter() {
            let p = build_array_tree(len);
            let p_len = p.len();
            params.push((p, p_len));
        }
        Self(params)
    }
}

fn build_attribute(len: usize) -> Vec<u8> {
    let len_str = to_string(len);
    let mut buf = Vec::new();
    buf.extend_from_slice(b"|");
    buf.extend_from_slice(len_str.as_bytes());
    buf.extend_from_slice(b"\r\n");

    for index in 0..len {
        let key = to_string(index);
        buf.extend_from_slice(b"+");
        buf.extend_from_slice(key.as_bytes());
        buf.extend_from_slice(b"\r\n");

        buf.extend_from_slice(b"$5\r\nhello\r\n");
    }
    buf.extend_from_slice(b"$3\r\nend\r\n");

    buf
}

struct DecodeAttributeParams(Vec<(Vec<u8>, usize)>);

impl DecodeAttributeParams {
    fn new() -> Self {
        let lens = [10usize, 100, 1000];
        let mut params = Vec::with_capacity(lens.len());
        for &len in lens.iter() {
            let p = build_attribute(len);
            let p_len = p.len();
            params.push((p, p_len));
        }
        Self(params)
    }
}

fn v3_decode(c: &mut Criterion) {
    let bulk_params = DecodeBulkParams::new();
    let array_params = DecodeArrayParams::new();
    let array_tree_params = DecodeArrayTreeParams::new();
    let attribute_params = DecodeAttributeParams::new();

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

    for (attribute, len) in attribute_params.0 {
        group.throughput(Throughput::Elements(len as u64));
        group.bench_with_input(
            BenchmarkId::new("decode_attribute", len),
            &attribute,
            |b, i| {
                b.iter(|| decode::<V3>(black_box(i)).unwrap().unwrap());
            },
        );
    }
}

criterion_group!(benches, v3_decode);
criterion_main!(benches);
