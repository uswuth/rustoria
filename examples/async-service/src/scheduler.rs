use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

/// How long `stop` waits for the task to finish before giving up.
const STOP_TIMEOUT: Duration = Duration::from_secs(5);

/// A task scheduler that runs a job on a fixed interval until cancelled.
pub struct Scheduler {
    interval: Duration,
    cancel: CancellationToken,
    handle: Option<JoinHandle<()>>,
    runs: Arc<AtomicU64>,
}

impl Scheduler {
    /// Creates a scheduler that runs its job every five seconds.
    pub fn new() -> Self {
        Self::with_interval(Duration::from_secs(5))
    }

    /// Creates a scheduler with a custom interval (useful for tests).
    pub fn with_interval(interval: Duration) -> Self {
        Self {
            interval,
            cancel: CancellationToken::new(),
            handle: None,
            runs: Arc::new(AtomicU64::new(0)),
        }
    }

    /// How many times the scheduled job has run since `start`.
    pub fn runs(&self) -> u64 {
        self.runs.load(Ordering::Relaxed)
    }

    /// Whether the scheduler task has been spawned and has not finished yet.
    pub fn is_running(&self) -> bool {
        self.handle.as_ref().is_some_and(|h| !h.is_finished())
    }

    /// Spawns the scheduler task. It runs until [`Scheduler::stop`] is called.
    pub fn start(&mut self) {
        let cancel = self.cancel.clone();
        let runs = self.runs.clone();
        let mut ticker = tokio::time::interval(self.interval);

        let handle = tokio::spawn(async move {
            tracing::info!("Scheduler started");

            loop {
                tokio::select! {
                    _ = cancel.cancelled() => break,
                    _ = ticker.tick() => {
                        let run = runs.fetch_add(1, Ordering::Relaxed) + 1;
                        tracing::debug!(run, "Scheduled job executed");
                    }
                }
            }

            tracing::info!("Scheduler stopped");
        });

        self.handle = Some(handle);
    }

    /// Signals the scheduler to stop and waits for its task to finish.
    pub async fn stop(&mut self) {
        self.cancel.cancel();
        // Take the handle out of the Option so we own the future and can
        // await it (a `&JoinHandle` is not itself a future).
        if let Some(handle) = self.handle.take() {
            let _ = tokio::time::timeout(STOP_TIMEOUT, handle).await;
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}
