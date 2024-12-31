/// Synthesis of three techniques:
/// - [Stack Overflow answer on implementing a queue with `tokio::mpsc`](https://stackoverflow.com/a/76829279)
/// - [`deadqueue`](https://docs.rs/deadqueue/latest/deadqueue/)
/// - [Tokio shutdown man page](https://tokio.rs/tokio/topics/shutdown#waiting-for-things-to-finish-shutting-down)

use std::sync::{atomic::{AtomicBool, Ordering}, Arc};

use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use crate::{mpd::Mpd, server_util::shutdown_signal};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Job {
    Download {
        url: String
    }
}

type JobQueue = deadqueue::unlimited::Queue<Job>;

#[derive(Debug, Clone)]
pub(super) struct JobManager {
    queue: Arc<JobQueue>,
    mpd: Arc<Mpd>,
    ongoing_job: Arc<AtomicBool>
}

impl JobManager {
    pub(super) fn new(mpd: Arc<Mpd>) -> Self {
        Self {
            queue: Arc::new(JobQueue::new()),
            mpd: mpd,
            ongoing_job: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(super) fn jobs(&self) -> usize {
        self.queue.len()
    }

    pub(super) fn ongoing(&self) -> bool {
        self.ongoing_job.fetch_or(false, Ordering::Relaxed)
    }

    pub(super) fn push(&self, job: Job) {
        self.queue.push(job);
    }

    pub(super) async fn run(&self) {
        let shutdown_token = CancellationToken::new();
        let handler = {
            let queue = self.queue.clone();
            let ongoing = self.ongoing_job.clone();
            let token = shutdown_token.clone();
            let mpd = self.mpd.clone();
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        job = queue.pop() => {
                            ongoing.store(true, Ordering::Relaxed);
                            Self::do_job(mpd.clone(), job).await;
                            ongoing.store(false, Ordering::Relaxed);
                        }
                        // Graceful shutdown.
                        // Task queue is not durable.
                        // We wait only for the current job to finish before
                        // shutting down.
                        _ = token.cancelled() => {
                            return;
                        }
                    }
                }
            })
        };

        // Await shutdown
        shutdown_signal().await;
        // Cancel token
        shutdown_token.cancel();
        // Wait for job to finish
        handler.await.expect("Handler to not err")
    }

    async fn do_job(mpd: Arc<Mpd>, job: Job) {
        match job {
            Job::Download { url } => {
                let result = mpd.download_link(url.as_str()).await;
                match result {
                    Ok(_) => {
                        info!(url = url, "URL successfully downloaded.")
                    },
                    Err(err) => {
                        error!(url = url, "URL ran into error while being downloaded: {}", err);
                    },
                }
            },
        }
    }
}
