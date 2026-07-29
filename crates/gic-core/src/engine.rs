use std::time::Instant;

/// Performance & engine runtime telemetry metrics.
#[derive(Debug, Clone)]
pub struct EngineMetrics {
    pub draw_calls: u64,
    pub last_render_time_ms: f64,
    pub screen_width: u16,
    pub screen_height: u16,
}

impl Default for EngineMetrics {
    fn default() -> Self {
        Self {
            draw_calls: 0,
            last_render_time_ms: 0.0,
            screen_width: 80,
            screen_height: 24,
        }
    }
}

impl EngineMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a render pass and its duration.
    pub fn record_render(&mut self, duration_ms: f64) {
        self.draw_calls += 1;
        self.last_render_time_ms = duration_ms;
    }

    /// Updates terminal window dimensions.
    pub fn update_dimensions(&mut self, width: u16, height: u16) {
        self.screen_width = width;
        self.screen_height = height;
    }
}

/// Operational state of the GIC Terminal Engine.
#[derive(Debug, Clone)]
pub struct EngineState {
    pub active_mode: String,
    pub status_message: String,
    pub mouse_enabled: bool,
    pub metrics: EngineMetrics,
}

impl Default for EngineState {
    fn default() -> Self {
        Self {
            active_mode: "NORMAL".to_string(),
            status_message: "Terminal Engine Ready".to_string(),
            mouse_enabled: true,
            metrics: EngineMetrics::default(),
        }
    }
}

impl EngineState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_status<S: Into<String>>(&mut self, msg: S) {
        self.status_message = msg.into();
    }

    pub fn toggle_mouse(&mut self) -> bool {
        self.mouse_enabled = !self.mouse_enabled;
        self.mouse_enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_metrics_defaults() {
        let metrics = EngineMetrics::default();
        assert_eq!(metrics.draw_calls, 0);
        assert_eq!(metrics.last_render_time_ms, 0.0);
        assert_eq!(metrics.screen_width, 80);
        assert_eq!(metrics.screen_height, 24);
    }

    #[test]
    fn test_record_render() {
        let mut metrics = EngineMetrics::new();
        metrics.record_render(1.5);
        assert_eq!(metrics.draw_calls, 1);
        assert_eq!(metrics.last_render_time_ms, 1.5);
    }

    #[test]
    fn test_engine_state_toggle_mouse() {
        let mut state = EngineState::new();
        assert!(state.mouse_enabled);
        let new_state = state.toggle_mouse();
        assert!(!new_state);
        assert!(!state.mouse_enabled);
    }
}
