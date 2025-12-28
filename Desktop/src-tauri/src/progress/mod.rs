// Progress reporting utilities
// Provides progress events for long-running operations
// Per Constitution SC-009: Provide progress feedback for operations > 3 seconds

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Progress update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub operation_id: String,
    pub operation_type: String,
    pub current_step: usize,
    pub total_steps: usize,
    pub step_name: String,
    pub percent_complete: f64,
    pub message: String,
}

/// Progress state for tracking an operation
#[derive(Debug, Clone)]
pub struct ProgressState {
    pub operation_id: String,
    pub operation_type: String,
    pub total_steps: usize,
    pub current_step: usize,
    pub step_name: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
}

impl ProgressState {
    pub fn new(operation_id: String, operation_type: String, total_steps: usize) -> Self {
        Self {
            operation_id,
            operation_type,
            total_steps,
            current_step: 0,
            step_name: "Initializing".to_string(),
            start_time: chrono::Utc::now(),
        }
    }

    pub fn percent_complete(&self) -> f64 {
        if self.total_steps == 0 {
            return 100.0;
        }
        (self.current_step as f64 / self.total_steps as f64) * 100.0
    }

    pub fn elapsed_ms(&self) -> i64 {
        let now = chrono::Utc::now();
        now.signed_duration_since(self.start_time)
            .num_milliseconds()
    }
}

/// Progress reporter for tracking and emitting progress updates
pub struct ProgressReporter {
    state: Arc<Mutex<ProgressState>>,
    emit_updates: bool,
}

impl ProgressReporter {
    /// Create a new progress reporter
    pub fn new(
        operation_id: String,
        operation_type: String,
        total_steps: usize,
        emit_updates: bool,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(ProgressState::new(
                operation_id,
                operation_type,
                total_steps,
            ))),
            emit_updates,
        }
    }

    /// Update progress to the next step
    pub fn next_step(&self, step_name: &str) {
        let mut state = self.state.lock().unwrap();
        state.current_step = state.current_step.saturating_add(1);
        state.step_name = step_name.to_string();
    }

    /// Update progress with custom step number
    pub fn update_step(&self, step: usize, step_name: &str) {
        let mut state = self.state.lock().unwrap();
        state.current_step = step.min(state.total_steps);
        state.step_name = step_name.to_string();
    }

    /// Get a message with current progress
    pub fn message(&self) -> String {
        let state = self.state.lock().unwrap();
        format!(
            "{}: {}/{}",
            state.step_name, state.current_step, state.total_steps
        )
    }

    /// Mark the operation as complete
    pub fn complete(&self, _message: &str) {
        let mut state = self.state.lock().unwrap();
        state.current_step = state.total_steps;
        state.step_name = "Complete".to_string();
    }

    /// Mark the operation as failed
    pub fn error(&self, _error_message: &str) {
        let mut state = self.state.lock().unwrap();
        state.step_name = "Error".to_string();
        state.current_step = state.current_step.saturating_sub(1);
    }

    /// Get the current progress state (cloned)
    pub fn state(&self) -> ProgressState {
        self.state.lock().unwrap().clone()
    }

    /// Get the operation ID
    pub fn operation_id(&self) -> String {
        self.state.lock().unwrap().operation_id.clone()
    }
}

/// Helper to create a progress reporter for common operations
pub fn create_import_reporter(operation_id: String) -> ProgressReporter {
    ProgressReporter::new(
        operation_id,
        "import_wdr".to_string(),
        5, // Parse, validate, insert SQL, insert metadata, complete
        true,
    )
}

pub fn create_comparison_reporter(operation_id: String) -> ProgressReporter {
    ProgressReporter::new(
        operation_id,
        "create_comparison".to_string(),
        4, // Load reports, match SQLs, calculate metrics, save
        true,
    )
}

pub fn create_audit_reporter(operation_id: String) -> ProgressReporter {
    ProgressReporter::new(
        operation_id,
        "run_audit".to_string(),
        3, // Load SQLs, run detection rules, save issues
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_state_calculation() {
        let state = ProgressState::new("test-1".to_string(), "test".to_string(), 10);
        assert_eq!(state.current_step, 0);
        assert_eq!(state.percent_complete(), 0.0);

        let mut state = state;
        state.current_step = 5;
        assert_eq!(state.percent_complete(), 50.0);

        state.current_step = 10;
        assert_eq!(state.percent_complete(), 100.0);
    }

    #[test]
    fn test_progress_reporter() {
        let reporter = ProgressReporter::new(
            "test-2".to_string(),
            "test".to_string(),
            5,
            false, // Don't emit in tests
        );

        assert_eq!(reporter.operation_id(), "test-2");
        assert_eq!(reporter.state().current_step, 0);

        reporter.next_step("Step 1");
        assert_eq!(reporter.state().current_step, 1);
        assert_eq!(reporter.state().percent_complete(), 20.0);

        reporter.update_step(3, "Step 3");
        assert_eq!(reporter.state().current_step, 3);
        assert_eq!(reporter.state().percent_complete(), 60.0);
    }
}
