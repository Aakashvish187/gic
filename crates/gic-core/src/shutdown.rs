/// Domain reasons for requesting application shutdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownReason {
    UserRequested,
    SignalInterrupt,
    FatalError,
}

impl std::fmt::Display for ShutdownReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserRequested => write!(f, "User requested exit"),
            Self::SignalInterrupt => write!(f, "Interrupt signal received (SIGINT/SIGTERM)"),
            Self::FatalError => write!(f, "Fatal runtime error"),
        }
    }
}

/// Domain trait for components capable of receiving shutdown notifications.
pub trait ShutdownSignal {
    fn should_shutdown(&self) -> bool;
    fn request_shutdown(&self, reason: ShutdownReason);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct TestShutdownHandler {
        flag: AtomicBool,
    }

    impl ShutdownSignal for TestShutdownHandler {
        fn should_shutdown(&self) -> bool {
            self.flag.load(Ordering::Relaxed)
        }

        fn request_shutdown(&self, _reason: ShutdownReason) {
            self.flag.store(true, Ordering::Relaxed);
        }
    }

    #[test]
    fn test_shutdown_reason_display() {
        assert_eq!(
            format!("{}", ShutdownReason::UserRequested),
            "User requested exit"
        );
        assert_eq!(
            format!("{}", ShutdownReason::SignalInterrupt),
            "Interrupt signal received (SIGINT/SIGTERM)"
        );
        assert_eq!(
            format!("{}", ShutdownReason::FatalError),
            "Fatal runtime error"
        );
    }

    #[test]
    fn test_shutdown_signal_trait() {
        let handler = TestShutdownHandler {
            flag: AtomicBool::new(false),
        };
        assert!(!handler.should_shutdown());
        handler.request_shutdown(ShutdownReason::UserRequested);
        assert!(handler.should_shutdown());
    }
}
