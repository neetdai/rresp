use criterion::{
    black_box, criterion_group, criterion_main, Bencher, BenchmarkId, Criterion, Throughput,
};
use lexical::to_string;
use rand::random;
use rresp::{
    encode,
    v3::{Frame, V3},
    EncodeLen,
};
use minivec::mini_vec;

fn v3_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("v3_encode");

    group.throughput(Throughput::Elements(11));
    group.bench_function("bulk_string", |b| {
        b.iter(|| {
            let frame = Frame::Bulkstring {
                data: b"hello",
                attributes: None,
            };
            encode::<V3>(black_box(frame)).unwrap();
        });
    });

    let frame = Frame::Array {
        data: mini_vec![Frame::Bulkstring {
            data: b"hello",
            attributes: None,
        }],
        attributes: None,
    };
    group.throughput(Throughput::Elements(frame.encode_len() as u64));
    group.bench_function("array", |b| {
        b.iter(|| {
            let frame = Frame::Array {
                data: mini_vec![Frame::Bulkstring {
                    data: b"hello",
                    attributes: None,
                }],
                attributes: None,
            };
            encode::<V3>(black_box(frame)).unwrap();
        });
    });

    let frame = Frame::Array {
        data: mini_vec![Frame::Array {
            data: mini_vec![Frame::Bulkstring {
                data: b"hello",
                attributes: None,
            }],
            attributes: None,
        }],
        attributes: None,
    };
    group.throughput(Throughput::Elements(frame.encode_len() as u64));
    group.bench_function("array2", |b| {
        b.iter(|| {
            let frame = Frame::Array {
                data: mini_vec![Frame::Array {
                    data: mini_vec![Frame::Bulkstring {
                        data: b"hello",
                        attributes: None,
                    }],
                    attributes: None,
                }],
                attributes: None,
            };
            encode::<V3>(black_box(frame)).unwrap();
        });
    });
}

criterion_group!(benches, v3_encode);
criterion_main!(benches);
