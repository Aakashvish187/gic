use super::{matcher::StringMatcher, rule::RuleContext};

/// A trait for reusable, composable conditions during rule evaluation.
pub trait RuleCondition: Send + Sync {
    /// Evaluates the condition against the current context.
    fn evaluate(&self, ctx: &dyn RuleContext) -> bool;
}

/// A composite condition that applies logical AND to its children.
pub struct AndCondition {
    pub conditions: Vec<Box<dyn RuleCondition>>,
}

impl RuleCondition for AndCondition {
    fn evaluate(&self, ctx: &dyn RuleContext) -> bool {
        self.conditions.iter().all(|c| c.evaluate(ctx))
    }
}

/// A composite condition that applies logical OR to its children.
pub struct OrCondition {
    pub conditions: Vec<Box<dyn RuleCondition>>,
}

impl RuleCondition for OrCondition {
    fn evaluate(&self, ctx: &dyn RuleContext) -> bool {
        self.conditions.iter().any(|c| c.evaluate(ctx))
    }
}

/// A condition that negates its inner condition.
pub struct NotCondition {
    pub condition: Box<dyn RuleCondition>,
}

impl RuleCondition for NotCondition {
    fn evaluate(&self, ctx: &dyn RuleContext) -> bool {
        !self.condition.evaluate(ctx)
    }
}

/// Condition checking if a string matches using `StringMatcher`.
pub struct MatchStringCondition {
    pub value: String,
    pub matcher: StringMatcher,
}

impl RuleCondition for MatchStringCondition {
    fn evaluate(&self, _ctx: &dyn RuleContext) -> bool {
        self.matcher.matches(&self.value)
    }
}
