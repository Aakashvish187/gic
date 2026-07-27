use gic_core::{EngineMetrics, UIConfig};
use std::time::{Duration, Instant};

/// FPS-independent render controller managing rendering cadence and CPU throttling.
pub struct RenderEngine {
    frame_budget: Duration,
    last_render: Instant,
}

impl RenderEngine {
    pub fn new(ui_config: &UIConfig) -> Self {
        let fps = ui_config.frame_rate_fps.max(1);
        let frame_budget_nanos = 1_000_000_000 / fps;
        Self {
            frame_budget: Duration::from_nanos(frame_budget_nanos),
            last_render: Instant::now(),
        }
    }

    /// Determines if a frame should be rendered according to the target FPS budget.
    pub fn should_render(&self) -> bool {
        self.last_render.elapsed() >= self.frame_budget
    }

    /// Records that a frame was drawn and updates engine telemetry metrics.
    pub fn record_render(&mut self, metrics: &mut EngineMetrics) {
        self.last_render = Instant::now();
        metrics.record_frame();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_engine_frame_budget() {
        let config = UIConfig {
            tick_rate_ms: 250,
            frame_rate_fps: 60,
        };
        let engine = RenderEngine::new(&config);
        // Budget for 60 FPS is ~16.6ms
        assert!(engine.frame_budget.as_millis() >= 16 && engine.frame_budget.as_millis() <= 17);
    }

    #[test]
    fn test_should_render_initial_state() {
        let config = UIConfig {
            tick_rate_ms: 250,
            frame_rate_fps: 1000, // Very high FPS budget (1ms per frame)
        };
        let mut engine = RenderEngine::new(&config);
        std::thread::sleep(Duration::from_millis(2));
        assert!(engine.should_render());

        let mut metrics = EngineMetrics::new();
        engine.record_render(&mut metrics);
        assert_eq!(metrics.frame_count, 1);
    }
}
