use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use anyhow::Result;
use crossbeam::channel;

use crate::plugin::Plugin;
use crate::utils::{self, PluginInstance, ProcessingResult, YieldValue};

/// A task in the work queue with target and kwargs
#[derive(Debug, Clone)]
struct WorkItem {
    target: String,
    kwargs: HashMap<String, String>,
}

/// The Orchestrator manages parallel task execution with deduplication
pub struct Orchestrator {
    plugin: Arc<Plugin>,
    instance: Arc<PluginInstance>,
    num_workers: usize,
    max_tasks: usize,
    initial_kwargs: HashMap<String, String>,
    visited: Arc<Mutex<HashSet<String>>>,
    results: Arc<Mutex<Vec<ProcessingResult>>>,
    tasks_processed: Arc<AtomicUsize>,
    shutdown: Arc<AtomicBool>,
}

impl Orchestrator {
    /// Create a new Orchestrator with the specified plugin and worker count
    pub fn new(plugin: Plugin, instance: PluginInstance, num_workers: usize) -> Self {
        Self {
            plugin: Arc::new(plugin),
            instance: Arc::new(instance),
            num_workers,
            max_tasks: 0, // 0 means unlimited
            initial_kwargs: HashMap::new(),
            visited: Arc::new(Mutex::new(HashSet::new())),
            results: Arc::new(Mutex::new(Vec::new())),
            tasks_processed: Arc::new(AtomicUsize::new(0)),
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Set maximum tasks to process (0 = unlimited)
    pub fn set_max_tasks(&mut self, max_tasks: usize) {
        self.max_tasks = max_tasks;
    }

    /// Set initial kwargs to pass to all run() calls
    pub fn set_kwargs(&mut self, kwargs: HashMap<String, String>) {
        self.initial_kwargs = kwargs;
    }

    /// Check if a target has been visited, and mark it if not
    /// Returns true if this is a new target, false if already visited
    fn mark_visited(&self, target: &str) -> bool {
        let mut visited = self.visited.lock().unwrap();
        if visited.contains(target) {
            false
        } else {
            visited.insert(target.to_string());
            true
        }
    }

    /// Run the orchestrator with initial targets
    pub fn run(&self, initial_targets: Vec<String>) -> Result<Vec<ProcessingResult>> {
        if self.num_workers == 0 {
            return Err(anyhow::anyhow!("No workers specified"));
        }

        // Create an unbounded channel for the work queue
        let (task_tx, task_rx) = channel::unbounded::<WorkItem>();

        // Seed initial targets (with dedup) and initial kwargs
        for target in initial_targets {
            if self.mark_visited(&target) {
                task_tx.send(WorkItem {
                    target,
                    kwargs: self.initial_kwargs.clone(),
                })?;
            }
        }

        let mut handles = vec![];
        let active_workers = Arc::new(AtomicUsize::new(self.num_workers));
        let initial_kwargs = Arc::new(self.initial_kwargs.clone());

        // Spawn worker threads
        for worker_id in 0..self.num_workers {
            let plugin = Arc::clone(&self.plugin);
            let instance = Arc::clone(&self.instance);
            let task_rx = task_rx.clone();
            let task_tx = task_tx.clone();
            let results = Arc::clone(&self.results);
            let visited = Arc::clone(&self.visited);
            let tasks_processed = Arc::clone(&self.tasks_processed);
            let shutdown = Arc::clone(&self.shutdown);
            let max_tasks = self.max_tasks;
            let active_workers = Arc::clone(&active_workers);
            let initial_kwargs = Arc::clone(&initial_kwargs);

            let handle = thread::spawn(move || {
                utils::debug(&format!("Worker {} started", worker_id));

                loop {
                    // Check for shutdown
                    if shutdown.load(Ordering::Relaxed) {
                        break;
                    }

                    // Try to receive a task with timeout
                    match task_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                        Ok(work_item) => {
                            // Check max tasks limit
                            if max_tasks > 0 {
                                let current = tasks_processed.fetch_add(1, Ordering::Relaxed);
                                if current >= max_tasks {
                                    shutdown.store(true, Ordering::Relaxed);
                                    break;
                                }
                            } else {
                                tasks_processed.fetch_add(1, Ordering::Relaxed);
                            }

                            utils::debug(&format!("Worker {} processing: {}", worker_id, work_item.target));

                            // Run the plugin for this target with kwargs
                            match plugin.run_target(&instance, &work_item.target, &work_item.kwargs) {
                                Ok(yields) => {
                                    for yield_value in yields {
                                        match yield_value {
                                            YieldValue::Result(result) => {
                                                let mut results = results.lock().unwrap();
                                                results.push(result);
                                            }
                                            YieldValue::Task(task) => {
                                                // Check dedup and queue new task
                                                let mut visited_guard = visited.lock().unwrap();
                                                if !visited_guard.contains(&task.target) {
                                                    visited_guard.insert(task.target.clone());
                                                    drop(visited_guard);

                                                    if !shutdown.load(Ordering::Relaxed) {
                                                        // Merge initial kwargs with task-specific kwargs
                                                        // Task kwargs take precedence
                                                        let mut merged_kwargs = initial_kwargs.as_ref().clone();
                                                        merged_kwargs.extend(task.kwargs);

                                                        let _ = task_tx.send(WorkItem {
                                                            target: task.target,
                                                            kwargs: merged_kwargs,
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    utils::debug(&format!(
                                        "Worker {} error processing {}: {}",
                                        worker_id, work_item.target, e
                                    ));
                                }
                            }
                        }
                        Err(channel::RecvTimeoutError::Timeout) => {
                            // No task available, check if we should exit
                            // If channel is empty and no other workers are active, we're done
                            if task_rx.is_empty() {
                                // Decrement active workers and check if we're the last one
                                let remaining = active_workers.fetch_sub(1, Ordering::SeqCst);
                                if remaining == 1 {
                                    // We were the last active worker, signal shutdown
                                    shutdown.store(true, Ordering::Relaxed);
                                    break;
                                }
                                // Wait a bit and re-check
                                thread::sleep(std::time::Duration::from_millis(50));
                                // Re-increment if there's new work
                                if !task_rx.is_empty() {
                                    active_workers.fetch_add(1, Ordering::SeqCst);
                                } else if shutdown.load(Ordering::Relaxed) {
                                    break;
                                } else {
                                    active_workers.fetch_add(1, Ordering::SeqCst);
                                }
                            }
                        }
                        Err(channel::RecvTimeoutError::Disconnected) => {
                            break;
                        }
                    }
                }

                utils::debug(&format!("Worker {} finished", worker_id));
            });

            handles.push(handle);
        }

        // Drop our sender so workers can detect when work is done
        drop(task_tx);

        // Wait for all workers to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Return collected results
        let results = self.results.lock().unwrap();
        Ok(results.clone())
    }

    /// Get the number of tasks processed
    pub fn tasks_processed(&self) -> usize {
        self.tasks_processed.load(Ordering::Relaxed)
    }

    /// Get the number of unique targets visited
    pub fn targets_visited(&self) -> usize {
        self.visited.lock().unwrap().len()
    }
}
