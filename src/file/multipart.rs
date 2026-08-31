//! Ports `ra.common.file.Multipart` — a `multipart/form-data` body builder.
//!
//! Like the Java version (whose HTTP transport was commented out) this only
//! accumulates the body string; sending it is the caller's concern.

use serde::{Deserialize, Serialize};

use crate::util::random::random_alphanumeric;

const LINE_FEED: &str = "\r\n";

/// Accumulates a `multipart/form-data` request body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Multipart {
    pub boundary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charset: Option<String>,
    #[serde(skip)]
    body: String,
}

impl Default for Multipart {
    fn default() -> Self {
        Multipart {
            boundary: random_alphanumeric(32),
            charset: None,
            body: String::new(),
        }
    }
}

impl Multipart {
    /// A new builder with a random boundary and the given charset.
    pub fn new(charset: impl Into<String>) -> Self {
        Multipart {
            charset: Some(charset.into()),
            ..Default::default()
        }
    }

    /// Add a simple form field.
    pub fn add_form_field(&mut self, name: &str, value: &str) {
        let charset = self.charset.clone().unwrap_or_else(|| "UTF-8".to_string());
        self.body.push_str(&format!("--{}{LINE_FEED}", self.boundary));
        self.body
            .push_str(&format!("Content-Disposition: form-data; name=\"{name}\"{LINE_FEED}"));
        self.body
            .push_str(&format!("Content-Type: text/plain; charset={charset}{LINE_FEED}{LINE_FEED}"));
        self.body.push_str(value);
        self.body.push_str(LINE_FEED);
    }

    /// Add a file part header (the caller appends the bytes and a trailing CRLF).
    pub fn add_file_part(&mut self, field_name: &str, file_name: Option<&str>) {
        self.body.push_str(&format!("--{}{LINE_FEED}", self.boundary));
        match file_name {
            Some(fname) => self
                .body
                .push_str(&format!("Content-Disposition: file; filename=\"{fname}\"{LINE_FEED}")),
            None => self
                .body
                .push_str(&format!("Content-Disposition: file; name=\"{field_name}\";{LINE_FEED}")),
        }
        self.body.push_str(&format!("Content-Type: application/octet-stream{LINE_FEED}"));
        self.body
            .push_str(&format!("Content-Transfer-Encoding: binary{LINE_FEED}{LINE_FEED}"));
    }

    /// Add a raw header line.
    pub fn add_header_field(&mut self, name: &str, value: &str) {
        self.body.push_str(&format!("{name}: {value}{LINE_FEED}"));
    }

    /// Append raw text to the body.
    pub fn append_raw(&mut self, text: &str) {
        self.body.push_str(text);
    }

    /// Close the body and return it.
    pub fn finish(mut self) -> String {
        self.body.push_str(&format!("--{}--{LINE_FEED}", self.boundary));
        self.body
    }

    /// The body accumulated so far (without the closing boundary).
    pub fn body(&self) -> &str {
        &self.body
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_form_body() {
        let mut m = Multipart::new("UTF-8");
        m.add_form_field("a", "1");
        let boundary = m.boundary.clone();
        let out = m.finish();
        assert!(out.contains(&format!("--{boundary}")));
        assert!(out.contains("name=\"a\""));
        assert!(out.trim_end().ends_with(&format!("--{boundary}--")));
    }
}
