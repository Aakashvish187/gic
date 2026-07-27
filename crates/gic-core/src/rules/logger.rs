/// Simple logger utility for the rule engine.
/// This provides an abstraction point for structured logging within the Universal Rule Engine.
/// In production, these would typically delegate to `tracing` or `log` crate macros.
pub struct RuleLogger;

impl RuleLogger {
    /// Logs debug information.
    #[allow(unused_variables)]
    pub fn debug(msg: &str) {
        #[cfg(test)]
        println!("[DEBUG] {}", msg);
    }

    /// Logs trace information (high volume).
    #[allow(unused_variables)]
    pub fn trace(msg: &str) {
        #[cfg(test)]
        println!("[TRACE] {}", msg);
    }

    /// Logs error information.
    #[allow(unused_variables)]
    pub fn error(msg: &str) {
        #[cfg(test)]
        eprintln!("[ERROR] {}", msg);
    }
}
