

#[derive(Debug, PartialEq)]
pub enum Frame<'a> {
    SimpleString {
        data: &'a [u8],
    },
    SimpleError {
        data: &'a [u8],
    },
    Boolean {
        data: bool,
    },
    Null {
        data: (),
    },
    Integer {
        data: isize,
    },
    Double {
        data: f64,
    },
    Bulkstring {
        data: &'a [u8],
    },
    VerbatimString {
        data: ([u8; 3], &'a [u8]),
    }
}