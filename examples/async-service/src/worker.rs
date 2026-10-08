use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

/// How long `stop` waits for the task to finish before giving up.
const STOP_TIMEOUT: Duration = Duration::from_secs(5);

/// A background worker that ticks on an interval until cancelled.
///
/// Shutdown is cooperative: `stop` signals a [`CancellationToken`] and the
/// worker's `select!` reacts immediately instead of polling a flag.
pub struct Worker {
    id: usize,
    tick_interval: Duration,
    cancel: CancellationToken,
    handle: Option<JoinHandle<()>>,
    ticks: Arc<AtomicU64>,
}

impl Worker {
    /// Creates a worker with the default tick interval of one second.
    pub fn new(id: usize) -> Self {
        Self::with_tick_interval(id, Duration::from_secs(1))
    }

    /// Creates a worker with a custom tick interval (useful for tests).
    pub fn with_tick_interval(id: usize, tick_interval: Duration) -> Self {
        Self {
            id,
            tick_interval,
            cancel: CancellationToken::new(),
            handle: None,
            ticks: Arc::new(AtomicU64::new(0)),
        }
    }

    /// The worker's identifier.
    pub fn id(&self) -> usize {
        self.id
    }

    /// How many times the worker has ticked since it was started.
    pub fn ticks(&self) -> u64 {
        self.ticks.load(Ordering::Relaxed)
    }

    /// Whether the worker task has been spawned and has not finished yet.
    pub fn is_running(&self) -> bool {
        self.handle.as_ref().is_some_and(|h| !h.is_finished())
    }

    /// Spawns the worker task. The task runs until [`Worker::stop`] is called.
    pub fn start(&mut self) {
        let cancel = self.cancel.clone();
        let ticks = self.ticks.clone();
        let id = self.id;
        let mut ticker = tokio::time::interval(self.tick_interval);

        let handle = tokio::spawn(async move {
            tracing::info!(worker_id = id, "Worker started");

            loop {
                tokio::select! {
                    // Cancellation is checked first so shutdown is immediate,
                    // even if a tick would fire at the same instant.
                    _ = cancel.cancelled() => break,
                    _ = ticker.tick() => {
                        let tick = ticks.fetch_add(1, Ordering::Relaxed) + 1;
                        tracing::debug!(worker_id = id, tick, "Worker tick");
                    }
                }
            }

            tracing::info!(worker_id = id, "Worker stopped");
        });

        self.handle = Some(handle);
    }

    /// Signals the worker to stop and waits for its task to finish.
    ///
    /// Returns once the task has exited, or after [`STOP_TIMEOUT`] if the
    /// task fails to shut down in time.
    pub async fn stop(&mut self) {
        self.cancel.cancel();
        // Take the handle out of the Option so we own the future and can
        // await it (a `&JoinHandle` is not itself a future).
        if let Some(handle) = self.handle.take() {
            let _ = tokio::time::timeout(STOP_TIMEOUT, handle).await;
        }
    }
}
