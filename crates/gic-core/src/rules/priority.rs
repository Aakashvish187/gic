use std::fmt;

/// Defines the execution priority of a rule.
/// Higher priority rules are executed first by the scheduler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RulePriority {
    /// Rules that run in the background (lowest priority).
    Background = 0,
    /// Low priority rules.
    Low = 1,
    /// Medium priority rules (default).
    Medium = 2,
    /// High priority rules.
    High = 3,
    /// Highest priority rules, executed immediately.
    Highest = 4,
}

impl fmt::Display for RulePriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RulePriority::Background => write!(f, "Background"),
            RulePriority::Low => write!(f, "Low"),
            RulePriority::Medium => write!(f, "Medium"),
            RulePriority::High => write!(f, "High"),
            RulePriority::Highest => write!(f, "Highest"),
        }
    }
}
