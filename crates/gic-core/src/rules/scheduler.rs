use super::{configuration::WorkspaceSettings, registry::RuleRegistry, rule::Rule};
use std::sync::Arc;

/// Responsible for filtering and prioritizing rules before evaluation.
pub struct RuleScheduler {
    registry: Arc<RuleRegistry>,
}

impl RuleScheduler {
    /// Creates a new RuleScheduler linked to a registry.
    pub fn new(registry: Arc<RuleRegistry>) -> Self {
        Self { registry }
    }

    /// Retrieves and orders the active rules based on the provided settings.
    pub fn get_active_rules(&self, settings: &WorkspaceSettings) -> Vec<Arc<dyn Rule>> {
        let mut rules = self.registry.get_all();

        // Filter out explicitly disabled rules
        rules.retain(|rule| {
            if settings.is_rule_disabled(&rule.metadata().id) {
                return false;
            }
            // By default, only include rules that are enabled via their metadata or profile
            // (A more advanced profile system would dynamically map this, simplified for now)
            rule.metadata().enabled
        });

        // Sort by priority (Highest first).
        // Since `Ord` is derived, a higher enum variant represents a higher value,
        // so we sort in reverse (b.cmp(a)) to put Highest at the beginning.
        rules.sort_by_key(|b| std::cmp::Reverse(b.metadata().priority));

        rules
    }
}
