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

#[derive(Debug)]
pub struct AdnotError {
    message: String,
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

    pub fn from_iter(mut i: impl Iterator<Item = char>) -> Result<Value, AdnotError> {
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

impl<I: Iterator<Item = char>> Parser<I> {
    fn parse(&mut self) -> Result<Value, AdnotError> {
        let value = self.parse_value()?;
        self.skip_whitespace()?;

        if let Some(c) = self.iter.next() {
            return Err(AdnotError {
                message: format!("Unexpected {}, expected end of input", c),
            });
        }

        Ok(value)
    }

    fn parse_value(&mut self) -> Result<Value, AdnotError> {
        self.skip_whitespace()?;
        match self.iter.next() {
            Some('[') => {
                return self.parse_list();
            }
            c => {
                return Err(AdnotError {
                    message: format!("Unimplemented: {:?}", c),
                })
            }
        }
        Err(AdnotError {
            message: format!("Unimplemented"),
        })
    }

    fn parse_list(&mut self) -> Result<Value, AdnotError> {
        let mut values = Vec::new();
        loop {
            self.skip_whitespace()?;
            if Some(&']') == self.iter.peek() {
                let _ = self.iter.next();
                return Ok(Value::List(values));
            } else {
                values.push(self.parse_value()?);
            }
        }
    }

    fn skip_whitespace(&mut self) -> Result<(), AdnotError> {
        while let Some(s) = self.iter.peek() {
            if s.is_whitespace() {
                let _ = self.iter.next();
            } else {
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
    fn it_parses_nested_empty_arrays() {
        let stuff = " [ [] ] ";
        assert_eq!(
            Value::List(vec![Value::List(Vec::new())]),
            Value::from_string(stuff).unwrap()
        );
    }
}
