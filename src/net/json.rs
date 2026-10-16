fn ws(s: &[u8]) -> &[u8] {
    s.trim_ascii_start()
}

fn string_len(s: &[u8]) -> Option<usize> {
    s.iter()
        .enumerate()
        .skip(1)
        .scan(false, |esc, (i, &c)| {
            let end = !*esc && c == b'"';
            *esc = !*esc && c == b'\\';
            Some((i, end))
        })
        .find(|&(_, end)| end)
        .map(|(i, _)| i + 1)
}

fn nested_len(s: &[u8]) -> Option<usize> {
    s.iter()
        .enumerate()
        .scan((0i32, false, false), |(depth, quoted, esc), (i, &c)| {
            match (*quoted, *esc, c) {
                (true, true, _) => *esc = false,
                (true, false, b'\\') => *esc = true,
                (true, false, b'"') => *quoted = false,
                (true, false, _) => {}
                (false, _, b'"') => *quoted = true,
                (false, _, b'{' | b'[') => *depth += 1,
                (false, _, b'}' | b']') => *depth -= 1,
                _ => {}
            }
            Some((i, *depth))
        })
        .find(|&(_, depth)| depth <= 0)
        .map(|(i, _)| i + 1)
}

fn value_len(s: &[u8]) -> Option<usize> {
    match s.first()? {
        b'"' => string_len(s),
        b'{' | b'[' => nested_len(s),
        _ => Some(s.iter().position(|c| matches!(c, b',' | b'}' | b']') || c.is_ascii_whitespace()).unwrap_or(s.len()))
            .filter(|&n| n > 0),
    }
}

fn split_value(s: &[u8]) -> Option<(&[u8], &[u8])> {
    let s = ws(s);
    let n = value_len(s)?;
    let rest = ws(&s[n..]);
    Some((&s[..n], rest.strip_prefix(b",").unwrap_or(rest)))
}

fn inner(s: &[u8], open: u8, close: u8) -> &[u8] {
    let s = s.trim_ascii();
    match (s.first(), s.last()) {
        (Some(&a), Some(&b)) if a == open && b == close && s.len() >= 2 => &s[1..s.len() - 1],
        _ => &[],
    }
}

pub fn elements(array: &[u8]) -> impl Iterator<Item = &[u8]> {
    let mut rest = inner(array, b'[', b']');
    std::iter::from_fn(move || {
        let (v, r) = split_value(rest)?;
        rest = r;
        Some(v)
    })
}

pub fn members(object: &[u8]) -> impl Iterator<Item = (&[u8], &[u8])> {
    let mut rest = inner(object, b'{', b'}');
    std::iter::from_fn(move || {
        let s = ws(rest);
        let k = string_len(s).filter(|_| s.first() == Some(&b'"'))?;
        let after = ws(&s[k..]).strip_prefix(b":")?;
        let (v, r) = split_value(after)?;
        rest = r;
        Some((&s[1..k - 1], v))
    })
}

pub fn get<'a>(object: &'a [u8], key: &str) -> Option<&'a [u8]> {
    members(object).find(|(k, _)| *k == key.as_bytes()).map(|(_, v)| v)
}

fn hex4(s: &[u8]) -> Option<u32> {
    s.get(..4).and_then(|h| std::str::from_utf8(h).ok()).and_then(|h| u32::from_str_radix(h, 16).ok())
}

fn unescape(s: &[u8]) -> Option<(char, usize)> {
    let high = hex4(s.get(1..)?)?;
    let pair = (0xD800..0xDC00)
        .contains(&high)
        .then(|| s.get(5..7).filter(|p| *p == b"\\u").and_then(|_| hex4(&s[7..])))
        .flatten()
        .filter(|low| (0xDC00..0xE000).contains(low));
    match pair {
        Some(low) => char::from_u32(0x10000 + ((high - 0xD800) << 10) + (low - 0xDC00)).map(|c| (c, 11)),
        None => Some((char::from_u32(high).unwrap_or('?'), 5)),
    }
}

fn next_char(s: &[u8]) -> Option<(char, usize)> {
    match s {
        [] => None,
        [b'\\', b'u', ..] => unescape(&s[1..]).map(|(c, n)| (c, n + 1)).or(Some(('?', 2))),
        [b'\\', e, ..] => Some((
            match e {
                b'n' | b't' | b'r' | b'b' | b'f' => ' ',
                other => *other as char,
            },
            2,
        )),
        _ => {
            let n = utf8_len(s);
            Some((std::str::from_utf8(&s[..n]).ok().and_then(|t| t.chars().next()).unwrap_or('?'), n))
        }
    }
}

fn utf8_len(s: &[u8]) -> usize {
    let lead = s[0].leading_ones() as usize;
    let want = if (2..=4).contains(&lead) { lead } else { 1 };
    1 + s[1..].iter().take(want - 1).take_while(|&&b| b & 0xC0 == 0x80).count()
}

pub fn text(value: &[u8], out: &mut [u8]) -> usize {
    let body = match value.first() {
        Some(b'"') if value.len() >= 2 => &value[1..value.len() - 1],
        _ => value,
    };
    let cap = out.len().saturating_sub(1);
    let mut rest = body;
    let n = std::iter::from_fn(|| {
        let (c, used) = next_char(rest)?;
        rest = &rest[used..];
        Some(if c.is_control() { ' ' } else { c })
    })
    .try_fold(0, |n, c| {
        let len = c.len_utf8();
        match n + len <= cap {
            true => {
                c.encode_utf8(&mut out[n..n + len]);
                Ok(n + len)
            }
            false => Err(n),
        }
    })
    .unwrap_or_else(|n| n);
    out[n..].fill(0);
    n
}

pub fn number(value: &[u8]) -> Option<i64> {
    let mut buf = [0; 24];
    let n = text(value, &mut buf);
    std::str::from_utf8(&buf[..n]).ok()?.split('.').next()?.parse().ok()
}

pub fn truthy(value: &[u8]) -> bool {
    value == b"true" || number(value).is_some_and(|n| n != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walks_nested_values() {
        let js = br#" [ {"a": "x]}\"y", "b": [1, {"c": 2}], "n": -3}, 4 , "s" ] "#;
        let items: Vec<&[u8]> = elements(js).collect();
        assert_eq!(items.len(), 3);
        assert_eq!(items[1], b"4");
        let keys: Vec<&[u8]> = members(items[0]).map(|(k, _)| k).collect();
        assert_eq!(keys, [&b"a"[..], b"b", b"n"]);
        assert_eq!(get(items[0], "b"), Some(&br#"[1, {"c": 2}]"#[..]));
        assert_eq!(get(items[0], "n").and_then(number), Some(-3));
        let mut out = [0; 8];
        assert_eq!(text(get(items[0], "a").unwrap(), &mut out), 5);
        assert_eq!(&out[..5], b"x]}\"y");
        assert_eq!(get(items[0], "missing"), None);
    }

    #[test]
    fn decodes_and_truncates_text() {
        let mut out = [0xFF; 8];
        assert_eq!(text(br#""Jo\u00e3o""#, &mut out), 5);
        assert_eq!(&out[..5], "João".as_bytes());
        assert_eq!(out[5..], [0; 3]);
        let mut small = [0; 5];
        assert_eq!(text("\"Joãoo\"".as_bytes(), &mut small), 4);
        assert_eq!(&small[..4], "Joã".as_bytes());
        let mut emoji = [0; 8];
        assert_eq!(text(br#""\ud83d\ude00!\n""#, &mut emoji), 6);
        assert_eq!(&emoji[..6], "😀! ".as_bytes());
        assert!(truthy(b"true") && truthy(b"1") && !truthy(b"false") && !truthy(b"0"));
        assert_eq!(number(b"55435"), Some(55435));
        assert_eq!(number(br#""12""#), Some(12));
        assert_eq!(number(b"null"), None);
    }

    #[test]
    fn tolerates_malformed_input() {
        assert_eq!(elements(b"[1, 2").count(), 0);
        assert_eq!(elements(b"{}").count(), 0);
        assert_eq!(members(br#"{"a" 1}"#).count(), 0);
        assert_eq!(elements(br#"[{"a": "unterminated}]"#).count(), 0);
        assert_eq!(text(b"\"\\u12\"", &mut [0; 8]), 3);
    }
}
