use criterion::{black_box, criterion_group, criterion_main, Bencher, BenchmarkId, Criterion, Throughput};
use lexical::to_string;
use rresp::{decode, V2};

struct DecodeBlobParams(Vec<(Vec<u8>, usize)>);

impl DecodeBlobParams {
    fn new() -> Self {
        let params = vec![
            (Self::build_blob(16), 16),
            (Self::build_blob(1024), 1024),
            (Self::build_blob(10240), 10240),
            (Self::build_blob(102400), 102400),
        ];
        Self(params)
    }

    fn build_blob(len: usize) -> Vec<u8> {
        let mut buf = Vec::new();
        let len_str = to_string(len);
        buf.push(b'$');
        buf.extend_from_slice(len_str.as_bytes());
        buf.extend_from_slice(b"\r\n");
        buf.extend_from_slice(vec![b'1'; len].as_slice());
        buf.extend_from_slice(b"\r\n");
        buf
    }
}

fn v2_decode(c: &mut Criterion) {
    let blob_params = DecodeBlobParams::new();
    let mut group = c.benchmark_group("v2_decode");
    
    for (blob, len) in blob_params.0 {
        group.throughput(Throughput::Elements(len as u64));
        group.bench_with_input(BenchmarkId::new("decode_blob", len), &blob, |b, i| {
            b.iter(|| decode::<V2>(black_box(i)).unwrap().unwrap());
        });
    }
}

criterion_group!(benches, v2_decode);
criterion_main!(benches);