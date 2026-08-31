//! Small string helpers. Ports `ra.common.StringUtil`.

/// Uppercase the first character, leave the rest untouched.
///
/// ```
/// # use ra_common::util::strings::capitalize_first;
/// assert_eq!(capitalize_first("hello world"), "Hello world");
/// ```
pub fn capitalize_first(input: &str) -> String {
    let mut chars = input.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// Uppercase the first character and every character immediately following a
/// space.
///
/// ```
/// # use ra_common::util::strings::capitalize;
/// assert_eq!(capitalize("hello world"), "Hello World");
/// ```
pub fn capitalize(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut capitalize_next = true;
    for c in input.chars() {
        if capitalize_next && c != ' ' {
            out.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            out.push(c);
            capitalize_next = c == ' ';
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first() {
        assert_eq!(capitalize_first(""), "");
        assert_eq!(capitalize_first("a"), "A");
        assert_eq!(capitalize_first("abc def"), "Abc def");
    }

    #[test]
    fn all_words() {
        assert_eq!(capitalize(""), "");
        assert_eq!(capitalize("one two three"), "One Two Three");
        assert_eq!(capitalize("  leading"), "  Leading");
    }
}
