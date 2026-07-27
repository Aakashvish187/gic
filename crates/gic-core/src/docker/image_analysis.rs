//! Image Size Estimation and Layer Breakdown Contracts.

/// Estimated image metrics.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImageMetricsReport {
    /// Total estimated layer count.
    pub total_layers: usize,
    /// Estimated size classification string.
    pub estimated_size_class: String,
}

/// Image size analyzer contract.
#[derive(Debug, Clone, Default)]
pub struct ImageAnalyzer;

impl ImageAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn estimate_metrics(&self, instruction_count: usize) -> ImageMetricsReport {
        let size_class = if instruction_count > 15 {
            "Large (>500MB)".to_string()
        } else if instruction_count > 5 {
            "Medium (100MB-500MB)".to_string()
        } else {
            "Small (<100MB)".to_string()
        };

        ImageMetricsReport {
            total_layers: instruction_count,
            estimated_size_class: size_class,
        }
    }
}
