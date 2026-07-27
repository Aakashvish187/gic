/// Provides documentation placeholders or actual documentation for a rule.
#[derive(Debug, Clone, Default)]
pub struct RuleDocumentation {
    /// A short, single-line explanation of the rule.
    pub summary: String,
    /// Detailed Markdown documentation for the rule.
    pub details: Option<String>,
    /// A URL linking to official documentation.
    pub url: Option<String>,
    /// Code examples showing what is considered "bad" and "good".
    pub examples: Vec<DocumentationExample>,
}

#[derive(Debug, Clone)]
pub struct DocumentationExample {
    /// The language of the example (e.g., "yaml", "rust").
    pub language: String,
    /// The code snippet illustrating the bad practice.
    pub bad: String,
    /// The code snippet illustrating the correct practice.
    pub good: String,
    /// Optional explanation of why it's bad and how the fix works.
    pub explanation: Option<String>,
}

impl RuleDocumentation {
    pub fn new<S: Into<String>>(summary: S) -> Self {
        Self {
            summary: summary.into(),
            details: None,
            url: None,
            examples: Vec::new(),
        }
    }

    pub fn with_details<S: Into<String>>(mut self, details: S) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn with_url<S: Into<String>>(mut self, url: S) -> Self {
        self.url = Some(url.into());
        self
    }
}
