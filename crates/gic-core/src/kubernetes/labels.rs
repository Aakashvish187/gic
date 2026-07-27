//! Kubernetes Label Syntax Validation Engine.
//!
//! Validates Kubernetes label keys and values against official DNS label standards
//! (RFC 1123 / RFC 952 guidelines, max 63 chars, valid prefix domains).

use std::collections::HashMap;

/// Type alias for label key-value pairs.
pub type LabelMap = HashMap<String, String>;

/// Defect found during label syntax evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelIssue {
    /// Target label key.
    pub key: String,
    /// Detailed message explaining invalid syntax.
    pub message: String,
}

/// Evaluator for checking label syntax validity.
#[derive(Debug, Clone, Default)]
pub struct LabelValidator;

impl LabelValidator {
    /// Creates a new LabelValidator.
    pub fn new() -> Self {
        Self
    }

    /// Validates an entire label map against Kubernetes label constraints.
    pub fn validate(&self, labels: &LabelMap) -> Vec<LabelIssue> {
        let mut issues = Vec::new();

        for (key, val) in labels {
            if let Err(msg) = self.validate_key(key) {
                issues.push(LabelIssue {
                    key: key.clone(),
                    message: format!("Invalid label key '{key}': {msg}"),
                });
            }
            if let Err(msg) = self.validate_value(val) {
                issues.push(LabelIssue {
                    key: key.clone(),
                    message: format!("Invalid label value '{val}' for key '{key}': {msg}"),
                });
            }
        }

        issues
    }

    /// Validates a label key (optional domain prefix / name <= 63 chars).
    pub fn validate_key(&self, key: &str) -> Result<(), &'static str> {
        if key.is_empty() {
            return Err("Label key cannot be empty");
        }

        let (prefix, name) = if let Some(slash_idx) = key.find('/') {
            let p = &key[..slash_idx];
            let n = &key[slash_idx + 1..];
            if p.is_empty() {
                return Err("Prefix subdomain cannot be empty before '/'");
            }
            if p.len() > 253 {
                return Err("Prefix subdomain exceeds maximum length of 253 characters");
            }
            (Some(p), n)
        } else {
            (None, key)
        };

        if name.is_empty() {
            return Err("Label name component cannot be empty");
        }
        if name.len() > 63 {
            return Err("Label name component exceeds maximum length of 63 characters");
        }

        if !name.chars().next().unwrap().is_alphanumeric()
            || !name.chars().last().unwrap().is_alphanumeric()
        {
            return Err("Label name must start and end with an alphanumeric character");
        }

        for ch in name.chars() {
            if !ch.is_alphanumeric() && ch != '-' && ch != '_' && ch != '.' {
                return Err("Label name contains illegal character (allowed: [a-zA-Z0-9-_.]");
            }
        }

        if let Some(p) = prefix {
            for part in p.split('.') {
                if part.is_empty() {
                    return Err("Subdomain parts cannot be empty");
                }
                for ch in part.chars() {
                    if !ch.is_alphanumeric() && ch != '-' {
                        return Err("Prefix subdomain contains illegal character");
                    }
                }
            }
        }

        Ok(())
    }

    /// Validates a label value (<= 63 chars, alphanumeric start/end).
    pub fn validate_value(&self, val: &str) -> Result<(), &'static str> {
        if val.is_empty() {
            return Ok(());
        }

        if val.len() > 63 {
            return Err("Label value exceeds maximum length of 63 characters");
        }

        if !val.chars().next().unwrap().is_alphanumeric()
            || !val.chars().last().unwrap().is_alphanumeric()
        {
            return Err("Label value must start and end with an alphanumeric character");
        }

        for ch in val.chars() {
            if !ch.is_alphanumeric() && ch != '-' && ch != '_' && ch != '.' {
                return Err("Label value contains illegal character");
            }
        }

        Ok(())
    }
}
