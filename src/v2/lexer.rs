use memchr::memmem::{FindIter, Finder, FinderBuilder, Prefilter};

use super::{ast::Ast, tag::Tag, utils::CRLF};


#[derive(Debug)]
pub(crate) struct Lexer<'a> {
    input: &'a [u8],
    scanner: FindIter<'a, 'static>,
    last_position: usize,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(input: &'a [u8]) -> Self {
        let mut builder = FinderBuilder::new();
        builder.prefilter(Prefilter::Auto);
        let finder = builder.build_forward(&CRLF);
        let scanner = finder.find_iter(input).into_owned();

        Self { input, scanner, last_position: 0, }
    }

    fn match_ast(split: &'a [u8]) -> Option<Tag<'a>> {
        let (first, follow) = split.split_first()?;
        match first {
            b'+' => Some(Tag::SimpleString(follow)),
            b'-' => Some(Tag::SimpleError(follow)),
            _ => Some(Tag::UnKnown),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Tag<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let end_position = self.scanner.next()?;
        let split = self.input.get(self.last_position..end_position)?;
        self.last_position = end_position + 2; // +2 to skip the CRLF

        Self::match_ast(split)
    }
}

mod test {
    use super::*;

    #[test]
    fn test_simple_string() {
        let input = b"+hello\r\n+world\r\n";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next().unwrap(), Tag::SimpleString(b"hello"));
        assert_eq!(lexer.next().unwrap(), Tag::SimpleString(b"world"));
    }

    #[test]
    fn test_simple_error() {
        let input = b"-error1\r\n-error2\r\n";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next().unwrap(), Tag::SimpleError(b"error1"));
        assert_eq!(lexer.next().unwrap(), Tag::SimpleError(b"error2"));
    }

    #[test]
    fn test_simple_unknown() {
        let input = b"unknown\r\n";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next().unwrap(), Tag::UnKnown);
    }
}