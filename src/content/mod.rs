//! Typed content submitted to the network for dissemination.
//!
//! Ports `ra.common.content.Content` and its subclasses (`Text`, `HTML`, `JSON`,
//! `Binary`, `Image`, `Audio`, `Video`). The Java class hierarchy collapses into
//! one [`Content`] struct tagged with a [`ContentKind`].

use serde::{Deserialize, Serialize};

use crate::crypto::hash_util;
use crate::crypto::{EncryptionAlgorithm, Hash, HashAlgorithm};
use crate::error::Result;
use crate::util::random::random_alphanumeric;

fn now_millis() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// The flavour of a [`Content`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentKind {
    Text,
    Html,
    Json,
    Image,
    Audio,
    Video,
    Binary,
}

impl ContentKind {
    /// Best-guess kind for a MIME type, matching `Content.buildContent`.
    pub fn for_content_type(content_type: &str) -> Option<ContentKind> {
        if content_type.starts_with("text/plain") {
            Some(ContentKind::Text)
        } else if content_type.starts_with("text/html") {
            Some(ContentKind::Html)
        } else if content_type.starts_with("application/json") {
            Some(ContentKind::Json)
        } else if content_type.starts_with("image/") {
            Some(ContentKind::Image)
        } else if content_type.starts_with("audio/") {
            Some(ContentKind::Audio)
        } else if content_type.starts_with("video/") {
            Some(ContentKind::Video)
        } else {
            None
        }
    }

    /// Whether the body is stored/serialized as text rather than base64.
    pub fn is_text(&self) -> bool {
        matches!(self, ContentKind::Text | ContentKind::Html | ContentKind::Json)
    }
}

/// A unit of content plus its metadata (hashes, encryption info, keywords, a
/// child tree).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    #[serde(rename = "type")]
    pub kind: ContentKind,
    pub content_type: String,
    #[serde(default)]
    pub version: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(default)]
    pub size: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_alias: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_address: Option<String>,
    /// The payload bytes. `None` means "metadata only".
    #[serde(default, skip_serializing_if = "Option::is_none", with = "opt_base64")]
    pub body: Option<Vec<u8>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_encoding: Option<String>,
    #[serde(default)]
    pub body_base64_encoded: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<Hash>,
    pub hash_algorithm: HashAlgorithm,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<Hash>,
    pub fingerprint_algorithm: HashAlgorithm,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Content>,
    #[serde(default)]
    pub encrypted: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encryption_algorithm: Option<EncryptionAlgorithm>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encryption_passphrase: Option<String>,
    #[serde(default)]
    pub encryption_passphrase_encrypted: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encryption_passphrase_algorithm: Option<EncryptionAlgorithm>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base64_encoded_iv: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
    /// Everyone has read access (e.g. an article).
    #[serde(default)]
    pub readable: bool,
    /// Everyone has write access (e.g. a wiki).
    #[serde(default)]
    pub writeable: bool,
}

impl Content {
    /// An empty content of the given kind and MIME type.
    pub fn new(kind: ContentKind, content_type: impl Into<String>) -> Self {
        Content {
            kind,
            content_type: content_type.into(),
            version: 0,
            id: None,
            label: None,
            name: None,
            location: None,
            size: 0,
            author_alias: None,
            author_address: None,
            body: None,
            body_encoding: None,
            body_base64_encoded: false,
            created_at: None,
            hash: None,
            hash_algorithm: HashAlgorithm::Sha256,
            fingerprint: None,
            fingerprint_algorithm: HashAlgorithm::Sha1,
            children: Vec::new(),
            encrypted: false,
            encryption_algorithm: None,
            encryption_passphrase: None,
            encryption_passphrase_encrypted: false,
            encryption_passphrase_algorithm: None,
            base64_encoded_iv: None,
            keywords: Vec::new(),
            readable: false,
            writeable: false,
        }
    }

    /// Build content from a body and MIME type, assigning an id and `created_at`
    /// and optionally computing the hash / fingerprint. Ports
    /// `Content.buildContent`.
    ///
    /// # Errors
    /// [`crate::error::RaError`] if `content_type` maps to no known kind or hash
    /// generation fails.
    pub fn build(
        body: Vec<u8>,
        content_type: &str,
        label: Option<String>,
        name: Option<String>,
        generate_hash: bool,
        generate_fingerprint: bool,
    ) -> Result<Self> {
        let kind = ContentKind::for_content_type(content_type)
            .ok_or_else(|| crate::error::RaError::invalid(format!("unsupported content type: {content_type}")))?;
        let mut c = Content::new(kind, content_type);
        c.label = label;
        c.name = name;
        if let Some(cs) = content_type.split("charset:").nth(1) {
            c.body_encoding = Some(cs.to_string());
        }
        c.set_body(body, generate_hash, generate_fingerprint)?;
        c.created_at = Some(now_millis());
        c.id = Some(random_alphanumeric(32));
        Ok(c)
    }

    /// Replace the body, update `size`, bump `version`, and (optionally) recompute
    /// the hash and fingerprint.
    ///
    /// # Errors
    /// Propagates hashing errors.
    pub fn set_body(
        &mut self,
        body: Vec<u8>,
        generate_hash: bool,
        generate_fingerprint: bool,
    ) -> Result<()> {
        self.size = body.len() as i64;
        if generate_hash {
            let h = hash_util::generate_hash(&body, self.hash_algorithm)?;
            self.hash = Some(Hash::new(h, self.hash_algorithm));
        }
        if generate_fingerprint {
            if let Some(h) = &self.hash {
                let fp = hash_util::generate_fingerprint(h.hash.as_bytes(), self.fingerprint_algorithm)?;
                self.fingerprint = Some(Hash::new(fp, self.fingerprint_algorithm));
            }
        }
        self.body = Some(body);
        self.version += 1;
        Ok(())
    }

    /// True if there is no body (metadata only).
    pub fn meta_only(&self) -> bool {
        self.body.is_none()
    }

    /// Add a keyword.
    pub fn add_keyword(&mut self, keyword: impl Into<String>) {
        self.keywords.push(keyword.into());
    }

    /// Add a child content.
    pub fn add_child(&mut self, child: Content) {
        self.children.push(child);
    }

    /// A [magnet URI](https://en.wikipedia.org/wiki/Magnet_URI_scheme) for this
    /// content, or `None` if there is nothing to describe.
    pub fn magnet_link(&self) -> Option<String> {
        let mut parts: Vec<String> = Vec::new();
        if let Some(body) = &self.body {
            parts.push(format!("xl={}", body.len()));
        }
        if let Some(h) = &self.hash {
            parts.push(format!(
                "xt=urn:{}:{}",
                h.algorithm.as_str().to_lowercase(),
                h.hash
            ));
        }
        if !self.keywords.is_empty() {
            parts.push(format!("kt={}", self.keywords.join("+")));
        }
        if parts.is_empty() {
            None
        } else {
            Some(format!("magnet:?{}", parts.join("&")))
        }
    }
}

/// serde helper: `Option<Vec<u8>>` <-> base64 string.
mod opt_base64 {
    use base64::Engine;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &Option<Vec<u8>>, s: S) -> Result<S::Ok, S::Error> {
        match v {
            Some(bytes) => s.serialize_str(&base64::engine::general_purpose::STANDARD.encode(bytes)),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Vec<u8>>, D::Error> {
        let opt = Option::<String>::deserialize(d)?;
        match opt {
            None => Ok(None),
            Some(s) => base64::engine::general_purpose::STANDARD
                .decode(s)
                .map(Some)
                .map_err(serde::de::Error::custom),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_text_content() {
        let c = Content::build(b"hello".to_vec(), "text/plain", None, Some("greeting".into()), true, true)
            .unwrap();
        assert_eq!(c.kind, ContentKind::Text);
        assert_eq!(c.size, 5);
        assert!(c.id.is_some());
        assert!(c.hash.is_some());
        assert!(c.fingerprint.is_some());
        assert_eq!(c.version, 1);
    }

    #[test]
    fn unsupported_type_errors() {
        assert!(Content::build(vec![], "application/x-tar", None, None, false, false).is_err());
    }

    #[test]
    fn magnet_link_shape() {
        let mut c = Content::new(ContentKind::Binary, "application/octet-stream");
        c.set_body(vec![1, 2, 3, 4], true, false).unwrap();
        c.add_keyword("alpha");
        c.add_keyword("beta");
        let m = c.magnet_link().unwrap();
        assert!(m.starts_with("magnet:?xl=4"));
        assert!(m.contains("kt=alpha+beta"));
    }

    #[test]
    fn json_round_trip() {
        let c = Content::build(b"{}".to_vec(), "application/json", None, None, false, false).unwrap();
        let json = serde_json::to_string(&c).unwrap();
        let back: Content = serde_json::from_str(&json).unwrap();
        assert_eq!(back.kind, ContentKind::Json);
        assert_eq!(back.body.unwrap(), b"{}");
    }
}
