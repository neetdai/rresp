

#[derive(Debug, PartialEq)]
pub enum Frame<'a> {
    SimpleString {
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
    }
}