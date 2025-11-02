use std::sync::OnceLock;

use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum EnvError {
    #[error("Environment variable {var} must be set")]
    Missing { var: &'static str },
    #[error("Environment variable {var} is invalid: {reason}")]
    Invalid { var: &'static str, reason: String },
}

#[derive(Debug, Error)]
#[error("Environment validation failed:\n{details}")]
pub struct EnvValidationError {
    details: String,
}

impl EnvValidationError {
    pub fn from_errors(errors: Vec<EnvError>) -> Self {
        let details = errors
            .into_iter()
            .map(|err| format!("- {}", err))
            .collect::<Vec<_>>()
            .join("\n");
        Self { details }
    }
}

pub struct EnvStore<T> {
    base_name: &'static str,
    resolved_name: OnceLock<&'static str>,
    value: OnceLock<T>,
}

impl<T> EnvStore<T> {
    pub const fn new(base_name: &'static str) -> Self {
        Self {
            base_name,
            resolved_name: OnceLock::new(),
            value: OnceLock::new(),
        }
    }

    fn resolved_name(&self) -> &'static str {
        *self.resolved_name.get_or_init(|| {
            if self.base_name.starts_with("DEV_") || self.base_name.starts_with("PROD_") {
                self.base_name
            } else {
                let prefix = if cfg!(debug_assertions) {
                    "DEV_"
                } else {
                    "PROD_"
                };
                Box::leak(format!("{}{}", prefix, self.base_name).into_boxed_str())
            }
        })
    }

    pub fn is_active(&self) -> bool {
        if self.base_name.starts_with("DEV_") {
            cfg!(debug_assertions)
        } else if self.base_name.starts_with("PROD_") {
            cfg!(not(debug_assertions))
        } else {
            true
        }
    }

    pub fn set(&self, value: T) {
        if !self.is_active() {
            return;
        }
        let name = self.resolved_name();
        if self.value.set(value).is_err() {
            panic!("Environment variable {} has already been initialized", name);
        }
    }

    pub fn get(&self) -> &T {
        if !self.is_active() {
            panic!(
                "Environment variable {} is not active in this build configuration",
                self.base_name
            );
        }
        let name = self.resolved_name();
        self.value.get().unwrap_or_else(|| {
            panic!(
                "Environment variable {} should have been initialized during startup validation",
                name
            )
        })
    }

    pub fn name(&self) -> &'static str {
        self.resolved_name()
    }
}
