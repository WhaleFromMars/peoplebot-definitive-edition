use crate::prelude::*;
use std::sync::OnceLock;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum EnvError {
    #[error("Environment variable {var} must be set")]
    Missing { var: &'static str },
    #[error("Environment variable {var} is invalid: {reason}")]
    Invalid { var: &'static str, reason: String },
    #[error("Environment variable {var} is already set")]
    AlreadySet { var: &'static str },
}

#[derive(Debug, Error)]
#[error("Environment validation failed:\n{details}")]
pub(crate) struct EnvValidationError {
    details: String,
}

impl EnvValidationError {
    pub(crate) fn from_errors(errors: Vec<EnvError>) -> Self {
        let details = errors
            .into_iter()
            .map(|err| format!("- {}", err))
            .collect::<Vec<_>>()
            .join("\n");
        Self { details }
    }
}

pub(crate) struct EnvStore<T> {
    base_key: &'static str,
    value: OnceLock<T>,
}

impl<T> EnvStore<T> {
    pub(crate) const fn new(base_key: &'static str) -> Self {
        Self {
            base_key,
            value: OnceLock::new(),
        }
    }

    pub(crate) const fn base_key(&self) -> &'static str {
        self.base_key
    }

    pub(crate) fn set(&self, value: T) -> Result<(), EnvError> {
        self.value
            .set(value)
            .map_err(|_| EnvError::AlreadySet { var: self.base_key })
    }

    /// Get the initialized value (panics if not set).
    pub(crate) fn get(&self) -> &T {
        self.value.get().unwrap_or_else(|| {
            panic!(
                "Environment variable {} should have been initialized during startup validation",
                self.base_key
            )
        })
    }
}

#[inline]
fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

#[inline]
fn is_dev(s: &str) -> bool {
    s.starts_with("DEV_")
}
#[inline]
fn is_prod(s: &str) -> bool {
    s.starts_with("PROD_")
}
#[inline]
fn is_both(s: &str) -> bool {
    s.starts_with("BOTH_")
}
#[inline]
fn has_any_prefix(s: &str) -> bool {
    is_dev(s) || is_prod(s) || is_both(s)
}

/// Should this key be validated in the current build?
/// - DEV_*  => only in debug
/// - PROD_* => only in release
/// - BOTH_* => always
/// - unprefixed => always (since we’ll resolve to DEV_/PROD_/BOTH_)
pub(crate) fn active_for_build(base_key: &'static str) -> bool {
    if is_dev(base_key) {
        cfg!(debug_assertions)
    } else if is_prod(base_key) {
        cfg!(not(debug_assertions))
    } else if is_both(base_key) {
        true
    } else {
        true
    }
}

/// if `name` is unprefixed, synthesize the *prefixed* expected key for the build (not including BOTH_)
pub(crate) fn prefixed_key_for(base_key: &'static str) -> &'static str {
    if has_any_prefix(base_key) {
        return base_key;
    }
    let pref = if cfg!(debug_assertions) {
        "DEV_"
    } else {
        "PROD_"
    };
    leak(format!("{pref}{base_key}"))
}

/// - If `name` is prefixed (DEV_/PROD_/BOTH_), read exactly that (if present).
/// - If `name` is unprefixed:
///     * debug: check DEV_<name>, else BOTH_<name>
///     * release: check PROD_<name>, else BOTH_<name>
pub(crate) fn pick_existing_key(name: &'static str) -> Option<&'static str> {
    if has_any_prefix(name) {
        return env::var_os(name).map(|_| name);
    }

    let (primary, fallback) = if cfg!(debug_assertions) {
        (format!("DEV_{name}"), format!("BOTH_{name}"))
    } else {
        (format!("PROD_{name}"), format!("BOTH_{name}"))
    };

    if env::var_os(&primary).is_some() {
        Some(leak(primary))
    } else if env::var_os(&fallback).is_some() {
        Some(leak(fallback))
    } else {
        None
    }
}

pub(crate) trait EnvTarget<U> {
    const OPTIONAL: bool;

    fn base_key(&self) -> &'static str;

    /// Set a present, parsed value.
    fn set_some(&'static self, v: U) -> Result<(), EnvError>;

    /// Set the absence of a value; only meaningful when OPTIONAL==true.
    fn set_none(&'static self) -> Result<(), EnvError>;
}

// Non Optional EnvStore
impl<U> EnvTarget<U> for EnvStore<U> {
    const OPTIONAL: bool = false;

    #[inline]
    fn base_key(&self) -> &'static str {
        self.base_key()
    }
    #[inline]
    fn set_some(&'static self, v: U) -> Result<(), EnvError> {
        self.set(v)
    }
    #[inline]
    fn set_none(&'static self) -> Result<(), EnvError> {
        Err(EnvError::Invalid {
            var: self.base_key(),
            reason: "cannot set None for non-optional environment variable".into(),
        })
    }
}

// Optional EnvStore
impl<U> EnvTarget<U> for EnvStore<Option<U>> {
    const OPTIONAL: bool = true;

    #[inline]
    fn base_key(&self) -> &'static str {
        self.base_key()
    }
    #[inline]
    fn set_some(&'static self, v: U) -> Result<(), EnvError> {
        self.set(Some(v))
    }
    #[inline]
    fn set_none(&'static self) -> Result<(), EnvError> {
        self.set(None)
    }
}

#[inline]
fn already_init(var: &'static str) -> EnvError {
    EnvError::Invalid {
        var,
        reason: "already initialized".into(),
    }
}

#[inline]
fn not_unicode(var: &'static str, val: std::ffi::OsString) -> EnvError {
    EnvError::Invalid {
        var,
        reason: format!("value is not valid UTF-8: {}", val.to_string_lossy()),
    }
}

/// Single validator used by both macro arms.
/// - If OPTIONAL: missing/empty ⇒ store None and return Ok.
/// - If REQUIRED: missing ⇒ Missing error.
/// - UTF-8 / parse errors always error.
pub(crate) async fn validate_env<S, U>(store: &'static S) -> Result<(), EnvError>
where
    S: EnvTarget<U>,
    U: FromStr,
    <U as FromStr>::Err: Display,
{
    let base_key = store.base_key();

    if !active_for_build(base_key) {
        return Ok(());
    }

    let key_for_error = prefixed_key_for(base_key);
    let chosen = pick_existing_key(base_key);

    if chosen.is_none() {
        if S::OPTIONAL {
            store.set_none().map_err(|_| already_init(key_for_error))?;
            return Ok(());
        } else {
            return Err(EnvError::Missing { var: key_for_error });
        }
    }

    let present_key = chosen.unwrap();

    let raw = match env::var(present_key) {
        Ok(v) => v,
        Err(std::env::VarError::NotUnicode(os)) => return Err(not_unicode(key_for_error, os)),
        Err(std::env::VarError::NotPresent) => {
            if S::OPTIONAL {
                store.set_none().map_err(|_| already_init(key_for_error))?;
                return Ok(());
            } else {
                return Err(EnvError::Missing { var: key_for_error });
            }
        }
    };

    // Empty => treat as None for OPTIONAL, Error for REQUIRED
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        if S::OPTIONAL {
            store.set_none().map_err(|_| already_init(key_for_error))?;
            return Ok(());
        } else {
            return Err(EnvError::Missing { var: key_for_error });
        }
    }

    let parsed = trimmed.parse::<U>().map_err(|e| EnvError::Invalid {
        var: key_for_error,
        reason: e.to_string(),
    })?;

    store
        .set_some(parsed)
        .map_err(|_| already_init(key_for_error))?;
    Ok(())
}
