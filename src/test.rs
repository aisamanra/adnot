use crate::*;

#[test]
fn it_parses_an_empty_array() {
    let stuff = "[]";
    assert_eq!(Value::List(Vec::new()), Value::parse_string(stuff).unwrap());
}

#[test]
fn it_parses_an_empty_array_with_spaces() {
    let stuff = " [ ] ";
    assert_eq!(Value::List(Vec::new()), Value::parse_string(stuff).unwrap());
}

#[test]
fn it_parses_an_empty_array_with_comments() {
    let stuff = "\n# before\n[\n  # in between\n]\n# after \n";
    assert_eq!(Value::List(Vec::new()), Value::parse_string(stuff).unwrap());
}

#[test]
fn it_parses_nested_empty_arrays() {
    let stuff = " [ [] ] ";
    assert_eq!(
        Value::List(vec![Value::List(Vec::new())]),
        Value::parse_string(stuff).unwrap()
    );
}

#[test]
fn it_parses_a_number() {
    let stuff = "[2 33 31337]";
    assert_eq!(
        Value::List(vec![Value::Int(2), Value::Int(33), Value::Int(31337)]),
        Value::parse_string(stuff).unwrap()
    );
}

#[test]
fn it_parses_numbers_of_various_bases() {
    let stuff = "[0x10 0xff 0z10 0zbb 0d10 0o10 0o77 0b10 0b11]";
    assert_eq!(
        Value::List(vec![
            Value::Int(16),
            Value::Int(0xff),
            Value::Int(12),
            Value::Int(143),
            Value::Int(10),
            Value::Int(8),
            Value::Int(63),
            Value::Int(2),
            Value::Int(3)
        ]),
        Value::parse_string(stuff).unwrap()
    );
}

#[test]
fn it_parses_floats() {
    let stuff = "[100.0 0.01 1.0e+2]";
    assert_eq!(
        Value::List(vec![
            Value::Double(100.0),
            Value::Double(0.01),
            Value::Double(100.0)
        ]),
        Value::parse_string(stuff).unwrap()
    );
}

#[test]
fn it_parses_bare_words() {
    let stuff = "[foo bar_baz]";
    assert_eq!(
        Value::List(vec![
            Value::String("foo".into()),
            Value::String("bar_baz".into()),
        ]),
        Value::parse_string(stuff).unwrap()
    );
}

#[test]
fn it_parses_string_literals() {
    let stuff = "[\"foo\" \"bar_baz\"]";
    assert_eq!(
        Value::List(vec![
            Value::String("foo".into()),
            Value::String("bar_baz".into()),
        ]),
        Value::parse_string(stuff).unwrap()
    );
}

#[test]
fn it_parses_string_literals_with_escapes() {
    let stuff = "[\"\\n\" \"foo\\\"bar\"]";
    assert_eq!(
        Value::List(vec![
            Value::String("\n".into()),
            Value::String("foo\"bar".into()),
        ]),
        Value::parse_string(stuff).unwrap()
    );
}

#[test]
fn it_parses_tags() {
    let stuff = "[(foo) (bar 42 yes) (\"baz\" 1 2 3)]";
    assert_eq!(
        Value::List(vec![
            Value::Sum("foo".into(), Vec::new()),
            Value::Sum(
                "bar".into(),
                vec![Value::Int(42), Value::String("yes".into()),]
            ),
            Value::Sum(
                "baz".into(),
                vec![Value::Int(1), Value::Int(2), Value::Int(3),]
            ),
        ]),
        Value::parse_string(stuff).unwrap()
    );
}

#[test]
fn it_parses_map() {
    let stuff = "{a 1 \"b\" 2 \"another key\" 3}";
    let mut expected_map = HashMap::new();
    expected_map.insert("a".into(), Value::Int(1));
    expected_map.insert("b".into(), Value::Int(2));
    expected_map.insert("another key".into(), Value::Int(3));
    assert_eq!(
        Value::Map(expected_map),
        Value::parse_string(stuff).unwrap()
    );
}
