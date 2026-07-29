use gic_core::{DirtyRegions, EngineMetrics, UIConfig};
use std::time::Instant;

/// Event-driven render controller.
pub struct RenderEngine {
    last_render: Instant,
}

impl RenderEngine {
    pub fn new(_ui_config: &UIConfig) -> Self {
        Self {
            last_render: Instant::now(),
        }
    }

    /// Determines if a frame should be rendered based on dirty regions.
    pub fn should_render(&self, dirty: &DirtyRegions) -> bool {
        dirty.full_redraw || dirty.status_bar || !dirty.lines.is_empty()
    }

    /// Records that a frame was drawn and updates engine telemetry metrics.
    pub fn record_render(&mut self, metrics: &mut EngineMetrics) {
        let elapsed = self.last_render.elapsed().as_secs_f64() * 1000.0;
        self.last_render = Instant::now();
        metrics.record_render(elapsed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_render_dirty() {
        let config = UIConfig::default();
        let engine = RenderEngine::new(&config);

        let mut dirty = DirtyRegions::default();
        assert!(engine.should_render(&dirty));

        dirty.clear();
        assert!(!engine.should_render(&dirty));

        dirty.mark_line(5);
        assert!(engine.should_render(&dirty));
    }
}
