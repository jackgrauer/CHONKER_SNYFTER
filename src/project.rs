use std::path::PathBuf;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Project management for CHONKER
/// Handles project creation, saving, loading, and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub documents: Vec<ProjectDocument>,
    pub project_path: Option<PathBuf>,
    pub settings: ProjectSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDocument {
    pub id: Uuid,
    pub file_path: PathBuf,
    pub original_filename: String,
    pub added_at: DateTime<Utc>,
    pub extraction_status: ExtractionStatus,
    pub extraction_tool: Option<String>,
    pub page_count: Option<usize>,
    pub corrections_count: usize,
    pub last_processed: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub auto_save: bool,
    pub preferred_extraction_tool: String,
    pub export_format: ExportFormat,
    pub backup_enabled: bool,
    pub workspace_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtractionStatus {
    Pending,
    Processing,
    Completed,
    Failed { error: String },
    RequiresReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Parquet,
    Csv,
    Json,
    Markdown,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            auto_save: true,
            preferred_extraction_tool: "auto".to_string(),
            export_format: ExportFormat::Parquet,
            backup_enabled: true,
            workspace_path: None,
        }
    }
}

impl Project {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            created_at: now,
            updated_at: now,
            documents: Vec::new(),
            project_path: None,
            settings: ProjectSettings::default(),
        }
    }
    
    pub fn add_document(&mut self, file_path: PathBuf) -> Uuid {
        let doc_id = Uuid::new_v4();
        let original_filename = file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        
        let document = ProjectDocument {
            id: doc_id,
            file_path,
            original_filename,
            added_at: Utc::now(),
            extraction_status: ExtractionStatus::Pending,
            extraction_tool: None,
            page_count: None,
            corrections_count: 0,
            last_processed: None,
        };
        
        self.documents.push(document);
        self.updated_at = Utc::now();
        
        doc_id
    }
    
    pub fn update_document_status(&mut self, doc_id: Uuid, status: ExtractionStatus) {
        if let Some(doc) = self.documents.iter_mut().find(|d| d.id == doc_id) {
            doc.extraction_status = status;
            doc.last_processed = Some(Utc::now());
            self.updated_at = Utc::now();
        }
    }
    
    pub fn get_document(&self, doc_id: Uuid) -> Option<&ProjectDocument> {
        self.documents.iter().find(|d| d.id == doc_id)
    }
    
    pub fn get_document_mut(&mut self, doc_id: Uuid) -> Option<&mut ProjectDocument> {
        self.documents.iter_mut().find(|d| d.id == doc_id)
    }
    
    pub fn remove_document(&mut self, doc_id: Uuid) -> bool {
        if let Some(pos) = self.documents.iter().position(|d| d.id == doc_id) {
            self.documents.remove(pos);
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }
    
    pub fn get_stats(&self) -> ProjectStats {
        let total_docs = self.documents.len();
        let completed_docs = self.documents.iter()
            .filter(|d| matches!(d.extraction_status, ExtractionStatus::Completed))
            .count();
        let failed_docs = self.documents.iter()
            .filter(|d| matches!(d.extraction_status, ExtractionStatus::Failed { .. }))
            .count();
        let pending_docs = self.documents.iter()
            .filter(|d| matches!(d.extraction_status, ExtractionStatus::Pending | ExtractionStatus::Processing))
            .count();
        let total_corrections = self.documents.iter()
            .map(|d| d.corrections_count)
            .sum();
        
        ProjectStats {
            total_documents: total_docs,
            completed_documents: completed_docs,
            failed_documents: failed_docs,
            pending_documents: pending_docs,
            total_corrections,
        }
    }
    
    pub fn save_to_file(&self, path: &PathBuf) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    pub fn load_from_file(path: &PathBuf) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let mut project: Project = serde_json::from_str(&json)?;
        project.project_path = Some(path.clone());
        Ok(project)
    }
    
    pub fn auto_save(&self) -> anyhow::Result<()> {
        if self.settings.auto_save {
            if let Some(ref path) = self.project_path {
                self.save_to_file(path)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ProjectStats {
    pub total_documents: usize,
    pub completed_documents: usize,
    pub failed_documents: usize,
    pub pending_documents: usize,
    pub total_corrections: usize,
}
