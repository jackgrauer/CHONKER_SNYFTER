pub mod parser;
pub mod formatter;
pub mod detector;
pub mod cleaner;

pub use cleaner::TableCleaner;
pub use parser::{ParsedTable, TableRow, TableCell, LineType};
pub use formatter::{TableFormatter, ColumnAlignment};
pub use detector::{SmartTableDetector, TableRegion};

#[derive(Debug, Default)]
pub struct ProcessingChanges {
    pub total_tables: usize,
    pub tables_fixed: usize,
    pub tables_skipped: usize,
    pub empty_cells_removed: usize,
    pub columns_normalized: usize,
}

impl ProcessingChanges {
    pub fn summary(&self) -> String {
        format!(
            "Processed {} tables: {} fixed, {} skipped. Removed {} empty cells, normalized {} columns.",
            self.total_tables, self.tables_fixed, self.tables_skipped,
            self.empty_cells_removed, self.columns_normalized
        )
    }
}
