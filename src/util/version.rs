//! Loose version-string comparison. Ports `ra.common.VersionComparator`.
//!
//! Segments are separated by `.`, `_` or `-`. Within a segment every non-digit
//! character is ignored (`"8-ea"` parses as `8`). Segments are compared
//! numerically, left to right; the first difference decides. A string that runs
//! out of segments first sorts lower.

use std::cmp::Ordering;

const fn is_separator(c: u8) -> bool {
    matches!(c, b'.' | b'_' | b'-')
}

/// Index of the next separator at or after `start`, or the string length.
fn next_separator(s: &[u8], mut start: usize) -> usize {
    while start < s.len() {
        if is_separator(s[start]) {
            return start;
        }
        start += 1;
    }
    start
}

/// Parse `s[start..end]` as an `i64`, ignoring non-digits. Returns `-1` if no
/// digit was seen or the value overflowed to negative (matching the Java guard).
fn parse_long(s: &[u8], start: usize, end: usize) -> i64 {
    let mut rv: i64 = 0;
    let mut parsed_any = false;
    let mut i = start;
    while i < end && rv >= 0 {
        let c = s[i];
        if c.is_ascii_digit() {
            parsed_any = true;
            rv = rv.wrapping_mul(10).wrapping_add((c - b'0') as i64);
        }
        i += 1;
    }
    if parsed_any {
        rv
    } else {
        -1
    }
}

/// Compare two version strings. See the module docs for the (deliberately loose)
/// semantics.
///
/// This is a straight port of the Java comparator, including its quirks:
/// - a shorter string sorts lower once its segments are exhausted, so
///   `"2.0" < "2.0.0"`;
/// - non-digits are only skipped *within* a segment that is actually compared,
///   so `"8ea" == "8"` but `"8-ea" > "8"` (the trailing `ea` is an extra
///   segment).
///
/// ```
/// # use ra_common::util::version::version_compare;
/// # use std::cmp::Ordering;
/// assert_eq!(version_compare("1.8", "1.11"), Ordering::Less);
/// assert_eq!(version_compare("2.0", "2.0"), Ordering::Equal);
/// assert_eq!(version_compare("2.0", "2.0.0"), Ordering::Less);
/// assert_eq!(version_compare("8ea", "8"), Ordering::Equal);
/// assert_eq!(version_compare("1.8.0_275", "1.8.0_271"), Ordering::Greater);
/// ```
pub fn version_compare(l: &str, r: &str) -> Ordering {
    if l == r {
        return Ordering::Equal;
    }
    let (lb, rb) = (l.as_bytes(), r.as_bytes());
    let (ll, rl) = (lb.len(), rb.len());
    let (mut il, mut ir) = (0usize, 0usize);

    loop {
        if il >= ll {
            return if ir >= rl {
                Ordering::Equal
            } else {
                Ordering::Less
            };
        } else if ir >= rl {
            return Ordering::Greater;
        }

        let mut lv: i64 = -1;
        while lv == -1 && il < ll {
            let nl = next_separator(lb, il);
            lv = parse_long(lb, il, nl);
            il = nl + 1;
        }

        let mut rv: i64 = -1;
        while rv == -1 && ir < rl {
            let nr = next_separator(rb, ir);
            rv = parse_long(rb, ir, nr);
            ir = nr + 1;
        }

        match lv.cmp(&rv) {
            Ordering::Equal => continue,
            other => return other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering::*;

    #[test]
    fn basics() {
        assert_eq!(version_compare("1.0", "1.0"), Equal);
        assert_eq!(version_compare("1.0", "1.1"), Less);
        assert_eq!(version_compare("1.2", "1.1"), Greater);
        assert_eq!(version_compare("2.0", "2.0.0"), Less);
        assert_eq!(version_compare("2.0.1", "2.0"), Greater);
    }

    #[test]
    fn ignores_non_digits() {
        assert_eq!(version_compare("8ea", "8"), Equal);
        assert_eq!(version_compare("8-ea", "8"), Greater); // extra segment
        assert_eq!(version_compare("1.8.0_275", "1.8.0_271"), Greater);
    }
}
