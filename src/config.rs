use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChonkerConfig {
    pub routing: RoutingConfig,
    pub processing: ProcessingConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    /// Complexity threshold: <= goes to Rust, > goes to Python
    pub complexity_threshold: f32,
    
    /// Force Python for these file types
    pub force_python_for_types: Vec<String>,
    
    /// Enable fallback from Rust to Python on failure
    pub enable_fallback: bool,
    
    /// Maximum file size for fast path (MB)
    pub max_fast_path_size_mb: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    /// Number of parallel workers
    pub parallel_workers: usize,
    
    /// Enable caching
    pub enable_caching: bool,
    
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Enable telemetry collection
    pub enable_telemetry: bool,
    
    /// Auto-vacuum database
    pub auto_vacuum: bool,
}

impl Default for ChonkerConfig {
    fn default() -> Self {
        Self {
            routing: RoutingConfig {
                complexity_threshold: 3.0,
                force_python_for_types: vec!["docx".to_string(), "xlsx".to_string()],
                enable_fallback: true,
                max_fast_path_size_mb: 10.0,
            },
            processing: ProcessingConfig {
                parallel_workers: 4,
                enable_caching: true,
                cache_ttl_seconds: 3600,
            },
            database: DatabaseConfig {
                enable_telemetry: true,
                auto_vacuum: true,
            },
        }
    }
}

impl ChonkerConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| anyhow!("Failed to read config file: {}", e))?;
        
        let config: ChonkerConfig = toml::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse config file: {}", e))?;
        
        Ok(config)
    }
    
    pub fn load_from_env() -> Self {
        let mut config = Self::default();
        
        // Override with environment variables
        if let Ok(threshold) = std::env::var("CHONKER_COMPLEXITY_THRESHOLD") {
            if let Ok(value) = threshold.parse::<f32>() {
                config.routing.complexity_threshold = value;
            }
        }
        
        if let Ok(fallback) = std::env::var("CHONKER_ENABLE_FALLBACK") {
            config.routing.enable_fallback = fallback.to_lowercase() == "true";
        }
        
        config
    }
    
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;
        
        std::fs::write(path.as_ref(), content)
            .map_err(|e| anyhow!("Failed to write config file: {}", e))?;
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum ToolPreference {
    Auto,
    Rust,
    Python,
}

impl From<&str> for ToolPreference {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "rust" | "native" | "fast" => ToolPreference::Rust,
            "python" | "ml" | "complex" => ToolPreference::Python,
            _ => ToolPreference::Auto,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_default_config() {
        let config = ChonkerConfig::default();
        assert_eq!(config.routing.complexity_threshold, 3.0);
        assert!(config.routing.enable_fallback);
    }

    #[test]
    fn test_config_serialization() {
        let config = ChonkerConfig::default();
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        config.save_to_file(&config_path).unwrap();
        
        let loaded_config = ChonkerConfig::load_from_file(&config_path).unwrap();
        assert_eq!(loaded_config.routing.complexity_threshold, 3.0);
    }

    #[test] 
    fn test_tool_preference_parsing() {
        assert!(matches!(ToolPreference::from("auto"), ToolPreference::Auto));
        assert!(matches!(ToolPreference::from("rust"), ToolPreference::Rust));
        assert!(matches!(ToolPreference::from("python"), ToolPreference::Python));
    }
}
