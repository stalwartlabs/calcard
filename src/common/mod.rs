pub mod parser;
pub mod tokenizer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Encoding {
    QuotedPrintable,
    Base64,
}

impl Encoding {
    pub fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            b"QUOTED-PRINTABLE" => Encoding::QuotedPrintable,
            b"BASE64" => Encoding::Base64,
            b"Q" => Encoding::QuotedPrintable,
            b"B" => Encoding::Base64,
        )
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Data {
    pub content_type: Option<String>,
    pub data: Vec<u8>,
}
