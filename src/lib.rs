#![allow(dead_code)]

// Public module exports for binary crates
#[cfg(feature = "gui")]
pub mod app;
pub mod database;
pub mod error;
pub mod cli;
pub mod config;
pub mod smart_chunker;
pub mod document_model;
#[cfg(feature = "advanced_pdf")]
pub mod native_extractor;
pub mod processing;
pub mod tui_simple;
pub mod export;
pub mod pdf;
#[cfg(feature = "advanced_pdf")]
pub mod analyzer;
pub mod extractor;
pub mod logging;
pub mod smart_column_extractor;
#[cfg(all(feature = "mupdf", feature = "gui"))]
pub mod mupdf_viewer;
#[cfg(feature = "gui")]
pub mod markdown_editor;
#[cfg(feature = "gui")]
pub mod coordinate_mapping;
#[cfg(feature = "gui")]
pub mod validation_editor;
#[cfg(feature = "gui")]
pub mod data_visualization;
pub mod html_extractor;
#[cfg(feature = "gui")]
pub mod extraction_integration;
pub mod sync;
pub mod project;
