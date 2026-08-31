//! Ports `ra.common.tasks.TaskRunner`.
//!
//! The Java version drove a `ThreadPoolExecutor` + `ScheduledThreadPoolExecutor`
//! from a 30-second poll loop. This port keeps the poll loop but gives every
//! task its own `std::thread`: simpler, and adequate for the handful of
//! long-lived tasks services register. A task whose `periodicity_ms` is `-1` is
//! skipped; `0` runs once; `> 0` re-runs on a fixed delay.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::task::{Task, TaskStatus};

/// Runner status. Ports `TaskRunner.Status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerStatus {
    Running,
    Stopping,
    Shutdown,
}

struct Managed {
    task: Option<Box<dyn Task>>,
    status: Arc<Mutex<TaskStatus>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    scheduled: bool,
}

struct Inner {
    tasks: Mutex<Vec<Managed>>,
    cvar: Condvar,
    running: AtomicBool,
    poll_ms: AtomicU64,
}

/// Schedules and runs [`Task`]s on background threads.
pub struct TaskRunner {
    inner: Arc<Inner>,
    runner: Option<JoinHandle<()>>,
}

impl TaskRunner {
    /// A new runner with a 30-second poll period (matching the Java default).
    pub fn new() -> Self {
        TaskRunner {
            inner: Arc::new(Inner {
                tasks: Mutex::new(Vec::new()),
                cvar: Condvar::new(),
                running: AtomicBool::new(false),
                poll_ms: AtomicU64::new(30_000),
            }),
            runner: None,
        }
    }

    /// Override the poll period.
    pub fn set_poll_period_ms(&self, ms: u64) {
        self.inner.poll_ms.store(ms.max(1), Ordering::Relaxed);
        self.inner.cvar.notify_all();
    }

    /// Register a task. If the runner is already started it will be picked up on
    /// the next poll pass; call [`TaskRunner::poke`] to run it immediately.
    pub fn add_task(&self, task: Box<dyn Task>) {
        let mut tasks = self.inner.tasks.lock().unwrap();
        tasks.push(Managed {
            task: Some(task),
            status: Arc::new(Mutex::new(TaskStatus::Ready)),
            stop: Arc::new(AtomicBool::new(false)),
            worker: None,
            scheduled: false,
        });
        self.inner.cvar.notify_all();
    }

    /// Wake the poll loop now instead of waiting for the next period.
    pub fn poke(&self) {
        self.inner.cvar.notify_all();
    }

    /// Start the runner thread. Idempotent.
    pub fn start(&mut self) {
        if self.runner.is_some() {
            return;
        }
        self.inner.running.store(true, Ordering::SeqCst);
        let inner = Arc::clone(&self.inner);
        self.runner = Some(thread::spawn(move || run_loop(inner)));
    }

    /// Current status.
    pub fn status(&self) -> RunnerStatus {
        if self.runner.is_none() {
            RunnerStatus::Shutdown
        } else if self.inner.running.load(Ordering::SeqCst) {
            RunnerStatus::Running
        } else {
            RunnerStatus::Stopping
        }
    }

    /// Number of tasks still tracked.
    pub fn task_count(&self) -> usize {
        self.inner.tasks.lock().unwrap().len()
    }

    /// Stop the poll loop, signal every task to stop, and join all threads.
    pub fn shutdown(&mut self) {
        self.inner.running.store(false, Ordering::SeqCst);
        {
            let tasks = self.inner.tasks.lock().unwrap();
            for m in tasks.iter() {
                m.stop.store(true, Ordering::SeqCst);
            }
        }
        self.inner.cvar.notify_all();
        if let Some(h) = self.runner.take() {
            let _ = h.join();
        }
        let mut tasks = self.inner.tasks.lock().unwrap();
        for m in tasks.iter_mut() {
            if let Some(w) = m.worker.take() {
                let _ = w.join();
            }
        }
        tasks.clear();
    }
}

impl Default for TaskRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TaskRunner {
    fn drop(&mut self) {
        if self.runner.is_some() {
            self.shutdown();
        }
    }
}

fn run_loop(inner: Arc<Inner>) {
    while inner.running.load(Ordering::SeqCst) {
        {
            let mut tasks = inner.tasks.lock().unwrap();

            // Reap finished workers / completed tasks.
            tasks.retain_mut(|m| {
                let done = *m.status.lock().unwrap() == TaskStatus::Completed;
                if done {
                    if let Some(w) = m.worker.take() {
                        let _ = w.join();
                    }
                    false
                } else {
                    true
                }
            });

            // Launch workers for anything not yet scheduled.
            for m in tasks.iter_mut() {
                if m.scheduled {
                    continue;
                }
                let cfg = m.task.as_ref().unwrap().config().clone();
                if cfg.periodicity_ms == -1 {
                    continue; // disabled
                }
                let mut task = m.task.take().unwrap();
                let status = Arc::clone(&m.status);
                let stop = Arc::clone(&m.stop);
                m.scheduled = true;
                m.worker = Some(thread::spawn(move || {
                    if cfg.delayed && cfg.delay_ms > 0 {
                        thread::sleep(Duration::from_millis(cfg.delay_ms));
                    }
                    loop {
                        if stop.load(Ordering::SeqCst) || task.should_stop() {
                            task.on_stop();
                            break;
                        }
                        *status.lock().unwrap() = TaskStatus::Running;
                        task.execute();
                        if cfg.periodicity_ms <= 0 {
                            break;
                        }
                        *status.lock().unwrap() = TaskStatus::Ready;
                        thread::sleep(Duration::from_millis(cfg.periodicity_ms as u64));
                    }
                    *status.lock().unwrap() = TaskStatus::Completed;
                }));
            }
        }

        let poll = inner.poll_ms.load(Ordering::Relaxed);
        let guard = inner.tasks.lock().unwrap();
        let _ = inner
            .cvar
            .wait_timeout(guard, Duration::from_millis(poll))
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::task::TaskConfig;
    use std::sync::atomic::AtomicU32;

    struct Counter {
        cfg: TaskConfig,
        runs: Arc<AtomicU32>,
    }

    impl Task for Counter {
        fn config(&self) -> &TaskConfig {
            &self.cfg
        }
        fn execute(&mut self) -> bool {
            self.runs.fetch_add(1, Ordering::SeqCst);
            true
        }
    }

    #[test]
    fn one_shot_runs_once_and_is_reaped() {
        let runs = Arc::new(AtomicU32::new(0));
        let mut runner = TaskRunner::new();
        runner.set_poll_period_ms(20);
        runner.add_task(Box::new(Counter {
            cfg: TaskConfig::once("c"),
            runs: Arc::clone(&runs),
        }));
        runner.start();

        // give the poll loop a few passes
        for _ in 0..50 {
            if runs.load(Ordering::SeqCst) >= 1 && runner.task_count() == 0 {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(runs.load(Ordering::SeqCst), 1);
        assert_eq!(runner.task_count(), 0);
        runner.shutdown();
        assert_eq!(runner.status(), RunnerStatus::Shutdown);
    }

    #[test]
    fn periodic_runs_multiple_times_then_stops() {
        let runs = Arc::new(AtomicU32::new(0));
        let mut runner = TaskRunner::new();
        runner.set_poll_period_ms(10);
        runner.add_task(Box::new(Counter {
            cfg: TaskConfig::periodic("p", 10),
            runs: Arc::clone(&runs),
        }));
        runner.start();
        thread::sleep(Duration::from_millis(120));
        runner.shutdown();
        let n = runs.load(Ordering::SeqCst);
        assert!(n >= 2, "expected multiple runs, got {n}");
    }
}
