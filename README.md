# Adnot

![stability: experimental](https://img.shields.io/badge/stability-experimental-red.svg)

**WARNING**: this repo contains unrepentant bikeshedding and wheel-reinvention. You almost definitely shouldn't use it, and it's probably best to disregard the entire thing!

The *Adnot* format is a simple data and configuration format intended to have a slightly enriched data model when compared to JSON or s-expressions but still retain the comparative simplicity of those formats. Unlike JSON, Adnot chooses to avoid redundant structural information like punctuation; unlike s-expressions, Adnot values natively express a wider range of basic data types.

*Adnot* is not intended to be a data interchange format, but rather to be a richer and more convenient syntax for certain kinds of data description that might otherwise be done in more unwieldy, complicated formats like YAML. As a first approximation, Adnot may be treated as a more human- and version-control-friendly version of JSON whose data model is intended to resemble the data model of statically typed functional programming languages.

A given Adnot value is either one of three basic types—an integer, a double, a string—or one of three composite types: a sequence of values, a mapping of symbols to values, or a tagged sequence of values which begins with a symbol:

```
expr ::= "{" (string expr) * "}"
       | "(" string expr* ")"
       | "[" expr* "]"
       | string
       | integer
       | double
```

Strings can be expressed in two different ways: one is quoted strings, which are formatted like JSON strings with the same encoding and the same set of escape sequences; the other is as bare words, in which strings that begin with a character of unicode class `XID_Start` and consist subsequently of zero or more `XID_Continue` characters can be written without quotation marks.

The three kinds of composite types are meant to resemble records, sum or variant types, and lists, respectively. Zero or more symbol-expression pairs inside curly brackets form a _mapping_:

```
# a basic mapping
{
  x 2
  y 3
  "and z" 4
}
```

Pairs do not include colons and are not separated by commas. A mapping _must_ contain an even number of sub-expressions, and every odd subexpression _must_ be a string. Whitespace is ignored except as a separator between tokens, so the above map is identical to

```
{x 2 y 3 "and z" 4}
```

A _list_ is represented by square brackets with zero or more possibly-heterogeneous expressions:

```
# a basic list
[ 2 "foo" bar ]
```

A _tagged expression_ is represented by parentheses with a single string followed by zero or more possibly-heterogeneous expressions:

```
# a basic tagged expression
(some_tag blah 7.8 "??")
```

These are how tagged data-types are represented: because the thing inside the parens _must_ be a string, it can correspond to a data type in an ML-like language.

Adnot values can contain comments, which are line-oriented and begin with a `#` character.
