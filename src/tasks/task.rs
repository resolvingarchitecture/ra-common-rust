//! Ports `ra.common.tasks.{Task, BaseTask}`.

/// Lifecycle state of a task. Ports `Task.Status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Ready,
    Running,
    Completed,
}

/// Scheduling configuration for a [`Task`]. Ports the `BaseTask` flag fields.
#[derive(Debug, Clone)]
pub struct TaskConfig {
    /// Human-readable name.
    pub name: String,
    /// `0` = run once; `> 0` = re-run every N milliseconds; `-1` = disabled.
    pub periodicity_ms: i64,
    /// Whether to wait [`TaskConfig::delay_ms`] before the first run.
    pub delayed: bool,
    /// Initial delay in milliseconds when [`TaskConfig::delayed`].
    pub delay_ms: u64,
    /// `true` = wait `periodicity_ms` *after* each run finishes (fixed delay);
    /// `false` = aim to start every `periodicity_ms` (fixed rate). This port
    /// currently always behaves as fixed-delay; the flag is preserved for
    /// callers.
    pub fixed_delay: bool,
    /// Hint that the task may run for a long time (informational in this port —
    /// every task already gets its own thread).
    pub long_running: bool,
}

impl TaskConfig {
    /// A one-shot task named `name`.
    pub fn once(name: impl Into<String>) -> Self {
        TaskConfig {
            name: name.into(),
            periodicity_ms: 0,
            delayed: false,
            delay_ms: 0,
            fixed_delay: false,
            long_running: false,
        }
    }

    /// A task that repeats every `period_ms` milliseconds.
    pub fn periodic(name: impl Into<String>, period_ms: u64) -> Self {
        TaskConfig {
            periodicity_ms: period_ms as i64,
            ..TaskConfig::once(name)
        }
    }

    /// Set the initial delay.
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delayed = true;
        self.delay_ms = delay_ms;
        self
    }
}

/// A unit of work run by a [`TaskRunner`](super::TaskRunner).
pub trait Task: Send {
    /// This task's scheduling configuration.
    fn config(&self) -> &TaskConfig;

    /// Do the work. Return `true` on success. For periodic tasks this is called
    /// repeatedly; it should check [`Task::should_stop`] for long loops.
    fn execute(&mut self) -> bool;

    /// Cooperative stop signal — override to observe it inside [`Task::execute`].
    /// The [`TaskRunner`](super::TaskRunner) also stops calling `execute` after a
    /// stop is requested.
    fn should_stop(&self) -> bool {
        false
    }

    /// Called by the runner when a stop is requested. Default: nothing.
    fn on_stop(&mut self) {}
}
