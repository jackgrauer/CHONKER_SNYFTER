#[cfg(feature = "advanced_pdf")]
pub mod fast_path;

#[cfg(feature = "advanced_pdf")]
pub use fast_path::FastPathProcessor;
