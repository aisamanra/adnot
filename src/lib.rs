use std::collections::hash_map::HashMap;
use std::iter::Peekable;

#[cfg(test)]
mod test;

/// A representation of an Adnot value.
#[derive(Debug, PartialEq)]
pub enum Value {
    /// A "sum" is a string tag plus zero or more payload values. In
    /// Adnot notation, it is written with parentheses. Tags can be
    /// bare strings or quoted strings.
    ///
    /// ```text
    /// [
    ///   # a sum with tag "foo" and three payload values
    ///   (foo 1 2 3)
    ///   # a sum with tag "bar" and no payload values
    ///   ("bar")
    /// ]
    /// ```
    Sum(String, Array),

    /// A "map" is a mapping from string keys to values. It is written
    /// with curly braces and space-separated key-value pairs. It is a
    /// parse error for any key to be repeated. Keys may be bare
    /// strings or quoted strings. It is also a parse error for the
    /// curly braces to contain a non-even number of values.
    ///
    /// ```text
    /// {
    ///   foo 1
    ///   "bar" 2
    /// }
    /// ```
    Map(HashMap<String, Value>),

    /// A "list" is a sequence of zero or more values. In Adnot
    /// notation, it is written with square brackets.
    ///
    /// ```text
    /// [foo 2 {}]
    /// ```
    List(Array),

    /// An "int" is an integer. This library supports sixty-four-bit
    /// signed integers. Numbers can include `_` as a separator, which
    /// is ignored for the purposes of parsing. Numbers can be written
    /// in different bases, as well: hexadecimal with `0x`, duodecimal
    /// with `0z`, octal with `0o`, and binary with `0b`. The
    /// redundant decimal prefix `0d` is also supported for symmetry.
    ///
    /// ```text
    /// [
    ///   1234         # decimal number
    ///   0d1234       # the same number with explicit prefix
    ///   0xbeef       # the hexadecimal representation of 48878
    ///   0zbaba       # the duodecimal representation of 20590
    ///   0o7171       # the octal representation of 3705
    ///   0b1010_0101  # the binary representation of 165
    /// ]
    /// ```
    Int(i64),

    /// A double-precision floating point value. **DOCUMENT ME**
    Double(f64),

    /// A string. This can be written either as a bare string, which
    /// starts with an alphabetic unicode character and is followed by
    /// zero or more alphanumeric unicode characters, or it can be
    /// quoted. Bare strings cannot include spaces or punctuation
    /// other than underscores.
    ///
    /// ```text
    /// [
    ///   foo            # a bare string
    ///   one_two_three  # a bare string with underscores
    ///   "bar baz"      # a quoted string
    /// ]
    /// ```
    String(String),
}

type Array = Vec<Value>;

#[derive(Debug)]
pub struct Location {
    row: u64,
    col: u64,
    src: Option<String>,
}

#[derive(Debug)]
pub struct AdnotError {
    pub message: String,
    pub loc: Location,
}

impl std::fmt::Display for AdnotError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        if let Some(filename) = &self.loc.src {
            writeln!(
                fmt,
                "{}[{}:{}]: {}",
                filename, self.loc.row, self.loc.col, self.message
            )
        } else {
            writeln!(fmt, "[{}:{}]: {}", self.loc.row, self.loc.col, self.message)
        }
    }
}

impl Value {
    pub fn parse_str(s: &str) -> Result<Value, AdnotError> {
        Parser {
            iter: s.chars().peekable(),
            row: 0,
            col: 0,
            source: None,
        }
        .parse()
    }

    pub fn parse_string(s: impl Into<String>) -> Result<Value, AdnotError> {
        Parser {
            iter: s.into().chars().peekable(),
            row: 0,
            col: 0,
            source: None,
        }
        .parse()
    }

    pub fn parse_iter(i: impl Iterator<Item = char>) -> Result<Value, AdnotError> {
        Parser {
            iter: i.peekable(),
            row: 0,
            col: 0,
            source: None,
        }
        .parse()
    }
}

struct Parser<I: Iterator<Item = char>> {
    iter: Peekable<I>,
    row: u64,
    col: u64,
    source: Option<String>,
}

fn is_special(c: char) -> bool {
    c == '[' || c == ']' || c == '(' || c == ')' || c == '{' || c == '}' || c == '#'
}

fn digit_to_num(c: char) -> i64 {
    match c {
        '0' => 0,
        '1' => 1,
        '2' => 2,
        '3' => 3,
        '4' => 4,
        '5' => 5,
        '6' => 6,
        '7' => 7,
        '8' => 8,
        '9' => 9,
        'A' | 'a' => 10,
        'B' | 'b' => 11,
        'C' | 'c' => 12,
        'D' | 'd' => 13,
        'E' | 'e' => 14,
        'F' | 'f' => 15,
        _ => unreachable!(),
    }
}

impl<I: Iterator<Item = char>> Parser<I> {
    // internal helpers
    fn peek_char(&mut self) -> Option<&char> {
        self.iter.peek()
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.iter.next();
        if c == Some('\n') {
            self.row += 0;
            self.col = 0;
        } else {
            self.col += 1;
        }
        c
    }

    fn loc(&self) -> Location {
        Location {
            row: self.row,
            col: self.col,
            src: self.source.clone(),
        }
    }

    fn err<T>(&self, message: String) -> Result<T, AdnotError> {
        Err(AdnotError {
            message,
            loc: self.loc(),
        })
    }

    // parsing entrypoint
    fn parse(&mut self) -> Result<Value, AdnotError> {
        self.skip_whitespace()?;
        let value = self.parse_value()?;
        self.skip_whitespace()?;

        if let Some(c) = self.next_char() {
            return self.err(format!("Unexpected {}, expected end of input", c));
        }

        Ok(value)
    }

    // parse a single value. this assumes that whitespace has already
    // been skipped
    fn parse_value(&mut self) -> Result<Value, AdnotError> {
        match self.next_char() {
            Some('[') => self.parse_list(),
            Some('(') => self.parse_tag(),
            Some('{') => self.parse_map(),

            Some('"') => Ok(Value::String(self.parse_string_literal()?)),
            Some('0') => match self.peek_char() {
                Some('X' | 'x') => {
                    let _ = self.next_char();
                    self.parse_number(0, 16)
                }
                Some('Z' | 'z') => {
                    let _ = self.next_char();
                    self.parse_number(0, 12)
                }
                Some('D' | 'd') => {
                    let _ = self.next_char();
                    self.parse_number(0, 10)
                }
                Some('O' | 'o') => {
                    let _ = self.next_char();
                    self.parse_number(0, 8)
                }
                Some('B' | 'b') => {
                    let _ = self.next_char();
                    self.parse_number(0, 2)
                }
                _ => self.parse_number(0, 10),
            },

            Some(c) if c.is_ascii_digit() => self.parse_number(digit_to_num(c), 10),
            Some(c) if c.is_alphabetic() => {
                Ok(Value::String(self.parse_bare_word(String::from(c))?))
            }
            c => self.err(format!("Unexpected character {:?}", c)),
        }
    }

    fn parse_number(&mut self, mut num: i64, base: u32) -> Result<Value, AdnotError> {
        while let Some(&s) = self.peek_char() {
            if s.is_digit(base) {
                let _ = self.next_char();
                num = (num * base as i64) + digit_to_num(s);
            } else if s.is_whitespace() || is_special(s) {
                break;
            } else if s == '.' {
                if base == 10 {
                    return self.parse_float(num as f64);
                } else {
                    return self.err(format!("Base-{} floats are not supported", base));
                }
            } else if s == '_' {
                // continue and ignore
                let _ = self.next_char();
            } else {
                return self.err(format!("Invalid character in number: {}", s));
            }
        }
        Ok(Value::Int(num))
    }

    // this will pick up after a `.` has been seen, with the
    // before-the-dot part being already cast into an f64 as `whole_part`
    fn parse_float(&mut self, _whole_part: f64) -> Result<Value, AdnotError> {
        panic!("unimplemented")
    }

    fn parse_bare_word(&mut self, mut buf: String) -> Result<String, AdnotError> {
        while let Some(&s) = self.peek_char() {
            if s.is_alphanumeric() || s == '_' {
                let _ = self.next_char();
                buf.push(s);
            } else if s.is_whitespace() || is_special(s) {
                break;
            } else {
                return self.err(format!("Invalid character in string: {}", s));
            }
        }
        Ok(buf)
    }

    fn parse_escape(&mut self) -> Result<char, AdnotError> {
        Ok(match self.next_char() {
            Some('n') => '\n',
            Some('t') => '\t',
            Some('r') => '\r',
            Some('f') => '\x0c',
            Some('\\') => '\\',
            Some('"') => '"',
            Some(c) => return self.err(format!("Invalid escape: \\{}", c)),
            None => return self.err("Unexpected end-of-file when parsing string literal".into()),
        })
    }

    fn parse_string_literal(&mut self) -> Result<String, AdnotError> {
        let mut buf = String::new();
        while let Some(s) = self.next_char() {
            match s {
                '"' => break,
                '\\' => buf.push(self.parse_escape()?),
                _ => buf.push(s),
            }
        }
        Ok(buf)
    }

    fn parse_list(&mut self) -> Result<Value, AdnotError> {
        let mut values = Vec::new();
        loop {
            self.skip_whitespace()?;
            if self.peek_char() == Some(&']') {
                let _ = self.next_char();
                return Ok(Value::List(values));
            } else {
                values.push(self.parse_value()?);
            }
        }
    }

    fn parse_tag(&mut self) -> Result<Value, AdnotError> {
        self.skip_whitespace()?;
        // next item _must_ be a tag string
        let tag = match self.next_char() {
            Some('"') => self.parse_string_literal()?,
            Some(c) if c.is_alphabetic() => self.parse_bare_word(String::from(c))?,
            Some(c) => return self.err(format!("Unexpected tag character: {}", c)),
            None => return self.err("Unexpected end of input while parsing tag".into()),
        };
        let mut values = Vec::new();
        loop {
            self.skip_whitespace()?;
            if self.peek_char() == Some(&')') {
                let _ = self.next_char();
                return Ok(Value::Sum(tag, values));
            } else {
                values.push(self.parse_value()?);
            }
        }
    }

    fn parse_map(&mut self) -> Result<Value, AdnotError> {
        let mut values = HashMap::new();
        loop {
            self.skip_whitespace()?;
            if self.peek_char() == Some(&'}') {
                let _ = self.next_char();
                return Ok(Value::Map(values));
            } else {
                let raw_key = self.parse_value()?;
                let key = if let Value::String(k) = raw_key {
                    k
                } else {
                    return self.err(format!("Expected a string key, found {:?}", raw_key));
                };
                self.skip_whitespace()?;
                let val = self.parse_value()?;
                values.insert(key, val);
            }
        }
    }

    fn skip_whitespace(&mut self) -> Result<(), AdnotError> {
        while let Some(s) = self.peek_char() {
            if s.is_whitespace() {
                let _ = self.next_char();
            } else if *s == '#' {
                self.skip_comment()?;
            } else {
                break;
            }
        }
        Ok(())
    }

    fn skip_comment(&mut self) -> Result<(), AdnotError> {
        while let Some(s) = self.next_char() {
            if s == '\n' {
                break;
            }
        }
        Ok(())
    }
}
