//! File-adjacent helpers. Phase 1 ports only `Multipart`; `FileUtil`,
//! `InfoVault*` and the secure-file wrappers are deferred.

pub mod multipart;

pub use multipart::Multipart;
