# rresp

## A Rust-Based Library for Redis Protocol Encoding/Decoding

This is an open-source library implemented in Rust for encoding and decoding the Redis protocol, supporting both RESP2 and RESP3 versions. Designed for high performance, it aims to deliver fast parsing with high throughput.

v2 protocol
```
bulk string benchmark

v2_decode/decode_bulk/16
    time:   [41.239 ns 41.276 ns 41.334 ns]
    thrpt:  [387.09 Melem/s 387.64 Melem/s 387.99 Melem/s]

v2_decode/decode_bulk/1024
    time:   [50.900 ns 50.976 ns 51.083 ns]
    thrpt:  [20.046 Gelem/s 20.088 Gelem/s 20.118 Gelem/s]

v2_decode/decode_bulk/10240
    time:   [115.36 ns 115.93 ns 116.62 ns]
    thrpt:  [87.808 Gelem/s 88.333 Gelem/s 88.768 Gelem/s]

v2_decode/decode_bulk/10240
    time:   [1.0466 µs 1.0472 µs 1.0478 µs]
    thrpt:  [97.726 Gelem/s 97.788 Gelem/s 97.836 Gelem/s]

v2_decode/decode_array/10
    time:   [304.71 ns 305.00 ns 305.34 ns]
    thrpt:  [32.751 Melem/s 32.787 Melem/s 32.818 Melem/s]

v2_decode/decode_array/100
    time:   [2.4564 µs 2.4584 µs 2.4609 µs]
    thrpt:  [40.636 Melem/s 40.677 Melem/s 40.709 Melem/s]

v2_decode/decode_array/1000
    time:   [23.651 µs 23.677 µs 23.708 µs]
    thrpt:  [42.179 Melem/s 42.236 Melem/s 42.282 Melem/s]

v2_decode/decode_array/10000
    time:   [237.05 µs 238.08 µs 240.00 µs]
    thrpt:  [41.667 Melem/s 42.003 Melem/s 42.186 Melem/s]

v2_decode/decode_array_half_null/10
    time:   [245.03 ns 245.60 ns 246.25 ns]
    thrpt:  [40.610 Melem/s 40.716 Melem/s 40.812 Melem/s]

v2_decode/decode_array_half_null/100
    time:   [2.0857 µs 2.0880 µs 2.0909 µs]
    thrpt:  [47.826 Melem/s 47.892 Melem/s 47.946 Melem/s]

v2_decode/decode_array_half_null/1000
    time:   [22.100 µs 22.241 µs 22.400 µs]
    thrpt:  [44.642 Melem/s 44.961 Melem/s 45.248 Melem/s]

v3_decode/decode_bulk/23
    time:   [107.31 ns 107.43 ns 107.56 ns]
    thrpt:  [213.83 Melem/s 214.09 Melem/s 214.33 Melem/s]

v3_decode/decode_bulk/1033
    time:   [116.18 ns 116.32 ns 116.48 ns]
    thrpt:  [8.8684 Gelem/s 8.8805 Gelem/s 8.8916 Gelem/s]

v3_decode/decode_bulk/10250
    time:   [166.88 ns 167.08 ns 167.31 ns]
    thrpt:  [61.263 Gelem/s 61.349 Gelem/s 61.420 Gelem/s]

v3_decode/decode_bulk/102411
    time:   [1.1139 µs 1.1146 µs 1.1155 µs]
    thrpt:  [91.809 Gelem/s 91.884 Gelem/s 91.943 Gelem/s]

v3_decode/decode_array/235
    time:   [458.26 ns 458.95 ns 459.64 ns]
    thrpt:  [511.27 Melem/s 512.04 Melem/s 512.81 Melem/s]

v3_decode/decode_array/2306
    time:   [2.9123 µs 2.9174 µs 2.9235 µs]
    thrpt:  [788.78 Melem/s 790.43 Melem/s 791.81 Melem/s]

v3_decode/decode_array/23007
    time:   [27.834 µs 27.886 µs 27.947 µs]
    thrpt:  [823.23 Melem/s 825.04 Melem/s 826.57 Melem/s]

v3_decode/decode_array/230008
    time:   [283.38 µs 284.61 µs 285.77 µs]
    thrpt:  [804.88 Melem/s 808.16 Melem/s 811.67 Melem/s]

v3_decode/decode_array_tree/43
    time:   [721.89 ns 722.51 ns 723.31 ns]
    thrpt:  [59.449 Melem/s 59.515 Melem/s 59.566 Melem/s]

v3_decode/decode_array_tree/403
    time:   [6.3135 µs 6.3177 µs 6.3236 µs]
    thrpt:  [63.729 Melem/s 63.789 Melem/s 63.832 Melem/s]

v3_decode/decode_array_tree/4003
    time:   [62.869 µs 62.938 µs 63.020 µs]
    thrpt:  [63.520 Melem/s 63.602 Melem/s 63.672 Melem/s]

v3_decode/decode_attribute/164
    time:   [1.0596 µs 1.0614 µs 1.0650 µs]
    thrpt:  [153.99 Melem/s 154.51 Melem/s 154.77 Melem/s]

v3_decode/decode_attribute/1605
    time:   [8.7148 µs 8.7214 µs 8.7305 µs]
    thrpt:  [183.84 Melem/s 184.03 Melem/s 184.17 Melem/s]

v3_decode/decode_attribute/16906
    time:   [87.753 µs 87.874 µs 88.039 µs]
    thrpt:  [192.03 Melem/s 192.39 Melem/s 192.65 Melem/s]
```
