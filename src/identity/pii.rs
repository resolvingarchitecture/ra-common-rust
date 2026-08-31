//! Ports `ra.common.identity.PIIClearable`.

/// Implemented by types carrying personally-identifiable information, providing
/// a way to scrub it (typically before handing a copy to another party).
pub trait PiiClearable {
    /// Null out / reset every field that carries PII.
    fn clear_sensitive(&mut self);
}
