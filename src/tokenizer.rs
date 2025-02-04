use std::{
    borrow::Cow,
    iter::{Enumerate, Peekable},
    slice::Iter,
};

pub struct Parser<'x> {
    input: &'x [u8],
    iter: Peekable<Enumerate<Iter<'x, u8>>>,
    strict: bool,
    stop_colon: bool,
    stop_semicolon: bool,
    stop_comma: bool,
    stop_equal: bool,
    stop_slash: bool,
    unfold_qp: bool,
    unquote: bool,
}

pub(crate) struct Token<'x> {
    text: Cow<'x, [u8]>,
    start: usize,
    end: usize,
    stop_char: u8,
}

impl<'x> Parser<'x> {
    pub fn new(input: &'x [u8]) -> Self {
        Self {
            input,
            iter: input.iter().enumerate().peekable(),
            strict: false,
            stop_colon: true,
            stop_semicolon: true,
            stop_comma: true,
            stop_equal: true,
            stop_slash: false,
            unfold_qp: false,
            unquote: true,
        }
    }

    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    pub fn qp_value(&mut self) {
        self.stop_colon = false;
        self.stop_semicolon = true;
        self.stop_comma = false;
        self.stop_equal = false;
        self.stop_slash = false;
        self.unfold_qp = true;
        self.unquote = false;
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
        let last_char;
        let mut buf: Vec<u8> = vec![];

        'outer: loop {
            let (idx, ch) = if let Some(next) = self.iter.next() {
                next
            } else if offset_start != usize::MAX {
                last_char = b'\n';
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
                        last_char = b'\n';
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
                                    last_char = b'\n';
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
                    last_char = b':';
                    break;
                }
                b';' if !in_quote && self.stop_semicolon => {
                    last_char = b';';
                    break;
                }
                b',' if !in_quote && self.stop_comma => {
                    last_char = b',';
                    break;
                }
                b'=' if !in_quote && self.stop_equal => {
                    last_char = b'=';
                    break;
                }
                b'/' if !in_quote && self.stop_slash => {
                    last_char = b'/';
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
                    stop_char: last_char,
                })
            } else {
                Some(Token {
                    text: Cow::Borrowed(b"".as_ref()),
                    start: offset_start,
                    end: offset_end,
                    stop_char: last_char,
                })
            }
        } else {
            Some(Token {
                text: Cow::Owned(buf),
                start: offset_start,
                end: offset_end,
                stop_char: last_char,
            })
        }
    }
}
