use std::path::PathBuf;
use std::process::Command;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

/// Extractor component that bridges to Python extraction tools
/// Manages Magic-PDF, Docling, and fallback extraction
pub struct Extractor {
    pub python_script_path: PathBuf,
    pub preferred_tool: String,
    pub extraction_cache: std::collections::HashMap<String, ExtractionResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub success: bool,
    pub tool: String,
    pub extractions: Vec<PageExtraction>,
    pub metadata: ExtractionMetadata,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageExtraction {
    pub page_number: usize,
    pub text: String,
    pub tables: Vec<serde_json::Value>,
    pub figures: Vec<serde_json::Value>,
    pub formulas: Vec<serde_json::Value>,
    pub confidence: f64,
    pub layout_boxes: Vec<serde_json::Value>,
    pub tool: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionMetadata {
    pub total_pages: usize,
    pub processing_time: u64,
}

impl Default for Extractor {
    fn default() -> Self {
        // Try to find the script relative to the executable or current directory
        let script_path = if let Ok(exe_path) = std::env::current_exe() {
            let exe_dir = exe_path.parent().unwrap_or_else(|| std::path::Path::new("."));
            let script_from_exe = exe_dir.join("../../../python/extraction_bridge.py");
            if script_from_exe.exists() {
                script_from_exe
            } else {
                PathBuf::from("python/extraction_bridge.py")
            }
        } else {
            PathBuf::from("python/extraction_bridge.py")
        };
        
        Self {
            python_script_path: script_path,
            preferred_tool: "auto".to_string(),
            extraction_cache: std::collections::HashMap::new(),
        }
    }
}

impl Extractor {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn set_python_script_path(&mut self, path: PathBuf) {
        self.python_script_path = path;
    }
    
    pub fn set_preferred_tool(&mut self, tool: String) {
        self.preferred_tool = tool;
    }
    
    pub async fn extract_pdf(&mut self, pdf_path: &PathBuf) -> Result<ExtractionResult> {
        // Check cache first
        let cache_key = format!("{}:{}", pdf_path.to_string_lossy(), self.preferred_tool);
        if let Some(cached_result) = self.extraction_cache.get(&cache_key) {
            return Ok(cached_result.clone());
        }
        
        // Run Python extraction bridge
        let result = self.run_python_extraction(pdf_path, None).await?;
        
        // Cache the result
        self.extraction_cache.insert(cache_key, result.clone());
        
        Ok(result)
    }
    
    pub async fn extract_page(&mut self, pdf_path: &PathBuf, page_num: usize) -> Result<ExtractionResult> {
        let cache_key = format!("{}:{}:{}", pdf_path.to_string_lossy(), page_num, self.preferred_tool);
        if let Some(cached_result) = self.extraction_cache.get(&cache_key) {
            return Ok(cached_result.clone());
        }
        
        let result = self.run_python_extraction(pdf_path, Some(page_num)).await?;
        
        self.extraction_cache.insert(cache_key, result.clone());
        
        Ok(result)
    }
    
    async fn run_python_extraction(&self, pdf_path: &PathBuf, page_num: Option<usize>) -> Result<ExtractionResult> {
        let mut cmd = Command::new("python3");
        cmd.arg(&self.python_script_path);
        cmd.arg(pdf_path);
        cmd.arg("--tool").arg(&self.preferred_tool);
        
        if let Some(page) = page_num {
            cmd.arg("--page").arg(page.to_string());
        }
        
        let output = cmd.output().map_err(|e| {
            anyhow!("Failed to run Python extraction script: {}. Make sure Python 3 is installed and the script is accessible.", e)
        })?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Python extraction failed: {}", stderr));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let result: ExtractionResult = serde_json::from_str(&stdout).map_err(|e| {
            anyhow!("Failed to parse extraction result: {}. Output was: {}", e, stdout)
        })?;
        
        if !result.success {
            return Err(anyhow!("Extraction failed: {}", result.error.unwrap_or_default()));
        }
        
        Ok(result)
    }
    
    pub fn clear_cache(&mut self) {
        self.extraction_cache.clear();
    }
    
    pub fn get_cache_stats(&self) -> (usize, usize) {
        let entries = self.extraction_cache.len();
        let memory_estimate = self.extraction_cache.iter()
            .map(|(k, v)| k.len() + serde_json::to_string(v).unwrap_or_default().len())
            .sum::<usize>();
        
        (entries, memory_estimate)
    }
}
