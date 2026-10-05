//! Hashcash proof-of-work tokens (v0 and v1). Ports `ra.common.HashCash`.
//!
//! A token is `version:[bits:]YYMMDD:resource:ext:rand:counter`; its "value" is
//! the number of leading zero **bits** in `SHA1(token)`. Minting searches
//! `counter` until that value reaches the requested difficulty.

use std::collections::BTreeMap;
use std::str::FromStr;

use chrono::{NaiveDate, Utc};
use sha1::{Digest, Sha1};

use crate::error::{RaError, Result};

const HASH_BITS: u32 = 160;

/// A minted or parsed hashcash token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashCash {
    token: String,
    value: u32,
    resource: String,
    date: NaiveDate,
    version: u8,
    extensions: BTreeMap<String, Vec<String>>,
}

fn leading_zero_bits(bytes: &[u8]) -> u32 {
    let mut total = 0;
    for &b in bytes {
        if b == 0 {
            total += 8;
        } else {
            total += b.leading_zeros();
            break;
        }
    }
    total
}

fn sha1_bits(token: &str) -> u32 {
    leading_zero_bits(&Sha1::digest(token.as_bytes()))
}

fn serialize_extensions(ext: &BTreeMap<String, Vec<String>>) -> Result<String> {
    if ext.is_empty() {
        return Ok(String::new());
    }
    let mut out = String::new();
    for (i, (k, vs)) in ext.iter().enumerate() {
        if k.contains([':', ';', '=']) {
            return Err(RaError::invalid(format!("illegal char in extension key: {k}")));
        }
        if i > 0 {
            out.push(';');
        }
        out.push_str(k);
        if !vs.is_empty() {
            out.push('=');
            for (j, v) in vs.iter().enumerate() {
                if v.contains([':', ';', ',']) {
                    return Err(RaError::invalid(format!("illegal char in extension value: {v}")));
                }
                if j > 0 {
                    out.push(',');
                }
                out.push_str(v);
            }
        }
    }
    Ok(out)
}

fn deserialize_extensions(s: &str) -> BTreeMap<String, Vec<String>> {
    let mut map = BTreeMap::new();
    if s.is_empty() {
        return map;
    }
    for item in s.split(';') {
        match item.split_once('=') {
            None => {
                map.insert(item.to_string(), Vec::new());
            }
            Some((k, v)) => {
                map.insert(k.to_string(), v.split(',').map(str::to_string).collect());
            }
        }
    }
    map
}

impl HashCash {
    /// Mint a v1 token for `resource` at `bits` difficulty, dated today (UTC).
    ///
    /// # Errors
    /// [`RaError::Invalid`] if `resource` contains a colon or `bits > 160`.
    pub fn mint(resource: &str, bits: u32) -> Result<Self> {
        Self::mint_with(resource, BTreeMap::new(), Utc::now().date_naive(), bits, 1)
    }

    /// Mint a token with full control over extensions, date and version (0 or 1).
    ///
    /// # Errors
    /// [`RaError::Invalid`] for a bad version, `bits > 160`, or a colon in
    /// `resource`.
    pub fn mint_with(
        resource: &str,
        extensions: BTreeMap<String, Vec<String>>,
        date: NaiveDate,
        bits: u32,
        version: u8,
    ) -> Result<Self> {
        if version > 1 {
            return Err(RaError::invalid("only hashcash versions 0 and 1 are supported"));
        }
        if bits > HASH_BITS {
            return Err(RaError::invalid("value must be between 0 and 160"));
        }
        if resource.contains(':') {
            return Err(RaError::invalid("resource may not contain a colon"));
        }
        let ext_str = serialize_extensions(&extensions)?;
        let date_str = date.format("%y%m%d").to_string();
        let prefix = match version {
            0 => format!("0:{date_str}:{resource}:{ext_str}:"),
            _ => format!("1:{bits}:{date_str}:{resource}:{ext_str}:"),
        };
        let token = Self::generate(&prefix, bits);
        let value = match version {
            0 => sha1_bits(&token),
            _ => bits,
        };
        Ok(HashCash {
            token,
            value,
            resource: resource.to_string(),
            date,
            version,
            extensions,
        })
    }

    fn generate(prefix: &str, bits: u32) -> String {
        let random: u64 = rand::random();
        let mut counter: u64 = rand::random();
        let stem = format!("{prefix}{random:x}:");
        loop {
            counter = counter.wrapping_add(1);
            let candidate = format!("{stem}{counter:x}");
            if sha1_bits(&candidate) >= bits {
                return candidate;
            }
        }
    }

    /// Parse and validate a token string, recomputing its value.
    ///
    /// # Errors
    /// [`RaError::Invalid`] for an unsupported version or wrong field count.
    pub fn parse(token: &str) -> Result<Self> {
        let parts: Vec<&str> = token.split(':').collect();
        let version: u8 = parts
            .first()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| RaError::invalid("bad hashcash version"))?;
        let expected = match version {
            0 => 6,
            1 => 7,
            _ => return Err(RaError::invalid("only hashcash versions 0 and 1 are supported")),
        };
        if parts.len() != expected {
            return Err(RaError::invalid("improperly formed hashcash"));
        }
        let mut idx = 1;
        let claimed_bits = if version == 1 {
            let b = parts[idx]
                .parse::<u32>()
                .map_err(|_| RaError::invalid("bad hashcash bits"))?;
            idx += 1;
            b
        } else {
            0
        };
        let date = NaiveDate::parse_from_str(parts[idx], "%y%m%d")
            .map_err(|e| RaError::invalid(format!("bad hashcash date: {e}")))?;
        idx += 1;
        let resource = parts[idx].to_string();
        idx += 1;
        let extensions = deserialize_extensions(parts[idx]);

        let actual = sha1_bits(token);
        let value = match version {
            0 => actual,
            _ => actual.min(claimed_bits),
        };
        Ok(HashCash {
            token: token.to_string(),
            value,
            resource,
            date,
            version,
            extensions,
        })
    }

    /// Whether this token is valid for `resource` at `min_bits` difficulty.
    pub fn is_valid_for(&self, resource: &str, min_bits: u32) -> bool {
        self.resource == resource && self.computed_bits() >= min_bits
    }

    /// The leading-zero-bit count actually present in `SHA1(token)`.
    pub fn computed_bits(&self) -> u32 {
        sha1_bits(&self.token)
    }

    /// The token's declared value.
    pub fn value(&self) -> u32 {
        self.value
    }
    /// The resource string the token was minted for.
    pub fn resource(&self) -> &str {
        &self.resource
    }
    /// The token date.
    pub fn date(&self) -> NaiveDate {
        self.date
    }
    /// The token version (0 or 1).
    pub fn version(&self) -> u8 {
        self.version
    }
    /// The token extensions.
    pub fn extensions(&self) -> &BTreeMap<String, Vec<String>> {
        &self.extensions
    }
    /// The raw token string.
    pub fn token(&self) -> &str {
        &self.token
    }
}

impl std::fmt::Display for HashCash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.token)
    }
}

impl FromStr for HashCash {
    type Err = RaError;
    fn from_str(s: &str) -> Result<Self> {
        HashCash::parse(s)
    }
}

impl PartialOrd for HashCash {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HashCash {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mint_then_verify() {
        let hc = HashCash::mint("brian@resolvingarchitecture.io", 12).unwrap();
        assert!(hc.computed_bits() >= 12);
        assert!(hc.is_valid_for("brian@resolvingarchitecture.io", 12));
        assert!(!hc.is_valid_for("someone.else", 12));

        let reparsed = HashCash::parse(hc.token()).unwrap();
        assert_eq!(reparsed.resource(), hc.resource());
        assert!(reparsed.value() >= 12);
    }

    #[test]
    fn rejects_bad_shape() {
        assert!(HashCash::parse("9:bogus").is_err());
        assert!(HashCash::parse("1:20:250101:res").is_err());
        assert!(HashCash::mint("has:colon", 4).is_err());
    }

    #[test]
    fn leading_zero_count() {
        assert_eq!(leading_zero_bits(&[0x00, 0x00, 0x0F]), 20);
        assert_eq!(leading_zero_bits(&[0xFF]), 0);
        assert_eq!(leading_zero_bits(&[0x00, 0x00]), 16);
    }
}
