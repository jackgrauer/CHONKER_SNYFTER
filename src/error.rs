use thiserror::Error;

/// Main error type for the CHONKER application
#[derive(Error, Debug)]
pub enum ChonkerError {
    #[error("PDF processing failed: {message}")]
    PdfProcessing { 
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    #[error("Database operation failed: {operation}")]
    Database { 
        operation: String,
        #[source]
        source: sqlx::Error,
    },
    
    #[error("File I/O error: {path}")]
    FileIO { 
        path: String,
        #[source]
        source: std::io::Error,
    },
    
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    
    #[error("Processing timeout after {seconds} seconds")]
    ProcessingTimeout { seconds: u64 },
    
    #[error("Invalid document format: {format}")]
    InvalidFormat { format: String },
    
    #[error("Memory limit exceeded: {limit_mb}MB")]
    MemoryLimit { limit_mb: u64 },
    
    #[error("UI rendering error: {message}")]
    UIRender { message: String },
    
    #[error("System resource error: {resource}")]
    SystemResource { 
        resource: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("General error: {0}")]
    General(#[from] anyhow::Error),
}

impl ChonkerError {
    /// Create a PDF processing error with context
    pub fn pdf_processing(message: impl Into<String>) -> Self {
        Self::PdfProcessing {
            message: message.into(),
            source: None,
        }
    }
    
    /// Create a PDF processing error with source
    pub fn pdf_processing_with_source(
        message: impl Into<String>, 
        source: impl std::error::Error + Send + Sync + 'static
    ) -> Self {
        Self::PdfProcessing {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
    
    /// Create a database error
    pub fn database(operation: impl Into<String>, source: sqlx::Error) -> Self {
        Self::Database {
            operation: operation.into(),
            source,
        }
    }
    
    /// Create a file I/O error
    pub fn file_io(path: impl Into<String>, source: std::io::Error) -> Self {
        Self::FileIO {
            path: path.into(),
            source,
        }
    }
    
    /// Create a configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }
    
    /// Check if error is recoverable (can continue operation)
    pub fn is_recoverable(&self) -> bool {
        match self {
            ChonkerError::ProcessingTimeout { .. } => true,
            ChonkerError::InvalidFormat { .. } => true,
            ChonkerError::UIRender { .. } => true,
            ChonkerError::MemoryLimit { .. } => false,
            ChonkerError::Database { .. } => false,
            ChonkerError::SystemResource { .. } => false,
            _ => true,
        }
    }
    
    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            ChonkerError::PdfProcessing { .. } => {
                "🐹 CHONKER couldn't process this PDF. It might be encrypted or corrupted.".to_string()
            },
            ChonkerError::Database { .. } => {
                "💾 Database error occurred. Your data might not be saved.".to_string()
            },
            ChonkerError::FileIO { .. } => {
                "📁 File access error. Check file permissions and disk space.".to_string()
            },
            ChonkerError::ProcessingTimeout { seconds } => {
                format!("⏰ Processing took longer than {} seconds. Try a smaller document.", seconds)
            },
            ChonkerError::InvalidFormat { format } => {
                format!("📄 Unsupported format: {}. CHONKER only processes PDFs.", format)
            },
            ChonkerError::MemoryLimit { limit_mb } => {
                format!("🧠 Document too large ({}MB limit). Try splitting it first.", limit_mb)
            },
            _ => "🐹 Something went wrong. Check the logs for details.".to_string(),
        }
    }
}

/// Result type alias for convenience
pub type ChonkerResult<T> = Result<T, ChonkerError>;

/// Error context for adding additional information
pub trait ErrorContext<T> {
    fn with_context(self, context: &str) -> ChonkerResult<T>;
}

impl<T, E> ErrorContext<T> for Result<T, E> 
where 
    E: std::error::Error + Send + Sync + 'static 
{
    fn with_context(self, context: &str) -> ChonkerResult<T> {
        self.map_err(|e| ChonkerError::SystemResource {
            resource: context.to_string(),
            source: Box::new(e),
        })
    }
}
