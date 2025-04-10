use std::sync::{Arc, Mutex};
use std::thread;
use anyhow::Result;
use crossbeam::channel;

use crate::plugin::Plugin;
use crate::utils;

/// The Orchestrator manages multiple worker threads for processing data
pub struct Orchestrator {
    plugin: Arc<Plugin>,
    num_workers: usize,
    data_queue: Arc<Mutex<Vec<utils::ExecutionContext>>>,
    results: Arc<Mutex<Vec<utils::ExecutionContext>>>,
}

impl Orchestrator {
    /// Create a new Orchestrator with the specified plugin and number of workers
    pub fn new(plugin: Plugin, num_workers: usize) -> Self {
        Self {
            plugin: Arc::new(plugin),
            num_workers,
            data_queue: Arc::new(Mutex::new(Vec::new())),
            results: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Initialize the plugin and set up the data queue
    pub fn init(&mut self, args: &[String]) -> Result<()> {
        let initial_data = self.plugin.init(args)?;
        utils::debug(&format!("Plugin initialized with {} items", initial_data.len()));
        
        self.set_data_queue(initial_data);
        
        Ok(())
    }

    /// Set the data queue
    pub fn set_data_queue(&mut self, new_data: Vec<utils::ExecutionContext>) {
        let mut data_queue = self.data_queue.lock().unwrap();
        *data_queue = new_data;
    }

    /// Start the worker threads and process all data
    pub fn run(&self) -> Result<Vec<utils::ExecutionContext>> {
        if self.num_workers == 0 {
            return Err(anyhow::anyhow!("No workers specified"));
        }

        let (tx, rx) = channel::bounded::<utils::ExecutionContext>(self.num_workers);
        let mut handles = vec![];

        // Create worker threads
        for worker_id in 0..self.num_workers {
            let plugin = Arc::clone(&self.plugin);
            let rx = rx.clone();
            let results = Arc::clone(&self.results);
            
            let handle = thread::spawn(move || {
                utils::debug(&format!("Worker {} started", worker_id));
                
                while let Ok(data) = rx.recv() {
                    utils::debug(&format!("Worker {} processing: {:?}", worker_id, data));
                    
                    match plugin.collect_data(&data) {
                        Ok(result) => {
                            let mut results = results.lock().unwrap();
                            results.extend(result);
                        },
                        Err(e) => {
                            println!("Worker {} error: {}", worker_id, e);
                        }
                    }
                }
                
                utils::debug(&format!("Worker {} finished", worker_id));
            });
            
            handles.push(handle);
        }

        // Feed data to workers
        {
            let data_queue = self.data_queue.lock().unwrap();
            for data in data_queue.iter() {
                tx.send(data.clone()).unwrap();
            }
        }
        
        // Signal workers to finish
        drop(tx);
        
        // Wait for all workers to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Return results
        let mut results = self.results.lock().unwrap();
        let results_clone = results.clone();

        results.clear();
        Ok(results_clone)
    }

    pub fn relinquish_plugin(&self) -> Arc<Plugin> {
        self.plugin.clone()
    }
} 