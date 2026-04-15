/// A parsed segment of a format template.
#[derive(Debug, PartialEq)]
pub enum Segment {
    /// Literal text to emit as-is.
    Literal(String),
    /// `{N}` or `{name}` — substitute the field value; 1-indexed for numeric keys.
    Field { key: FieldKey },
    /// `{N:default}` — substitute the field, falling back to a default when empty.
    FieldWithDefault { key: FieldKey, default: String },
    /// `{?N:text}` — emit `text` only when field N is non-empty. `text` may
    /// itself contain `{N}` substitutions.
    Conditional { key: FieldKey, body: Vec<Segment> },
}

/// How a field is addressed — by 1-based index or by header name.
#[derive(Debug, PartialEq, Clone)]
pub enum FieldKey {
    Index(usize),
    Name(String),
}

/// Parse a format template into a list of [`Segment`]s.
///
/// Syntax:
/// - `{1}` / `{name}` — field substitution (1-indexed)
/// - `{1:default}` / `{name:default}` — field with fallback default
/// - `{?1:body text}` — conditional: include body only if field is non-empty
///   (the body may itself contain `{N}` or `{name}` references)
/// - `{{` / `}}` — escaped literal brace
pub fn parse_template(template: &str) -> Result<Vec<Segment>, String> {
    let chars: Vec<char> = template.chars().collect();
    let mut segments = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '{' {
            // Escaped `{{`
            if i + 1 < chars.len() && chars[i + 1] == '{' {
                segments.push(Segment::Literal("{".to_string()));
                i += 2;
                continue;
            }

            // Find the matching closing `}`
            let start = i + 1;
            let end = find_closing_brace(&chars, start)?;

            let inner: String = chars[start..end].iter().collect();
            let seg = parse_placeholder(&inner)?;
            segments.push(seg);
            i = end + 1; // skip past `}`
        } else if chars[i] == '}' {
            // Escaped `}}`
            if i + 1 < chars.len() && chars[i + 1] == '}' {
                segments.push(Segment::Literal("}".to_string()));
                i += 2;
            } else {
                return Err(format!("Unexpected `}}` at position {}", i));
            }
        } else {
            // Collect a run of literal characters.
            let mut lit = String::new();
            while i < chars.len() && chars[i] != '{' && chars[i] != '}' {
                lit.push(chars[i]);
                i += 1;
            }
            if !lit.is_empty() {
                segments.push(Segment::Literal(lit));
            }
        }
    }

    Ok(segments)
}

/// Locate the `}` that closes an opening `{`, starting the search at `start`
/// (the character *after* `{`).  Handles nested `{…}` pairs so that
/// `{?1:text {2} here}` finds the outer `}` correctly.
fn find_closing_brace(chars: &[char], start: usize) -> Result<usize, String> {
    let mut depth = 0usize;
    for (i, &ch) in chars.iter().enumerate().skip(start) {
        match ch {
            '{' => depth += 1,
            '}' if depth > 0 => depth -= 1,
            '}' => return Ok(i),
            _ => {}
        }
    }
    Err("Unclosed `{` in format template".to_string())
}

/// Parse the content inside a `{ … }` into the appropriate [`Segment`] variant.
fn parse_placeholder(inner: &str) -> Result<Segment, String> {
    if inner.is_empty() {
        return Err("Empty placeholder `{}`".to_string());
    }

    // Conditional: `?key:body`
    if let Some(rest) = inner.strip_prefix('?') {
        let (key_str, body_str) = split_first_colon(rest)?;
        let key = parse_key(key_str.trim())?;
        let body = parse_template(body_str)?;
        return Ok(Segment::Conditional { key, body });
    }

    // Field with default: `key:default`
    if let Some(colon_pos) = inner.find(':') {
        let key_str = &inner[..colon_pos];
        let default = inner[colon_pos + 1..].to_string();
        let key = parse_key(key_str.trim())?;
        return Ok(Segment::FieldWithDefault { key, default });
    }

    // Plain field: `key`
    let key = parse_key(inner.trim())?;
    Ok(Segment::Field { key })
}

/// Split on the first `:`, returning an error when none is found.
fn split_first_colon(s: &str) -> Result<(&str, &str), String> {
    s.find(':')
        .map(|pos| (&s[..pos], &s[pos + 1..]))
        .ok_or_else(|| format!("Expected `:` in conditional placeholder `{{?{s}}}`"))
}

/// Parse a key that is either a 1-based integer index or a header name.
fn parse_key(s: &str) -> Result<FieldKey, String> {
    if s.is_empty() {
        return Err("Empty field key in placeholder".to_string());
    }
    match s.parse::<usize>() {
        Ok(0) => Err("Field indices are 1-based; `{0}` is not valid".to_string()),
        Ok(n) => Ok(FieldKey::Index(n)),
        Err(_) => Ok(FieldKey::Name(s.to_string())),
    }
}

/// Resolve a [`FieldKey`] against a record, returning the field value or `None`.
pub fn resolve_key<'a>(key: &FieldKey, record: &'a [String], headers: Option<&[String]>) -> Option<&'a str> {
    match key {
        FieldKey::Index(n) => record.get(n - 1).map(String::as_str),
        FieldKey::Name(name) => {
            let headers = headers?;
            let pos = headers.iter().position(|h| h == name)?;
            record.get(pos).map(String::as_str)
        }
    }
}

/// Render a list of [`Segment`]s against a CSV record.
pub fn render(segments: &[Segment], record: &[String], headers: Option<&[String]>) -> String {
    let mut out = String::new();
    for seg in segments {
        match seg {
            Segment::Literal(s) => out.push_str(s),
            Segment::Field { key } => {
                if let Some(val) = resolve_key(key, record, headers) {
                    out.push_str(val);
                }
            }
            Segment::FieldWithDefault { key, default } => {
                let val = resolve_key(key, record, headers).unwrap_or("");
                if val.is_empty() {
                    out.push_str(default);
                } else {
                    out.push_str(val);
                }
            }
            Segment::Conditional { key, body } => {
                let val = resolve_key(key, record, headers).unwrap_or("");
                if !val.is_empty() {
                    out.push_str(&render(body, record, headers));
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    // ── parse_template ─────────────────────────────────────────────────────────

    #[test]
    fn literal_only() {
        let segs = parse_template("hello world").unwrap();
        assert_eq!(segs, vec![Segment::Literal("hello world".to_string())]);
    }

    #[test]
    fn single_index_field() {
        let segs = parse_template("{1}").unwrap();
        assert_eq!(segs, vec![Segment::Field { key: FieldKey::Index(1) }]);
    }

    #[test]
    fn named_field() {
        let segs = parse_template("{name}").unwrap();
        assert_eq!(
            segs,
            vec![Segment::Field {
                key: FieldKey::Name("name".to_string())
            }]
        );
    }

    #[test]
    fn field_with_default() {
        let segs = parse_template("{2:n/a}").unwrap();
        assert_eq!(
            segs,
            vec![Segment::FieldWithDefault {
                key: FieldKey::Index(2),
                default: "n/a".to_string()
            }]
        );
    }

    #[test]
    fn conditional_segment() {
        let segs = parse_template("{?3: and {3}}").unwrap();
        assert_eq!(
            segs,
            vec![Segment::Conditional {
                key: FieldKey::Index(3),
                body: vec![
                    Segment::Literal(" and ".to_string()),
                    Segment::Field { key: FieldKey::Index(3) },
                ]
            }]
        );
    }

    #[test]
    fn escaped_braces() {
        let segs = parse_template("{{literal}}").unwrap();
        assert_eq!(
            segs,
            vec![
                Segment::Literal("{".to_string()),
                Segment::Literal("literal".to_string()),
                Segment::Literal("}".to_string()),
            ]
        );
    }

    #[test]
    fn error_on_zero_index() {
        assert!(parse_template("{0}").is_err());
    }

    #[test]
    fn error_unclosed_brace() {
        assert!(parse_template("{1").is_err());
    }

    // ── render ─────────────────────────────────────────────────────────────────

    #[test]
    fn render_basic() {
        let segs = parse_template("Hello {1}, how are you {2}?").unwrap();
        let rec = fields(&["Alice", "Bob"]);
        assert_eq!(render(&segs, &rec, None), "Hello Alice, how are you Bob?");
    }

    #[test]
    fn render_default_used_when_empty() {
        let segs = parse_template("{1:unknown}").unwrap();
        let rec = fields(&[""]);
        assert_eq!(render(&segs, &rec, None), "unknown");
    }

    #[test]
    fn render_default_not_used_when_value_present() {
        let segs = parse_template("{1:unknown}").unwrap();
        let rec = fields(&["Alice"]);
        assert_eq!(render(&segs, &rec, None), "Alice");
    }

    #[test]
    fn render_conditional_included() {
        let segs = parse_template("Hi {1}{?2:, age {2}}").unwrap();
        let rec = fields(&["Alice", "30"]);
        assert_eq!(render(&segs, &rec, None), "Hi Alice, age 30");
    }

    #[test]
    fn render_conditional_excluded() {
        let segs = parse_template("Hi {1}{?2:, age {2}}").unwrap();
        let rec = fields(&["Alice", ""]);
        assert_eq!(render(&segs, &rec, None), "Hi Alice");
    }

    #[test]
    fn render_named_field() {
        let headers = fields(&["first", "last"]);
        let segs = parse_template("{first} {last}").unwrap();
        let rec = fields(&["Alice", "Smith"]);
        assert_eq!(render(&segs, &rec, Some(&headers)), "Alice Smith");
    }

    #[test]
    fn render_out_of_bounds_field_empty() {
        let segs = parse_template("{5}").unwrap();
        let rec = fields(&["a", "b"]);
        assert_eq!(render(&segs, &rec, None), "");
    }
}
