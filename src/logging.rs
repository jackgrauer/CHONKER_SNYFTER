use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
    Registry,
    Layer,
};
use tracing_appender::{rolling, non_blocking};

use crate::error::{ChonkerError, ChonkerResult};

/// Logging configuration for CHONKER
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub log_dir: PathBuf,
    pub enable_file_logging: bool,
    pub enable_json_format: bool,
    pub max_log_files: usize,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            log_dir: PathBuf::from("logs"),
            enable_file_logging: true,
            enable_json_format: false,
            max_log_files: 10,
        }
    }
}

/// Initialize the logging system for CHONKER
pub fn init_logging(config: &LoggingConfig) -> ChonkerResult<()> {
    // Create logs directory if it doesn't exist
    if config.enable_file_logging {
        fs::create_dir_all(&config.log_dir)
            .map_err(|e| ChonkerError::file_io(
                config.log_dir.to_string_lossy().to_string(), 
                e
            ))?;
    }

    // Set up environment filter with database log filtering for TUI
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            // Filter out noisy database logs to prevent TUI corruption
            EnvFilter::new(&format!(
                "chonker_tui={},sqlx=warn,{}", 
                config.level, config.level
            ))
        });

    let registry = Registry::default().with(env_filter);

    if config.enable_file_logging {
        // Set up file logging with rotation
        let file_appender = rolling::daily(&config.log_dir, "chonker.log");
        let (file_writer, _guard) = non_blocking(file_appender);

        let file_layer = if config.enable_json_format {
            fmt::layer()
                .json()
                .with_writer(file_writer)
                .boxed()
        } else {
            fmt::layer()
                .with_writer(file_writer)
                .with_ansi(false)
                .boxed()
        };

        // Console layer (completely disabled for TUI to prevent interference)
        // All logs go to file only when TUI is running
        let console_layer = fmt::layer()
            .with_writer(std::io::sink) // Send to nowhere
            .with_target(false)
            .without_time()
            .compact()
            .boxed();

        registry
            .with(file_layer)
            .with(console_layer)
            .init();
    } else {
        // Console-only logging
        let console_layer = fmt::layer()
            .with_writer(std::io::stderr)
            .with_target(false)
            .without_time()
            .compact()
            .boxed();

        registry.with(console_layer).init();
    }

    info!("🐹 CHONKER logging initialized");
    info!("Log level: {}", config.level);
    
    if config.enable_file_logging {
        info!("File logging enabled: {}", config.log_dir.display());
    }

    Ok(())
}

/// Log system information for debugging
pub fn log_system_info() {
    info!("🐹 CHONKER v10.0 - Spatial Intelligence Document Chunker");
    info!("System: {} {}", std::env::consts::OS, std::env::consts::ARCH);
    
    // Log memory info if available
    if let Ok(_metadata) = fs::metadata("/proc/meminfo") {
        info!("System memory info available");
    }
    
    // Log current working directory
    if let Ok(cwd) = std::env::current_dir() {
        info!("Working directory: {}", cwd.display());
    }
}

/// Performance logging utilities
pub struct PerformanceTimer {
    start: std::time::Instant,
    operation: String,
}

impl PerformanceTimer {
    pub fn start(operation: impl Into<String>) -> Self {
        let operation = operation.into();
        info!("⏱️  Starting: {}", operation);
        Self {
            start: std::time::Instant::now(),
            operation,
        }
    }
    
    pub fn checkpoint(&self, checkpoint: &str) {
        let elapsed = self.start.elapsed();
        info!("⏱️  {} - {}: {:.2}ms", self.operation, checkpoint, elapsed.as_millis());
    }
}

impl Drop for PerformanceTimer {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        info!("⏱️  Completed {}: {:.2}ms", self.operation, elapsed.as_millis());
    }
}

/// Memory usage logging
pub fn log_memory_usage(context: &str) {
    // Get process memory info (platform specific)
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("ps")
            .args(&["-o", "rss=", "-p"])
            .arg(std::process::id().to_string())
            .output()
        {
            if let Ok(memory_str) = String::from_utf8(output.stdout) {
                if let Ok(memory_kb) = memory_str.trim().parse::<u64>() {
                    let memory_mb = memory_kb / 1024;
                    info!("🧠 Memory usage ({}): {}MB", context, memory_mb);
                    
                    // Warn if memory usage is high
                    if memory_mb > 500 {
                        warn!("⚠️  High memory usage detected: {}MB", memory_mb);
                    }
                }
            }
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        info!("🧠 Memory monitoring not implemented for this platform");
    }
}

/// Clean up old log files
pub fn cleanup_old_logs(config: &LoggingConfig) -> ChonkerResult<()> {
    if !config.enable_file_logging {
        return Ok(());
    }
    
    let mut log_files = Vec::new();
    
    let entries = fs::read_dir(&config.log_dir)
        .map_err(|e| ChonkerError::file_io(
            config.log_dir.to_string_lossy().to_string(), 
            e
        ))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| ChonkerError::file_io(
            config.log_dir.to_string_lossy().to_string(), 
            e
        ))?;
        
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("log") {
            if let Ok(metadata) = fs::metadata(&path) {
                log_files.push((path, metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)));
            }
        }
    }
    
    // Sort by modification time (newest first)
    log_files.sort_by(|a, b| b.1.cmp(&a.1));
    
    // Remove old files if we exceed the limit
    if log_files.len() > config.max_log_files {
        let files_to_remove = &log_files[config.max_log_files..];
        for (path, _) in files_to_remove {
            if let Err(e) = fs::remove_file(path) {
                warn!("Failed to remove old log file {}: {}", path.display(), e);
            } else {
                info!("Removed old log file: {}", path.display());
            }
        }
    }
    
    Ok(())
}

/// Macro for logging with context
#[macro_export]
macro_rules! log_error {
    ($error:expr, $context:expr) => {
        tracing::error!(
            error = %$error,
            context = $context,
            recoverable = $error.is_recoverable(),
            "CHONKER error occurred"
        );
    };
}

#[macro_export]
macro_rules! log_processing_start {
    ($file:expr, $size:expr) => {
        tracing::info!(
            file = %$file,
            size_bytes = $size,
            "🐹 Starting PDF processing"
        );
    };
}

#[macro_export]
macro_rules! log_chunk_created {
    ($chunk_id:expr, $char_count:expr, $element_types:expr) => {
        tracing::debug!(
            chunk_id = $chunk_id,
            char_count = $char_count,
            element_types = ?$element_types,
            "Created document chunk"
        );
    };
}
