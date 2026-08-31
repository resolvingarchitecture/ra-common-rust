//! Configuration loading and cross-platform directory resolution.
//!
//! Ports `ra.common.Config` and `ra.common.SystemSettings`. The JVM-specific
//! "system variables" (`System.getProperties()`) source is dropped; environment
//! variables and a `.properties`/`.config` file are the sources.

use std::path::{Path, PathBuf};

use crate::error::{RaError, Result};
use crate::Properties;

/// Parse `key<delimiter>value` arguments into [`Properties`]. Arguments without
/// the delimiter are ignored (matching the Java behaviour).
pub fn load_from_args(args: &[String], delimiter: char) -> Properties {
    let mut props = Properties::new();
    for arg in args {
        if let Some((k, v)) = arg.split_once(delimiter) {
            props.insert(k.to_string(), v.to_string());
        }
    }
    props
}

/// Snapshot the process environment as [`Properties`].
pub fn load_from_env() -> Properties {
    std::env::vars().collect()
}

/// Load a `.properties` / `.config` file: `key=value` lines, `#` or `!` comment
/// lines, blank lines ignored.
///
/// # Errors
/// [`RaError::Io`] if the file cannot be read.
pub fn load_from_file(path: impl AsRef<Path>) -> Result<Properties> {
    let text = std::fs::read_to_string(path.as_ref()).map_err(RaError::Io)?;
    Ok(parse_properties(&text))
}

/// Parse `.properties`-style text.
pub fn parse_properties(text: &str) -> Properties {
    let mut props = Properties::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=').or_else(|| line.split_once(':')) {
            props.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    props
}

/// Merge sources with increasing precedence: environment, then the optional
/// config file, then `client_props`.
///
/// # Errors
/// Propagates [`load_from_file`] errors when `config_path` is `Some`.
pub fn load_all(client_props: &Properties, config_path: Option<&Path>) -> Result<Properties> {
    let mut config = load_from_env();
    if let Some(path) = config_path {
        config.extend(load_from_file(path)?);
    }
    config.extend(client_props.clone());
    Ok(config)
}

/// XDG-style, cross-platform application directory resolution. Ports
/// `ra.common.SystemSettings`.
pub struct SystemSettings;

impl SystemSettings {
    /// The user's home directory (`$HOME`, or `$USERPROFILE` on Windows).
    pub fn user_home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
    }

    fn xdg_dir(env_key: &str, default_suffix: &str) -> Option<PathBuf> {
        if let Some(v) = std::env::var_os(env_key) {
            return Some(PathBuf::from(v));
        }
        Self::user_home_dir().map(|h| h.join(default_suffix))
    }

    /// `$XDG_DATA_HOME` or `~/.local/share`.
    pub fn user_data_dir() -> Option<PathBuf> {
        Self::xdg_dir("XDG_DATA_HOME", ".local/share")
    }

    /// `$XDG_CONFIG_HOME` or `~/.config`.
    pub fn user_config_dir() -> Option<PathBuf> {
        Self::xdg_dir("XDG_CONFIG_HOME", ".config")
    }

    /// `$XDG_CACHE_HOME` or `~/.cache`.
    pub fn user_cache_dir() -> Option<PathBuf> {
        Self::xdg_dir("XDG_CACHE_HOME", ".cache")
    }

    /// `<base>/<group>/<app>`, optionally creating it.
    ///
    /// # Errors
    /// [`RaError::FileCreationFailed`] if `create` is set and the directory
    /// cannot be made.
    pub fn app_dir(base: &Path, group: &str, app: &str, create: bool) -> Result<PathBuf> {
        let dir = base.join(group).join(app);
        if create && !dir.exists() {
            std::fs::create_dir_all(&dir)
                .map_err(|e| RaError::FileCreationFailed(format!("{}: {e}", dir.display())))?;
        }
        Ok(dir)
    }

    /// `~/.local/share/<group>/<app>`.
    ///
    /// # Errors
    /// As [`SystemSettings::app_dir`]; also errors if the home directory is
    /// unknown.
    pub fn user_app_data_dir(group: &str, app: &str, create: bool) -> Result<PathBuf> {
        let base = Self::user_data_dir()
            .ok_or_else(|| RaError::FileCreationFailed("no user data dir".into()))?;
        Self::app_dir(&base, group, app, create)
    }

    /// `~/.config/<group>/<app>`.
    ///
    /// # Errors
    /// As [`SystemSettings::user_app_data_dir`].
    pub fn user_app_config_dir(group: &str, app: &str, create: bool) -> Result<PathBuf> {
        let base = Self::user_config_dir()
            .ok_or_else(|| RaError::FileCreationFailed("no user config dir".into()))?;
        Self::app_dir(&base, group, app, create)
    }

    /// `~/.cache/<group>/<app>`.
    ///
    /// # Errors
    /// As [`SystemSettings::user_app_data_dir`].
    pub fn user_app_cache_dir(group: &str, app: &str, create: bool) -> Result<PathBuf> {
        let base = Self::user_cache_dir()
            .ok_or_else(|| RaError::FileCreationFailed("no user cache dir".into()))?;
        Self::app_dir(&base, group, app, create)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_parsing() {
        let args = vec!["a=1".to_string(), "b=2".to_string(), "ignored".to_string()];
        let p = load_from_args(&args, '=');
        assert_eq!(p.get("a").map(String::as_str), Some("1"));
        assert_eq!(p.get("b").map(String::as_str), Some("2"));
        assert_eq!(p.len(), 2);
    }

    #[test]
    fn properties_parsing() {
        let p = parse_properties("# comment\nra.version = 1.2.0\n\nempty:\nx = y = z\n");
        assert_eq!(p.get("ra.version").map(String::as_str), Some("1.2.0"));
        assert_eq!(p.get("x").map(String::as_str), Some("y = z"));
    }

    #[test]
    fn load_all_precedence() {
        std::env::set_var("RA_COMMON_TEST_KEY", "from-env");
        let mut client = Properties::new();
        client.insert("RA_COMMON_TEST_KEY".into(), "from-client".into());
        let merged = load_all(&client, None).unwrap();
        assert_eq!(merged.get("RA_COMMON_TEST_KEY").map(String::as_str), Some("from-client"));
        std::env::remove_var("RA_COMMON_TEST_KEY");
    }
}
