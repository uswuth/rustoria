use std::time::{Duration, Instant};

use async_service::scheduler::Scheduler;
use async_service::worker::Worker;

#[tokio::test]
async fn worker_ticks_until_stopped() {
    let mut worker = Worker::with_tick_interval(1, Duration::from_millis(20));
    worker.start();
    assert!(worker.is_running(), "worker task should be running");

    // 20ms interval over 150ms => several ticks (first tick is immediate).
    tokio::time::sleep(Duration::from_millis(150)).await;
    let ticks = worker.ticks();
    assert!(ticks >= 2, "worker should have ticked, got {ticks}");

    worker.stop().await;
    assert!(!worker.is_running(), "worker task should have finished");

    // No ticks may be recorded after shutdown.
    let ticks_at_stop = worker.ticks();
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert_eq!(
        worker.ticks(),
        ticks_at_stop,
        "worker kept ticking after stop"
    );
}

#[tokio::test]
async fn worker_stop_is_immediate_not_next_tick() {
    // With a 60s tick interval, a polling-based shutdown would hang until the
    // next tick. CancellationToken wakes the task immediately.
    let mut worker = Worker::with_tick_interval(7, Duration::from_secs(60));
    worker.start();
    tokio::time::sleep(Duration::from_millis(20)).await;

    let started = Instant::now();
    worker.stop().await;
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "stop should not wait for the next tick, took {:?}",
        started.elapsed()
    );
}

#[tokio::test]
async fn scheduler_runs_job_until_stopped() {
    let mut scheduler = Scheduler::with_interval(Duration::from_millis(20));
    scheduler.start();
    assert!(scheduler.is_running(), "scheduler task should be running");

    tokio::time::sleep(Duration::from_millis(150)).await;
    let runs = scheduler.runs();
    assert!(runs >= 2, "scheduled job should have run, got {runs}");

    scheduler.stop().await;
    assert!(
        !scheduler.is_running(),
        "scheduler task should have finished"
    );

    let runs_at_stop = scheduler.runs();
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert_eq!(
        scheduler.runs(),
        runs_at_stop,
        "scheduler kept running after stop"
    );
}

#[tokio::test]
async fn multiple_workers_run_concurrently() {
    let mut workers: Vec<Worker> = (0..5)
        .map(|i| Worker::with_tick_interval(i, Duration::from_millis(20)))
        .collect();

    for worker in &mut workers {
        worker.start();
    }

    tokio::time::sleep(Duration::from_millis(150)).await;

    for worker in &workers {
        assert!(
            worker.ticks() >= 2,
            "worker {} should have ticked, got {}",
            worker.id(),
            worker.ticks()
        );
    }

    // The whole shutdown must finish well within the 5s stop timeout.
    let started = Instant::now();
    for worker in &mut workers {
        worker.stop().await;
    }
    assert!(started.elapsed() < Duration::from_secs(5));

    for worker in &workers {
        assert!(!worker.is_running());
    }
}

#[tokio::test]
async fn stop_without_start_is_a_no_op() {
    let mut worker = Worker::new(0);
    worker.stop().await; // must not hang or panic
    assert!(!worker.is_running());
}
