use std::time::Instant;

/// Performance & engine runtime telemetry metrics.
#[derive(Debug, Clone)]
pub struct EngineMetrics {
    pub frame_count: u64,
    pub tick_count: u64,
    pub current_fps: f64,
    pub last_frame_time: Instant,
    pub screen_width: u16,
    pub screen_height: u16,
}

impl Default for EngineMetrics {
    fn default() -> Self {
        Self {
            frame_count: 0,
            tick_count: 0,
            current_fps: 0.0,
            last_frame_time: Instant::now(),
            screen_width: 80,
            screen_height: 24,
        }
    }
}

impl EngineMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Increments frame count and updates calculated FPS based on elapsed frame duration.
    pub fn record_frame(&mut self) {
        self.frame_count += 1;
        let elapsed = self.last_frame_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            let instant_fps = 1.0 / elapsed;
            // Exponential moving average for smooth FPS display
            self.current_fps = if self.current_fps == 0.0 {
                instant_fps
            } else {
                0.9 * self.current_fps + 0.1 * instant_fps
            };
        }
        self.last_frame_time = Instant::now();
    }

    /// Increments state tick counter.
    pub fn record_tick(&mut self) {
        self.tick_count += 1;
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
        assert_eq!(metrics.frame_count, 0);
        assert_eq!(metrics.tick_count, 0);
        assert_eq!(metrics.screen_width, 80);
        assert_eq!(metrics.screen_height, 24);
    }

    #[test]
    fn test_record_frame_and_tick() {
        let mut metrics = EngineMetrics::new();
        metrics.record_tick();
        metrics.record_tick();
        assert_eq!(metrics.tick_count, 2);

        metrics.record_frame();
        assert_eq!(metrics.frame_count, 1);
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
