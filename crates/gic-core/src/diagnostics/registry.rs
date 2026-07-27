//! Registry management for validation rules, language validators, and rule configurations.

use crate::diagnostics::errors::{DiagnosticError, DiagnosticResult};
use crate::diagnostics::rule::{Rule, RuleCategory, RuleConfig, RulePriority};
use crate::diagnostics::validator::Validator;
use crate::parser::language::LanguageId;
use std::collections::HashMap;
use std::sync::Arc;

/// Registry holding registered rules and their active configurations.
#[derive(Default)]
pub struct RuleRegistry {
    rules: HashMap<String, Arc<dyn Rule>>,
    configs: HashMap<String, RuleConfig>,
}

impl RuleRegistry {
    /// Creates a new empty `RuleRegistry`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a validation rule into the registry.
    pub fn register(&mut self, rule: impl Rule + 'static) {
        let meta = rule.metadata();
        let id = meta.id.clone();
        self.rules.insert(id.clone(), Arc::new(rule));
        self.configs.entry(id).or_insert_with(RuleConfig::enabled);
    }

    /// Configures a rule's active runtime settings.
    pub fn set_config(&mut self, rule_id: impl Into<String>, config: RuleConfig) {
        self.configs.insert(rule_id.into(), config);
    }

    /// Enables or disables a rule by ID.
    pub fn set_enabled(&mut self, rule_id: &str, enabled: bool) -> DiagnosticResult<()> {
        let config = self
            .configs
            .get_mut(rule_id)
            .ok_or_else(|| DiagnosticError::RuleNotFound(rule_id.to_string()))?;
        config.enabled = enabled;
        Ok(())
    }

    /// Retrieves a rule by ID.
    pub fn get_rule(&self, rule_id: &str) -> Option<Arc<dyn Rule>> {
        self.rules.get(rule_id).cloned()
    }

    /// Returns configuration for a rule.
    pub fn get_config(&self, rule_id: &str) -> Option<&RuleConfig> {
        self.configs.get(rule_id)
    }

    /// Returns all active enabled rules supporting the target language, sorted by priority.
    pub fn get_rules_for_language(&self, language: LanguageId) -> Vec<Arc<dyn Rule>> {
        let mut active_rules = Vec::new();

        for (id, rule) in &self.rules {
            if let Some(config) = self.configs.get(id) {
                if config.enabled && rule.supports_language(language.clone()) {
                    active_rules.push(rule.clone());
                }
            }
        }

        // Sort rules by priority (highest priority evaluated first)
        active_rules.sort_by(|a, b| {
            let p_a = self.get_effective_priority(a.as_ref());
            let p_b = self.get_effective_priority(b.as_ref());
            p_b.cmp(&p_a)
        });

        active_rules
    }

    /// Returns rules filtered by category.
    pub fn get_rules_by_category(&self, category: &RuleCategory) -> Vec<Arc<dyn Rule>> {
        self.rules
            .values()
            .filter(|r| &r.metadata().category == category)
            .cloned()
            .collect()
    }

    /// Computes effective priority considering overrides.
    fn get_effective_priority(&self, rule: &dyn Rule) -> RulePriority {
        let meta = rule.metadata();
        if let Some(cfg) = self.configs.get(&meta.id) {
            cfg.effective_priority(meta.priority)
        } else {
            meta.priority
        }
    }
}

/// Registry managing language-specific and multi-language validators.
#[derive(Default)]
pub struct ValidatorRegistry {
    validators: HashMap<LanguageId, Vec<Arc<dyn Validator>>>,
    global_validators: Vec<Arc<dyn Validator>>,
}

impl ValidatorRegistry {
    /// Creates a new `ValidatorRegistry`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a validator for a specific language.
    pub fn register(&mut self, validator: impl Validator + 'static) {
        let lang = validator.language();
        self.validators
            .entry(lang)
            .or_default()
            .push(Arc::new(validator));
    }

    /// Registers a global validator that runs across all languages.
    pub fn register_global(&mut self, validator: impl Validator + 'static) {
        self.global_validators.push(Arc::new(validator));
    }

    /// Retrieves all validators registered for a language (including global validators).
    pub fn get_validators(&self, language: LanguageId) -> Vec<Arc<dyn Validator>> {
        let mut list = self.global_validators.clone();
        if let Some(lang_validators) = self.validators.get(&language) {
            list.extend(lang_validators.iter().cloned());
        }
        list
    }
}

/// Loader for dynamic or external rule definitions.
pub struct RuleLoader;

impl RuleLoader {
    /// Constructs default core rule set.
    pub fn load_default_rules() -> RuleRegistry {
        RuleRegistry::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::diagnostic::Diagnostic;
    use crate::diagnostics::rule::RuleMetadata;
    use crate::diagnostics::severity::DiagnosticLevel;
    use crate::diagnostics::validator::ValidationContext;
    use crate::parser::tree::SyntaxTree;

    struct TestRule {
        meta: RuleMetadata,
    }

    impl Rule for TestRule {
        fn metadata(&self) -> &RuleMetadata {
            &self.meta
        }

        fn evaluate(
            &self,
            _tree: &SyntaxTree,
            _ctx: &ValidationContext,
        ) -> DiagnosticResult<Vec<Diagnostic>> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn test_rule_and_validator_registries() {
        let mut rule_reg = RuleRegistry::new();
        let rule = TestRule {
            meta: RuleMetadata::new(
                "R-01",
                "Rule 1",
                RuleCategory::Security,
                DiagnosticLevel::Error,
            ),
        };

        rule_reg.register(rule);
        assert!(rule_reg.get_rule("R-01").is_some());
        assert_eq!(rule_reg.get_rules_for_language(LanguageId::Yaml).len(), 1);

        rule_reg.set_enabled("R-01", false).unwrap();
        assert_eq!(rule_reg.get_rules_for_language(LanguageId::Yaml).len(), 0);
    }
}
