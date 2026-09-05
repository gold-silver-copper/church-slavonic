//! The S-expression reader and printer — hand-rolled, zero dependencies.
//!
//! Grammar: a value is an atom (`гдⷭ҇ь`, `nom`, `3`), a keyword (`:case`),
//! a double-quoted string with `\"` and `\\` escapes (`"гдⷭ҇ь,"`), or a
//! parenthesised list of values. `;` comments to end of line. Whitespace
//! separates. The printer emits a form the parser reads back identically
//! (`parse(print(v)) == v`, tested).

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// bare atom — heads, lemmas, feature values
    Atom(String),
    /// `:keyword` — feature names
    Key(String),
    /// `"…"` — verbatim surface text (may hold spaces, quotes, anything)
    Str(String),
    List(Vec<Value>),
}

/// A parse error with 1-based line/column of the offending character.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.col, self.message)
    }
}

impl std::error::Error for ParseError {}

struct Reader<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    line: usize,
    col: usize,
}

impl<'a> Reader<'a> {
    fn new(text: &'a str) -> Self {
        Reader { chars: text.chars().peekable(), line: 1, col: 1 }
    }
    fn err(&self, message: impl Into<String>) -> ParseError {
        ParseError { line: self.line, col: self.col, message: message.into() }
    }
    fn bump(&mut self) -> Option<char> {
        let c = self.chars.next();
        match c {
            Some('\n') => {
                self.line += 1;
                self.col = 1;
            }
            Some(_) => self.col += 1,
            None => {}
        }
        c
    }
    fn skip_space(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.bump();
            } else if c == ';' {
                while let Some(&c) = self.chars.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.bump();
                }
            } else {
                break;
            }
        }
    }
    fn value(&mut self) -> Result<Value, ParseError> {
        self.skip_space();
        match self.chars.peek() {
            None => Err(self.err("unexpected end of input")),
            Some('(') => {
                self.bump();
                let mut items = Vec::new();
                loop {
                    self.skip_space();
                    match self.chars.peek() {
                        Some(')') => {
                            self.bump();
                            return Ok(Value::List(items));
                        }
                        None => return Err(self.err("unclosed list")),
                        _ => items.push(self.value()?),
                    }
                }
            }
            Some(')') => Err(self.err("unexpected )")),
            Some('"') => {
                self.bump();
                let mut s = String::new();
                loop {
                    match self.bump() {
                        None => return Err(self.err("unclosed string")),
                        Some('"') => return Ok(Value::Str(s)),
                        Some('\\') => match self.bump() {
                            Some('"') => s.push('"'),
                            Some('\\') => s.push('\\'),
                            _ => return Err(self.err("bad escape")),
                        },
                        Some(c) => s.push(c),
                    }
                }
            }
            Some(':') => {
                self.bump();
                let word = self.word();
                if word.is_empty() {
                    return Err(self.err("bare :"));
                }
                Ok(Value::Key(word))
            }
            Some(_) => {
                let word = self.word();
                Ok(Value::Atom(word))
            }
        }
    }
    fn word(&mut self) -> String {
        let mut s = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() || matches!(c, '(' | ')' | '"' | ';') {
                break;
            }
            s.push(c);
            self.bump();
        }
        s
    }
}

/// Parse one value; trailing content is an error.
pub fn parse(text: &str) -> Result<Value, ParseError> {
    let mut r = Reader::new(text);
    let v = r.value()?;
    r.skip_space();
    if r.chars.peek().is_some() {
        return Err(r.err("trailing content after value"));
    }
    Ok(v)
}

/// Parse a whole file of values (a treebank book: one tree per verse).
pub fn parse_many(text: &str) -> Result<Vec<Value>, ParseError> {
    let mut r = Reader::new(text);
    let mut out = Vec::new();
    loop {
        r.skip_space();
        if r.chars.peek().is_none() {
            return Ok(out);
        }
        out.push(r.value()?);
    }
}

/// Print a value the parser reads back identically.
pub fn print(v: &Value) -> String {
    let mut s = String::new();
    write_value(v, &mut s);
    s
}

fn write_value(v: &Value, out: &mut String) {
    match v {
        Value::Atom(a) => out.push_str(a),
        Value::Key(k) => {
            out.push(':');
            out.push_str(k);
        }
        Value::Str(t) => {
            out.push('"');
            for c in t.chars() {
                if c == '"' || c == '\\' {
                    out.push('\\');
                }
                out.push(c);
            }
            out.push('"');
        }
        Value::List(items) => {
            out.push('(');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                write_value(item, out);
            }
            out.push(')');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_and_prints_back() {
        let text = r#"(s (w "Въ") (n нача́ло :case loc :num sg) (w "бг҃ъ,") ; comment
            (v рещѝ :t aor :p 3))"#;
        let v = parse(text).expect("parses");
        let printed = print(&v);
        assert_eq!(parse(&printed).expect("round-trip"), v);
        // strings hold anything, escapes included
        let v = Value::Str("say \"это\" \\ done".to_string());
        assert_eq!(parse(&print(&v)).expect("escapes"), v);
    }

    #[test]
    fn errors_carry_positions() {
        let e = parse("(a\n  (b").expect_err("unclosed");
        assert_eq!((e.line, e.col), (2, 5));
        assert!(parse("(a) b").is_err(), "trailing content");
        assert!(parse(")").is_err());
        assert!(parse("(:)").is_err(), "bare colon");
    }

    #[test]
    fn parse_many_reads_a_book() {
        let vs = parse_many("(a) ; verse 1\n(b c)\n").expect("many");
        assert_eq!(vs.len(), 2);
        assert_eq!(parse_many("  ").expect("empty"), Vec::new());
    }
}
