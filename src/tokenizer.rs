use std::borrow::Cow;

use crate::parser::Parser;

pub(crate) struct Token<'x> {
    pub(crate) text: Cow<'x, [u8]>,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) stop_char: StopChar,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum StopChar {
    Colon,
    Semicolon,
    Comma,
    Equal,
    Slash,
    Dot,
    Lf,
}

impl<'x> Parser<'x> {
    pub(crate) fn expect_iana_token(&mut self) {
        self.stop_colon = true;
        self.stop_semicolon = true;
        self.stop_comma = true;
        self.stop_equal = true;
        self.stop_slash = false;
        self.unfold_qp = false;
        self.unquote = true;
    }

    pub(crate) fn expect_qp_value(&mut self) {
        self.stop_colon = false;
        self.stop_comma = false;
        self.stop_equal = false;
        self.stop_slash = false;
        self.unquote = false;
        self.unfold_qp = true;
        self.stop_semicolon = true;
    }

    pub(crate) fn expect_single_value(&mut self) {
        self.stop_colon = false;
        self.stop_comma = false;
        self.stop_equal = false;
        self.stop_slash = false;
        self.stop_semicolon = false;
        self.unquote = false;
        self.unfold_qp = false;
    }

    pub(crate) fn expect_multi_value_comma(&mut self) {
        self.stop_colon = false;
        self.stop_comma = true;
        self.stop_equal = false;
        self.stop_slash = false;
        self.stop_semicolon = false;
        self.unquote = false;
        self.unfold_qp = false;
    }

    pub(crate) fn expect_multi_value_semicolon(&mut self) {
        self.stop_colon = false;
        self.stop_comma = false;
        self.stop_equal = false;
        self.stop_slash = false;
        self.stop_semicolon = true;
        self.unquote = false;
        self.unfold_qp = false;
    }

    pub(crate) fn expect_param_value(&mut self) {
        self.stop_colon = true;
        self.stop_semicolon = true;
        self.stop_comma = true;
        self.stop_equal = false;
        self.stop_slash = false;
        self.unfold_qp = false;
        self.unquote = true;
    }

    fn try_unfold(&mut self) -> bool {
        if let Some((_, next)) = self.iter.peek() {
            if **next == b' ' || **next == b'\t' {
                self.iter.next();
                return true;
            }
        }
        false
    }

    pub(crate) fn token(&mut self) -> Option<Token<'x>> {
        let mut offset_start = usize::MAX;
        let mut offset_end = usize::MAX;
        let mut in_quote = false;
        let stop_char;
        let mut buf: Vec<u8> = vec![];

        'outer: loop {
            let (idx, ch) = if let Some(next) = self.iter.next() {
                next
            } else if offset_start != usize::MAX {
                stop_char = StopChar::Lf;
                break;
            } else {
                return None;
            };

            match ch {
                b' ' | b'\t' => {
                    if in_quote || buf.last().is_some_and(|ch| !ch.is_ascii_whitespace()) {
                        if offset_start == usize::MAX {
                            offset_start = idx;
                        }
                        offset_end = idx;

                        if !buf.is_empty() {
                            buf.push(*ch);
                        }
                    }
                }
                b'\r' => {}
                b'\n' => {
                    if self.unfold_qp
                        && buf.last().or_else(|| self.input.get(offset_end)).copied() == Some(b'=')
                    {
                        offset_end = idx;

                        if !buf.is_empty() {
                            buf.push(*ch);
                        }
                    } else if self.try_unfold() {
                        if buf.is_empty() && offset_start != usize::MAX {
                            buf.extend_from_slice(&self.input[offset_start..=offset_end]);
                        }
                    } else if offset_start != usize::MAX {
                        stop_char = StopChar::Lf;
                        break;
                    }
                }
                b'\\' => {
                    let mut next_ch = b'\\';
                    let mut next_offset_end = idx;
                    while let Some((idx, ch)) = self.iter.next() {
                        match ch {
                            b' ' | b'\t' | b'\r' => {}
                            b'\n' => {
                                if self.try_unfold() {
                                    if let Some((idx, ch)) = self.iter.next() {
                                        next_ch = *ch;
                                        next_offset_end = idx;
                                        break;
                                    }
                                } else {
                                    stop_char = StopChar::Lf;
                                    offset_end = idx - 1;
                                    break 'outer;
                                }
                            }
                            _ => {
                                next_ch = *ch;
                                next_offset_end = idx;
                                break;
                            }
                        }
                    }
                    if offset_start != usize::MAX {
                        if buf.is_empty() {
                            buf.extend_from_slice(&self.input[offset_start..=offset_end]);
                        }
                    } else {
                        offset_start = next_offset_end;
                    }
                    buf.push(match next_ch {
                        b'n' | b'N' => b'\n',
                        b't' | b'T' => b'\t',
                        b'r' | b'R' => b'\r',
                        _ => next_ch,
                    });
                    offset_end = next_offset_end;
                }
                b'"' if self.unquote => {
                    in_quote = !in_quote;
                }
                b':' if !in_quote && self.stop_colon => {
                    stop_char = StopChar::Colon;
                    break;
                }
                b';' if !in_quote && self.stop_semicolon => {
                    stop_char = StopChar::Semicolon;
                    break;
                }
                b',' if !in_quote && self.stop_comma => {
                    stop_char = StopChar::Comma;
                    break;
                }
                b'=' if !in_quote && self.stop_equal => {
                    stop_char = StopChar::Equal;
                    break;
                }
                b'/' if !in_quote && self.stop_slash => {
                    stop_char = StopChar::Slash;
                    break;
                }
                b'.' if !in_quote && self.stop_dot => {
                    stop_char = StopChar::Dot;
                    break;
                }
                _ => {
                    if offset_start == usize::MAX {
                        offset_start = idx;
                    }
                    offset_end = idx;

                    if !buf.is_empty() {
                        buf.push(*ch);
                    }
                }
            }
        }

        if buf.is_empty() {
            if offset_start != usize::MAX {
                Some(Token {
                    text: Cow::Borrowed(&self.input[offset_start..=offset_end]),
                    start: offset_start,
                    end: offset_end,
                    stop_char,
                })
            } else {
                Some(Token {
                    text: Cow::Borrowed(b"".as_ref()),
                    start: offset_start,
                    end: offset_end,
                    stop_char,
                })
            }
        } else {
            Some(Token {
                text: Cow::Owned(buf),
                start: offset_start,
                end: offset_end,
                stop_char,
            })
        }
    }

    pub(crate) fn seek_lf(&mut self) -> bool {
        loop {
            match self.token() {
                Some(Token {
                    stop_char: StopChar::Lf,
                    ..
                }) => return true,
                None => return false,
                _ => {}
            }
        }
    }

    pub(crate) fn seek_value_or_eol(&mut self) -> StopChar {
        loop {
            match self.token() {
                Some(Token {
                    stop_char: StopChar::Colon,
                    ..
                }) => return StopChar::Colon,
                Some(Token {
                    stop_char: StopChar::Lf,
                    ..
                })
                | None => return StopChar::Lf,
                _ => {}
            }
        }
    }

    pub(crate) fn seek_param_value_or_eol(&mut self) -> StopChar {
        loop {
            match self.token() {
                Some(Token {
                    stop_char: stop_char @ (StopChar::Colon | StopChar::Semicolon | StopChar::Equal),
                    ..
                }) => return stop_char,
                Some(Token {
                    stop_char: StopChar::Lf,
                    ..
                })
                | None => return StopChar::Lf,
                _ => {}
            }
        }
    }
}

impl Token<'_> {
    pub fn into_string(self) -> String {
        String::from_utf8(self.text.into_owned()).unwrap_or_default()
    }
}
