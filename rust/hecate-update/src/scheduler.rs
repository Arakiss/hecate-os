//! Update scheduling module
//!
//! Handles maintenance windows and workload-aware scheduling

use anyhow::Result;
use crate::{UpdatePlan, MaintenanceWindow};
use chrono::{Datelike, Local, Timelike, Weekday};
use tokio::sync::Mutex;
use std::sync::Arc;

pub struct UpdateScheduler {
    maintenance_window: MaintenanceWindow,
    scheduled_plans: Arc<Mutex<Vec<UpdatePlan>>>,
}

impl UpdateScheduler {
    pub fn new(maintenance_window: MaintenanceWindow) -> Result<Self> {
        Ok(Self {
            maintenance_window,
            scheduled_plans: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub async fn schedule_plan(&self, plan: UpdatePlan) -> Result<()> {
        let mut plans = self.scheduled_plans.lock().await;
        plans.push(plan);
        tracing::info!("Scheduled update plan for next maintenance window");
        Ok(())
    }

    pub async fn get_scheduled_plans(&self) -> Result<Vec<UpdatePlan>> {
        let plans = self.scheduled_plans.lock().await;
        Ok(plans.clone())
    }

    pub fn is_in_maintenance_window(&self) -> bool {
        let now = Local::now();
        let current_day = now.weekday();
        let current_hour = now.hour();
        
        // Check if today is a maintenance day
        if !self.maintenance_window.days.contains(&current_day) {
            return false;
        }
        
        // Check if current hour is within window
        current_hour >= self.maintenance_window.start_hour 
            && current_hour < self.maintenance_window.end_hour
    }

    pub fn next_maintenance_window(&self) -> chrono::DateTime<Local> {
        let now = Local::now();
        
        // Find next maintenance day
        for days_ahead in 0..7 {
            let check_date = now + chrono::Duration::days(days_ahead);
            let check_day = check_date.weekday();
            
            if self.maintenance_window.days.contains(&check_day) {
                // Set time to start of maintenance window
                let window_start = check_date
                    .with_hour(self.maintenance_window.start_hour)
                    .unwrap()
                    .with_minute(0)
                    .unwrap()
                    .with_second(0)
                    .unwrap();
                
                // If it's today but we've passed the window, continue to next day
                if days_ahead == 0 && now >= window_start {
                    continue;
                }
                
                return window_start;
            }
        }
        
        // Shouldn't reach here if days are configured correctly
        now + chrono::Duration::days(7)
    }

    pub async fn check_system_load(&self) -> Result<f64> {
        // Read system load average
        let loadavg = std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_else(|_| "0.0 0.0 0.0 0/0 0".to_string());
        
        let parts: Vec<&str> = loadavg.split_whitespace().collect();
        let load = parts.first()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        
        Ok(load)
    }

    pub async fn should_defer_update(&self) -> Result<bool> {
        // Check if system load is too high
        let load = self.check_system_load().await?;
        let cpu_count = std::thread::available_parallelism()
            .map(|n| n.get() as f64)
            .unwrap_or(1.0);
        
        // Defer if load is above 80% of CPU capacity
        if load > cpu_count * 0.8 {
            tracing::info!("Deferring update due to high system load: {:.2}", load);
            return Ok(true);
        }
        
        // Check for critical processes
        if self.has_critical_processes() {
            tracing::info!("Deferring update due to critical processes running");
            return Ok(true);
        }
        
        Ok(false)
    }

    fn has_critical_processes(&self) -> bool {
        // Check for processes that shouldn't be interrupted
        let critical_processes = vec![
            "backup",
            "rsync",
            "dd",
            "database",
            "mysql",
            "postgres",
            "oracle",
        ];
        
        if let Ok(entries) = std::fs::read_dir("/proc") {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if name.chars().all(|c| c.is_ascii_digit()) {
                        // It's a process directory
                        let cmdline_path = entry.path().join("cmdline");
                        if let Ok(cmdline) = std::fs::read_to_string(cmdline_path) {
                            let cmd_lower = cmdline.to_lowercase();
                            for critical in &critical_processes {
                                if cmd_lower.contains(critical) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        false
    }
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}