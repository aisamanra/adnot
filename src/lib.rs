use std::collections::hash_map::HashMap;
use std::iter::Peekable;

#[derive(Debug, PartialEq)]
pub enum Value {
    Sum(String, Array),
    Product(HashMap<String, Value>),
    List(Array),
    Int(i64),
    Double(f64),
    String(String),
}

pub type Array = Vec<Value>;

pub type Loc = (u64, u64);

#[derive(Debug)]
pub struct AdnotError {
    pub message: String,
    pub loc: Loc,
}

impl Value {
    pub fn from_string(s: impl Into<String>) -> Result<Value, AdnotError> {
        Parser {
            iter: s.into().chars().peekable(),
            row: 0,
            col: 0,
            source: None,
        }
        .parse()
    }

    pub fn from_file(f: impl Into<std::path::PathBuf>) -> Result<Value, AdnotError> {
        panic!("unimplemented")
    }

    pub fn from_iter(i: impl Iterator<Item = char>) -> Result<Value, AdnotError> {
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

    fn loc(&self) -> (u64, u64) {
        (self.row, self.col)
    }

    fn err(&self, message: String) -> Result<Value, AdnotError> {
        Err(AdnotError {
            message: message,
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
            Some(c) if c.is_digit(10) => self.parse_number(digit_to_num(c)),
            c => self.err(format!("Unimplemented: {:?}", c)),
        }
    }

    fn parse_number(&mut self, mut num: i64) -> Result<Value, AdnotError> {
        while let Some(&s) = self.peek_char() {
            if s.is_digit(10) {
                let _ = self.next_char();
                num = (num * 10) + digit_to_num(s);
            } else {
                break;
            }
        }
        Ok(Value::Int(num))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_an_empty_array() {
        let stuff = "[]";
        assert_eq!(Value::List(Vec::new()), Value::from_string(stuff).unwrap());
    }

    #[test]
    fn it_parses_an_empty_array_with_spaces() {
        let stuff = " [ ] ";
        assert_eq!(Value::List(Vec::new()), Value::from_string(stuff).unwrap());
    }

    #[test]
    fn it_parses_an_empty_array_with_comments() {
        let stuff = "\n# before\n[\n  # in between\n]\n# after \n";
        assert_eq!(Value::List(Vec::new()), Value::from_string(stuff).unwrap());
    }

    #[test]
    fn it_parses_nested_empty_arrays() {
        let stuff = " [ [] ] ";
        assert_eq!(
            Value::List(vec![Value::List(Vec::new())]),
            Value::from_string(stuff).unwrap()
        );
    }

    #[test]
    fn it_parses_a_number() {
        let stuff = "[2 33 31337]";
        assert_eq!(
            Value::List(vec![Value::Int(2), Value::Int(33), Value::Int(31337)]),
            Value::from_string(stuff).unwrap()
        );
    }
}
