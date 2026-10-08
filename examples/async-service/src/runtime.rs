use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cancellation;
use crate::scheduler::Scheduler;
use crate::worker::Worker;

/// Number of workers the runtime starts.
const WORKER_COUNT: usize = 3;

/// Application runtime that manages workers and the scheduler.
pub struct Runtime {
    workers: Arc<RwLock<Vec<Worker>>>,
    scheduler: Arc<RwLock<Scheduler>>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            workers: Arc::new(RwLock::new(Vec::new())),
            scheduler: Arc::new(RwLock::new(Scheduler::new())),
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        // Start workers. `start` does not await, so holding the write guard
        // for this short critical section is fine.
        {
            let mut workers = self.workers.write().await;
            for i in 0..WORKER_COUNT {
                let mut worker = Worker::new(i);
                worker.start();
                workers.push(worker);
            }
        }

        {
            let mut scheduler = self.scheduler.write().await;
            scheduler.start();
        }

        // Wait for a shutdown signal (Ctrl+C or SIGTERM).
        cancellation::wait_for_shutdown().await;

        tracing::info!("Shutdown signal received, stopping workers...");

        // Take the workers out of the lock so the write guard is dropped
        // before we `.await` each shutdown. Holding a guard across an
        // `.await` blocks every other reader/writer for the whole wait.
        let workers = std::mem::take(&mut *self.workers.write().await);
        for mut worker in workers {
            worker.stop().await;
        }

        // Same pattern for the scheduler: swap in a fresh (stopped) one,
        // drop the guard, then await the real shutdown.
        let mut scheduler = std::mem::take(&mut *self.scheduler.write().await);
        scheduler.stop().await;

        Ok(())
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
