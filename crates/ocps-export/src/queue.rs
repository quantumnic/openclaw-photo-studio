//! Export queue with retry and status tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportJob {
    pub id: String,
    pub photo_id: String,
    pub settings: serde_json::Value,
    pub output_path: PathBuf,
    pub status: JobStatus,
    pub retry_count: u32,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Error)]
pub enum QueueError {
    #[error("Job not found: {0}")]
    JobNotFound(String),
    #[error("Queue is empty")]
    QueueEmpty,
    #[error("Job already running")]
    JobAlreadyRunning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatus {
    pub pending: u32,
    pub completed: u32,
    pub failed: u32,
    pub is_running: bool,
}

pub struct ExportQueue {
    jobs: VecDeque<ExportJob>,
    active_job: Option<ExportJob>,
    completed_jobs: Vec<ExportJob>,
    max_retries: u32,
}

impl ExportQueue {
    pub fn new() -> Self {
        Self {
            jobs: VecDeque::new(),
            active_job: None,
            completed_jobs: Vec::new(),
            max_retries: 3,
        }
    }

    pub fn with_max_retries(max_retries: u32) -> Self {
        Self {
            max_retries,
            ..Self::new()
        }
    }

    /// Enqueue a new export job
    pub fn enqueue(&mut self, mut job: ExportJob) -> String {
        job.status = JobStatus::Pending;
        job.created_at = Utc::now();
        let id = job.id.clone();
        self.jobs.push_back(job);
        id
    }

    /// Cancel a pending or running job
    pub fn cancel(&mut self, job_id: &str) -> Result<bool, QueueError> {
        // Check active job
        if let Some(ref mut active) = self.active_job {
            if active.id == job_id {
                active.status = JobStatus::Cancelled;
                active.completed_at = Some(Utc::now());
                let completed = active.clone();
                self.completed_jobs.push(completed);
                self.active_job = None;
                return Ok(true);
            }
        }

        // Check pending jobs
        if let Some(pos) = self.jobs.iter().position(|j| j.id == job_id) {
            let mut job = self.jobs.remove(pos).unwrap();
            job.status = JobStatus::Cancelled;
            job.completed_at = Some(Utc::now());
            self.completed_jobs.push(job);
            return Ok(true);
        }

        Err(QueueError::JobNotFound(job_id.to_string()))
    }

    /// Retry a failed job
    pub fn retry(&mut self, job_id: &str) -> Result<bool, QueueError> {
        if let Some(pos) = self
            .completed_jobs
            .iter()
            .position(|j| j.id == job_id && j.status == JobStatus::Failed)
        {
            let mut job = self.completed_jobs.remove(pos);
            job.status = JobStatus::Pending;
            job.error = None;
            job.retry_count += 1;
            self.jobs.push_back(job);
            return Ok(true);
        }

        Err(QueueError::JobNotFound(job_id.to_string()))
    }

    /// Retry all failed jobs
    pub fn retry_all_failed(&mut self) -> u32 {
        let mut retried = 0;
        let failed: Vec<_> = self
            .completed_jobs
            .iter()
            .enumerate()
            .filter(|(_, j)| j.status == JobStatus::Failed)
            .map(|(i, _)| i)
            .collect();

        for i in failed.iter().rev() {
            let mut job = self.completed_jobs.remove(*i);
            job.status = JobStatus::Pending;
            job.error = None;
            job.retry_count += 1;
            self.jobs.push_back(job);
            retried += 1;
        }

        retried
    }

    /// Get queue status
    pub fn status(&self) -> QueueStatus {
        let pending = self.jobs.len() as u32;
        let completed = self
            .completed_jobs
            .iter()
            .filter(|j| j.status == JobStatus::Completed)
            .count() as u32;
        let failed = self
            .completed_jobs
            .iter()
            .filter(|j| j.status == JobStatus::Failed)
            .count() as u32;
        let is_running = self.active_job.is_some();

        QueueStatus {
            pending,
            completed,
            failed,
            is_running,
        }
    }

    /// Get all completed jobs
    pub fn completed_jobs(&self) -> Vec<&ExportJob> {
        self.completed_jobs
            .iter()
            .filter(|j| j.status == JobStatus::Completed)
            .collect()
    }

    /// Get all failed jobs
    pub fn failed_jobs(&self) -> Vec<&ExportJob> {
        self.completed_jobs
            .iter()
            .filter(|j| j.status == JobStatus::Failed)
            .collect()
    }

    /// Get next pending job
    pub fn next_job(&mut self) -> Option<ExportJob> {
        if self.active_job.is_some() {
            return None;
        }

        if let Some(mut job) = self.jobs.pop_front() {
            job.status = JobStatus::Running;
            self.active_job = Some(job.clone());
            Some(job)
        } else {
            None
        }
    }

    /// Mark active job as completed
    pub fn mark_completed(&mut self, job_id: &str) -> Result<(), QueueError> {
        if let Some(mut job) = self.active_job.take() {
            if job.id == job_id {
                job.status = JobStatus::Completed;
                job.completed_at = Some(Utc::now());
                self.completed_jobs.push(job);
                return Ok(());
            }
            // Put it back if not matching
            self.active_job = Some(job);
        }

        Err(QueueError::JobNotFound(job_id.to_string()))
    }

    /// Mark active job as failed
    pub fn mark_failed(&mut self, job_id: &str, error: String) -> Result<(), QueueError> {
        if let Some(mut job) = self.active_job.take() {
            if job.id == job_id {
                job.status = JobStatus::Failed;
                job.error = Some(error);
                job.completed_at = Some(Utc::now());

                // Auto-retry if under max retries
                if job.retry_count < self.max_retries {
                    job.retry_count += 1;
                    job.status = JobStatus::Pending;
                    job.error = None;
                    job.completed_at = None;
                    self.jobs.push_back(job);
                } else {
                    self.completed_jobs.push(job);
                }

                return Ok(());
            }
            // Put it back if not matching
            self.active_job = Some(job);
        }

        Err(QueueError::JobNotFound(job_id.to_string()))
    }

    /// Get all jobs (for UI display)
    pub fn all_jobs(&self) -> Vec<&ExportJob> {
        let mut all = Vec::new();

        if let Some(ref active) = self.active_job {
            all.push(active);
        }

        for job in &self.jobs {
            all.push(job);
        }

        for job in &self.completed_jobs {
            all.push(job);
        }

        all
    }

    /// Clear completed jobs
    pub fn clear_completed(&mut self) {
        self.completed_jobs
            .retain(|j| j.status != JobStatus::Completed);
    }
}

impl Default for ExportQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_job(id: &str) -> ExportJob {
        ExportJob {
            id: id.to_string(),
            photo_id: format!("photo_{}", id),
            settings: serde_json::json!({}),
            output_path: PathBuf::from("/tmp/test.jpg"),
            status: JobStatus::Pending,
            retry_count: 0,
            error: None,
            created_at: Utc::now(),
            completed_at: None,
        }
    }

    #[test]
    fn test_queue_enqueue_and_status() {
        let mut queue = ExportQueue::new();

        queue.enqueue(create_test_job("job1"));
        queue.enqueue(create_test_job("job2"));
        queue.enqueue(create_test_job("job3"));

        let status = queue.status();
        assert_eq!(status.pending, 3);
        assert_eq!(status.completed, 0);
        assert_eq!(status.failed, 0);
        assert!(!status.is_running);
    }

    #[test]
    fn test_queue_cancel() {
        let mut queue = ExportQueue::new();

        queue.enqueue(create_test_job("job1"));
        queue.enqueue(create_test_job("job2"));

        let result = queue.cancel("job1");
        assert!(result.is_ok());

        let status = queue.status();
        assert_eq!(status.pending, 1);
    }

    #[test]
    fn test_queue_cancel_active() {
        let mut queue = ExportQueue::new();

        queue.enqueue(create_test_job("job1"));
        let _job = queue.next_job();

        let result = queue.cancel("job1");
        assert!(result.is_ok());

        let status = queue.status();
        assert!(!status.is_running);
    }

    #[test]
    fn test_queue_status_counts() {
        let mut queue = ExportQueue::new();

        queue.enqueue(create_test_job("job1"));
        queue.enqueue(create_test_job("job2"));
        queue.enqueue(create_test_job("job3"));

        let job = queue.next_job().unwrap();
        queue.mark_completed(&job.id).unwrap();

        let job = queue.next_job().unwrap();
        queue
            .mark_failed(&job.id, "Test error".to_string())
            .unwrap();

        let status = queue.status();
        assert_eq!(status.pending, 2); // job3 + auto-retry of job2
        assert_eq!(status.completed, 1);
        assert_eq!(status.failed, 0); // Auto-retried, not failed yet
    }

    #[test]
    fn test_next_job() {
        let mut queue = ExportQueue::new();

        queue.enqueue(create_test_job("job1"));
        queue.enqueue(create_test_job("job2"));

        let job = queue.next_job();
        assert!(job.is_some());
        assert_eq!(job.unwrap().id, "job1");

        let status = queue.status();
        assert!(status.is_running);

        // Can't get next job while one is active
        let job = queue.next_job();
        assert!(job.is_none());
    }

    #[test]
    fn test_mark_completed() {
        let mut queue = ExportQueue::new();

        queue.enqueue(create_test_job("job1"));
        let job = queue.next_job().unwrap();

        queue.mark_completed(&job.id).unwrap();

        let status = queue.status();
        assert_eq!(status.completed, 1);
        assert!(!status.is_running);
    }

    #[test]
    fn test_mark_failed_with_retry() {
        let mut queue = ExportQueue::with_max_retries(2);

        queue.enqueue(create_test_job("job1"));
        let job = queue.next_job().unwrap();

        queue
            .mark_failed(&job.id, "First failure".to_string())
            .unwrap();

        let status = queue.status();
        assert_eq!(status.pending, 1); // Auto-retried
        assert_eq!(status.failed, 0);
        assert!(!status.is_running);
    }

    #[test]
    fn test_mark_failed_max_retries() {
        let mut queue = ExportQueue::with_max_retries(1);

        queue.enqueue(create_test_job("job1"));

        // First attempt
        let job = queue.next_job().unwrap();
        queue
            .mark_failed(&job.id, "Failure 1".to_string())
            .unwrap();

        // Auto-retry (retry_count = 1)
        let job = queue.next_job().unwrap();
        assert_eq!(job.retry_count, 1);

        queue
            .mark_failed(&job.id, "Failure 2".to_string())
            .unwrap();

        // Should be failed now (retry_count >= max_retries)
        let status = queue.status();
        assert_eq!(status.pending, 0);
        assert_eq!(status.failed, 1);
    }

    #[test]
    fn test_retry_failed() {
        let mut queue = ExportQueue::with_max_retries(0); // No auto-retry

        queue.enqueue(create_test_job("job1"));
        let job = queue.next_job().unwrap();

        queue
            .mark_failed(&job.id, "Test error".to_string())
            .unwrap();

        let status = queue.status();
        assert_eq!(status.failed, 1);

        let result = queue.retry("job1");
        assert!(result.is_ok());

        let status = queue.status();
        assert_eq!(status.pending, 1);
        assert_eq!(status.failed, 0);
    }

    #[test]
    fn test_retry_all_failed() {
        let mut queue = ExportQueue::with_max_retries(0);

        queue.enqueue(create_test_job("job1"));
        queue.enqueue(create_test_job("job2"));
        queue.enqueue(create_test_job("job3"));

        let job = queue.next_job().unwrap();
        queue
            .mark_failed(&job.id, "Error 1".to_string())
            .unwrap();

        let job = queue.next_job().unwrap();
        queue
            .mark_failed(&job.id, "Error 2".to_string())
            .unwrap();

        let retried = queue.retry_all_failed();
        assert_eq!(retried, 2);

        let status = queue.status();
        assert_eq!(status.pending, 3); // 2 retried + 1 original
    }

    #[test]
    fn test_completed_jobs() {
        let mut queue = ExportQueue::new();

        queue.enqueue(create_test_job("job1"));
        queue.enqueue(create_test_job("job2"));

        let job = queue.next_job().unwrap();
        queue.mark_completed(&job.id).unwrap();

        let completed = queue.completed_jobs();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].id, "job1");
    }

    #[test]
    fn test_failed_jobs() {
        let mut queue = ExportQueue::with_max_retries(0);

        queue.enqueue(create_test_job("job1"));
        let job = queue.next_job().unwrap();

        queue
            .mark_failed(&job.id, "Test error".to_string())
            .unwrap();

        let failed = queue.failed_jobs();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].id, "job1");
        assert_eq!(failed[0].error, Some("Test error".to_string()));
    }

    #[test]
    fn test_clear_completed() {
        let mut queue = ExportQueue::new();

        queue.enqueue(create_test_job("job1"));
        queue.enqueue(create_test_job("job2"));

        let job = queue.next_job().unwrap();
        queue.mark_completed(&job.id).unwrap();

        let job = queue.next_job().unwrap();
        queue.mark_completed(&job.id).unwrap();

        assert_eq!(queue.completed_jobs().len(), 2);

        queue.clear_completed();

        assert_eq!(queue.completed_jobs().len(), 0);
    }
}
