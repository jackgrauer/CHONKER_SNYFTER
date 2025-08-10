mod spatial;
use anyhow::Result;
use copypasta::{ClipboardContext, ClipboardProvider};
use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use image::{DynamicImage, RgbaImage};
use pdfium_render::prelude::*;
use ratatui::{prelude::*, widgets::*};
use ratatui_image::picker::Picker;
use ratatui_image::{protocol::StatefulProtocol, StatefulImage};
use rfd::FileDialog;
use spatial::Spatial;
use std::path::PathBuf;
use std::time::{Duration, Instant};

mod pdf_cache;

// ============= THEME SYSTEM =============
#[derive(Clone, Copy, Debug)]
enum Theme {
    Dark,
    Light,
}

struct ThemeColors {
    bg: Color,
    fg: Color,
    teal: Color,
    highlight: Color,
    dim: Color,
    error: Color,
    yellow: Color,
    green: Color,
    blue: Color,
    chrome: Color,
}

impl Theme {
    fn colors(&self) -> ThemeColors {
        match self {
            Theme::Dark => ThemeColors {
                bg: Color::Rgb(10, 15, 20),
                fg: Color::Rgb(26, 188, 156),
                teal: Color::Rgb(26, 188, 156),
                highlight: Color::Rgb(22, 160, 133),
                dim: Color::Rgb(80, 100, 100),
                error: Color::Rgb(255, 80, 80),
                yellow: Color::Rgb(255, 200, 0),
                green: Color::Rgb(46, 204, 113),
                blue: Color::Rgb(52, 152, 219),
                chrome: Color::Rgb(82, 86, 89),
            },
            Theme::Light => ThemeColors {
                bg: Color::Rgb(250, 250, 250),
                fg: Color::Rgb(40, 40, 40),
                teal: Color::Rgb(0, 128, 128),
                highlight: Color::Rgb(0, 150, 150),
                dim: Color::Rgb(150, 150, 150),
                error: Color::Rgb(200, 0, 0),
                yellow: Color::Rgb(180, 140, 0),
                green: Color::Rgb(0, 150, 0),
                blue: Color::Rgb(0, 100, 200),
                chrome: Color::Rgb(200, 200, 200),
            },
        }
    }
}

// ============= MATRIX SELECTION =============
#[derive(Clone, Debug)]
struct MatrixSelection {
    start: Option<(usize, usize)>,
    end: Option<(usize, usize)>,
}

impl MatrixSelection {
    fn new() -> Self {
        Self {
            start: None,
            end: None,
        }
    }

    fn is_selected(&self, row: usize, col: usize) -> bool {
        if let (Some(start), Some(end)) = (self.start, self.end) {
            let min_row = start.0.min(end.0);
            let max_row = start.0.max(end.0);
            let min_col = start.1.min(end.1);
            let max_col = start.1.max(end.1);
            row >= min_row && row <= max_row && col >= min_col && col <= max_col
        } else {
            false
        }
    }

    fn get_selected_text(&self, matrix: &[Vec<char>]) -> String {
        if let (Some(start), Some(end)) = (self.start, self.end) {
            let min_row = start.0.min(end.0).min(matrix.len().saturating_sub(1));
            let max_row = start.0.max(end.0).min(matrix.len().saturating_sub(1));
            let min_col = start.1.min(end.1);
            let max_col = start.1.max(end.1);

            // Limit selection size to prevent performance issues
            if (max_row - min_row + 1) * (max_col - min_col + 1) > 100000 {
                return String::from("[Selection too large]");
            }

            let mut result =
                String::with_capacity((max_row - min_row + 1) * (max_col - min_col + 2));
            for row in min_row..=max_row {
                if row < matrix.len() {
                    let row_data = &matrix[row];
                    // Get exactly the selected columns, padding with spaces if needed
                    for col in min_col..=max_col {
                        if col < row_data.len() {
                            result.push(row_data[col]);
                        } else {
                            result.push(' '); // Pad with space to maintain block shape
                        }
                    }
                    if row < max_row {
                        result.push('\n');
                    }
                }
            }
            result
        } else {
            String::new()
        }
    }

    fn clear(&mut self) {
        self.start = None;
        self.end = None;
    }
}

// ============= CHARACTER MATRIX =============
#[derive(Clone)]
struct CharacterMatrix {
    width: usize,
    height: usize,
    matrix: Vec<Vec<char>>,
}

impl CharacterMatrix {
    fn from_text(text: &str) -> Self {
        let lines: Vec<Vec<char>> = text
            .lines()
            .map(|line| {
                // Strip line numbers if present
                if let Some(pos) = line.find(' ') {
                    line[pos + 1..].chars().collect()
                } else {
                    line.chars().collect()
                }
            })
            .collect();

        let height = lines.len();
        let width = lines.iter().map(|l| l.len()).max().unwrap_or(0);

        // Pad all lines to same width
        let mut matrix = Vec::new();
        for line in lines {
            let mut padded = line.clone();
            padded.resize(width, ' ');
            matrix.push(padded);
        }

        Self {
            width,
            height,
            matrix,
        }
    }
}

// ============= PANE FOCUS =============
#[derive(Clone, Copy, PartialEq, Debug)]
enum TextViewMode {
    RawMatrix,
    SmartLayout,
}

// ============= SIMPLE TUI STRUCT =============
struct ChonkerTUI {
    // PDF state
    pdf_path: Option<PathBuf>,
    current_page: usize,
    total_pages: usize,
    zoom_level: f32,
    pdf_render_cache: Option<String>,
    pdf_image: Option<DynamicImage>,
    image_picker: Option<Picker>,
    image_protocol: Option<Box<dyn StatefulProtocol>>,

    // Matrix state
    character_matrix: Option<CharacterMatrix>,
    editable_matrix: Option<Vec<Vec<char>>>,
    matrix_modified: bool,

    // Smart layout state
    smart_layout_text: Option<String>,
    smart_layout_scroll: u16,

    // UI state
    text_view_mode: TextViewMode,
    split_ratio: u16,
    theme: Theme,

    // Cursor and selection
    cursor: (usize, usize),
    selection: MatrixSelection,
    is_selecting: bool,

    // Clipboard
    clipboard: Vec<Vec<char>>,

    // Scrolling
    pdf_scroll: (u16, u16),
    matrix_scroll: (u16, u16),

    // Search
    search_query: String,
    search_results: Vec<(usize, usize)>,
    current_search_index: usize,

    // Status and messages
    status_message: String,
    show_help: bool,
    show_line_numbers: bool,

    // Performance
    cursor_blink_state: bool,
    last_blink_time: Instant,

    // File input
    file_input_active: bool,
    file_input_buffer: String,

    // Search input
    search_input_active: bool,

    // Advanced caching
    cache_hits: usize,
    cache_misses: usize,
}

impl ChonkerTUI {
    fn new() -> Self {
        // Initialize image picker for terminal protocol detection
        // Using font size 8x18 as a reasonable default
        let mut picker = Picker::new((8, 18));
        picker.guess_protocol();

        Self {
            pdf_path: None,
            current_page: 0,
            total_pages: 0,
            zoom_level: 1.0, // Start at 100% zoom for safety
            pdf_render_cache: None,
            pdf_image: None,
            image_picker: Some(picker),
            image_protocol: None,
            character_matrix: None,
            editable_matrix: None,
            matrix_modified: false,
            smart_layout_text: None,
            smart_layout_scroll: 0,
            text_view_mode: TextViewMode::RawMatrix,
            split_ratio: 50,
            theme: Theme::Dark,
            cursor: (0, 0),
            selection: MatrixSelection::new(),
            is_selecting: false,
            clipboard: Vec::new(),
            pdf_scroll: (0, 0),
            matrix_scroll: (0, 0),
            search_query: String::new(),
            search_results: Vec::new(),
            current_search_index: 0,
            status_message: "Press Ctrl+O to open PDF, Ctrl+H for help".to_string(),
            show_help: false,
            show_line_numbers: true,
            cursor_blink_state: true,
            last_blink_time: Instant::now(),
            file_input_active: false,
            file_input_buffer: String::new(),
            search_input_active: false,
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    fn open_pdf(&mut self, path: PathBuf) -> Result<()> {
        if path.exists() {
            self.pdf_path = Some(path.clone());

            // Initialize PDFium and extract page count + render first page
            let (total_pages, pdf_image) = {
                let pdfium = Pdfium::new(
                    Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./lib/"))
                        .or_else(|_| Pdfium::bind_to_system_library())?,
                );

                let result = pdfium.load_pdf_from_file(&path, None);
                match result {
                    Ok(document) => {
                        let total = document.pages().len() as usize;

                        // Render first page as image for display
                        let page = document.pages().get(0).unwrap();
                        // Get terminal size for initial render
                        let term_size = crossterm::terminal::size().unwrap_or((80, 24));
                        let target_width = ((term_size.0 as f32 * 0.5) * 7.0) as i32; // Half terminal width * char width
                        let target_height = ((term_size.1 as f32 - 7.0) * 14.0) as i32; // Terminal height * char height

                        let render_config = PdfRenderConfig::new()
                            .set_target_width(target_width)
                            .set_maximum_height(target_height);

                        let bitmap = page.render_with_config(&render_config)?;

                        // Get the actual image data from the bitmap
                        let width = bitmap.width() as u32;
                        let height = bitmap.height() as u32;
                        let bytes = bitmap.as_rgba_bytes().to_vec();

                        // Create image from the actual bitmap data
                        let rgba_image = RgbaImage::from_raw(width, height, bytes)
                            .ok_or_else(|| anyhow::anyhow!("Failed to create image from bitmap"))?;

                        (total, Some(DynamicImage::ImageRgba8(rgba_image)))
                    }
                    Err(e) => {
                        self.status_message = format!("Failed to load PDF: {}", e);
                        return Ok(());
                    }
                }
            };

            self.total_pages = total_pages;
            self.current_page = 0;
            self.pdf_image = pdf_image;
            self.image_protocol = None; // Reset image protocol for new PDF
            self.render_current_page()?;
            self.status_message = format!(
                "Loaded: {} ({} pages)",
                path.file_name().unwrap_or_default().to_string_lossy(),
                self.total_pages
            );
        } else {
            self.status_message = format!("File not found: {}", path.display());
        }
        Ok(())
    }

    fn render_current_page(&mut self) -> Result<()> {
        // Skip image rendering if zoom is outside safe range to prevent crashes
        if self.zoom_level < 0.8 || self.zoom_level > 1.2 {
            self.pdf_render_cache = Some(format!(
                "Page {}/{}\nZoom: {:.0}%\n\n[Image disabled - zoom out of range]\nZoom must be between 80% and 120%",
                self.current_page + 1,
                self.total_pages,
                self.zoom_level * 100.0
            ));
            self.pdf_image = None;
            return Ok(());
        }

        if let Some(pdf_path) = &self.pdf_path.clone() {
            self.cache_misses += 1;

            // Render current page as image
            let pdfium = Pdfium::new(
                Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./lib/"))
                    .or_else(|_| Pdfium::bind_to_system_library())?,
            );

            if let Ok(document) = pdfium.load_pdf_from_file(&pdf_path, None) {
                if let Ok(page) = document.pages().get(self.current_page as u16) {
                    // Calculate render size based on terminal and zoom
                    let term_size = crossterm::terminal::size().unwrap_or((80, 24));
                    let base_width = ((term_size.0 as f32 * 0.5) * 7.0) as i32;
                    let base_height = ((term_size.1 as f32 - 7.0) * 14.0) as i32;

                    // Clamp render dimensions to prevent overflow and crashes
                    // Very conservative limits to avoid any index out of bounds
                    let target_width =
                        ((base_width as f32 * self.zoom_level * 2.0) as i32).clamp(500, 1500);
                    let target_height =
                        ((base_height as f32 * self.zoom_level * 2.0) as i32).clamp(500, 1500);

                    let render_config = PdfRenderConfig::new()
                        .set_target_width(target_width)
                        .set_maximum_height(target_height);

                    let bitmap = page.render_with_config(&render_config)?;

                    // Get the actual image data from the bitmap
                    let width = bitmap.width() as u32;
                    let height = bitmap.height() as u32;
                    let bytes = bitmap.as_rgba_bytes().to_vec();

                    // Create image from the actual bitmap data
                    if let Some(rgba_image) = RgbaImage::from_raw(width, height, bytes) {
                        self.pdf_image = Some(DynamicImage::ImageRgba8(rgba_image));
                        // Reset image protocol when changing pages to force re-render
                        self.image_protocol = None;
                    }
                }
            }

            // Keep text representation as fallback
            self.pdf_render_cache = Some(format!(
                "Page {}/{}",
                self.current_page + 1,
                self.total_pages
            ));
        }
        Ok(())
    }

    fn extract_smart_layout(&mut self) -> Result<()> {
        if self.pdf_path.is_none() {
            self.status_message = "No PDF loaded".to_string();
            return Ok(());
        }

        // Simulate smart layout extraction (would use Ferrules or similar in production)
        let sample_layout = r#"
SMART LAYOUT EXTRACTION
══════════════════════

Document Structure:
├── Header
│   └── Title: "Sample Document"
├── Sections
│   ├── Section 1: Introduction
│   │   └── Paragraphs: 3
│   ├── Section 2: Main Content
│   │   ├── Paragraphs: 5
│   │   └── Table: 3x4
│   └── Section 3: Conclusion
│       └── Paragraphs: 2
└── Footer
    └── Page Numbers

Detected Elements:
• Tables: 1
• Images: 0
• Lists: 2
• Headers: 3
• Paragraphs: 10

Layout Analysis:
- Column Layout: Single
- Reading Order: Top-to-bottom
- Font Families: 2
- Dominant Language: English
"#;

        self.smart_layout_text = Some(sample_layout.to_string());
        self.status_message = "Smart layout extracted".to_string();
        Ok(())
    }

    fn extract_matrix(&mut self) -> Result<()> {
        if let Some(pdf_path) = &self.pdf_path.clone() {
            // Use fixed dimensions to extract the whole page, not just viewport
            // This ensures we get all text regardless of zoom level
            let mw = 200; // Wide enough for most PDFs
            let mh = 100; // Tall enough for most pages

            // CREATE PDFIUM AND EXTRACT, PROCESS ALL IN ONE EXPRESSION
            let result = {
                let pdfium = Pdfium::new(
                    Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./lib/"))
                        .or_else(|_| Pdfium::bind_to_system_library())?,
                );

                pdfium
                    .load_pdf_from_file(&pdf_path, None)
                    .ok()
                    .and_then(|document| {
                        Spatial::extract(&document, self.current_page, mw, mh).ok()
                    })
            };

            if let Some(matrix) = result {
                // UPDATE STATE
                self.character_matrix = Some(CharacterMatrix {
                    width: matrix[0].len(),
                    height: matrix.len(),
                    matrix: matrix.clone(),
                });
                self.editable_matrix = Some(matrix.clone());

                let txt_count = matrix
                    .iter()
                    .flat_map(|r| r.iter())
                    .filter(|&&c| c != ' ')
                    .count();
                self.status_message = format!(
                    "SPATIAL: {}x{} grid, {} chars",
                    matrix[0].len(),
                    matrix.len(),
                    txt_count
                );
            } else {
                self.status_message = "Failed to extract text from PDF".to_string();
            }
        } else {
            self.status_message = "No PDF loaded".to_string();
        }
        Ok(())
    }

    fn perform_search(&mut self) {
        if self.search_query.is_empty() {
            return;
        }

        self.search_results.clear();

        if let Some(matrix) = &self.editable_matrix {
            for (row_idx, row) in matrix.iter().enumerate() {
                let row_str: String = row.iter().collect();
                for (col_idx, _) in row_str.match_indices(&self.search_query) {
                    self.search_results.push((row_idx, col_idx));
                }
            }
        }

        if !self.search_results.is_empty() {
            self.current_search_index = 0;
            let (row, col) = self.search_results[0];
            self.cursor = (row, col);
            self.status_message = format!("Found {} matches", self.search_results.len());
        } else {
            self.status_message = format!("No matches found for '{}'", self.search_query);
        }
    }

    fn next_search_result(&mut self) {
        if !self.search_results.is_empty() {
            self.current_search_index = (self.current_search_index + 1) % self.search_results.len();
            let (row, col) = self.search_results[self.current_search_index];
            self.cursor = (row, col);
            self.status_message = format!(
                "Match {}/{}",
                self.current_search_index + 1,
                self.search_results.len()
            );
        }
    }

    fn prev_search_result(&mut self) {
        if !self.search_results.is_empty() {
            if self.current_search_index == 0 {
                self.current_search_index = self.search_results.len() - 1;
            } else {
                self.current_search_index -= 1;
            }
            let (row, col) = self.search_results[self.current_search_index];
            self.cursor = (row, col);
            self.status_message = format!(
                "Match {}/{}",
                self.current_search_index + 1,
                self.search_results.len()
            );
        }
    }

    fn sanitize_clipboard_text(&self, text: &str) -> String {
        // Preserve spaces for rectangular blocks - minimal sanitization
        text.chars()
            .map(|ch| {
                if ch == '\t' {
                    // Replace tabs with 4 spaces
                    "    ".to_string()
                } else if ch == '\r' {
                    // Convert CR to LF
                    "\n".to_string()
                } else if ch.is_control() && ch != '\n' && ch != ' ' {
                    // Skip control chars except newline and space
                    String::new()
                } else {
                    // Keep everything else including spaces
                    ch.to_string()
                }
            })
            .collect::<String>()
    }

    fn is_rectangular_block(&self, text: &str) -> bool {
        // Check if clipboard content is a rectangular block (uniform line lengths)
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() <= 1 {
            return false;
        }

        // Check if all lines have similar length (allowing small variations)
        let first_len = lines[0].len();
        let all_similar = lines.iter().all(|line| {
            let len = line.len();
            (len as i32 - first_len as i32).abs() <= 2 // Allow 2 char difference
        });

        // Also check if lines have consistent leading spaces (columnar data)
        if all_similar {
            // Check for consistent indentation suggesting column data
            let leading_spaces: Vec<usize> = lines
                .iter()
                .map(|line| line.chars().take_while(|c| *c == ' ').count())
                .collect();

            // If all lines have same leading spaces, it's likely a rectangular block
            if !leading_spaces.is_empty() {
                let first_spaces = leading_spaces[0];
                return leading_spaces.iter().all(|&spaces| spaces == first_spaces);
            }
        }

        all_similar
    }

    fn copy_selection(&mut self) {
        if let Some(matrix) = &self.editable_matrix {
            if self.selection.start.is_some() && self.selection.end.is_some() {
                let text = self.selection.get_selected_text(matrix);

                // Copy to system clipboard
                if let Ok(mut ctx) = ClipboardContext::new() {
                    if ctx.set_contents(text.clone()).is_ok() {
                        self.status_message = "Copied to system clipboard".to_string();
                    } else {
                        self.status_message = "Failed to copy to clipboard".to_string();
                    }
                } else {
                    self.status_message = "Clipboard not available".to_string();
                }

                // Also keep internal copy for fallback
                let lines: Vec<Vec<char>> = text.lines().map(|l| l.chars().collect()).collect();
                self.clipboard = lines;
            }
        }
    }

    fn cut_selection(&mut self) {
        self.copy_selection();
        self.delete_selection();
    }

    fn delete_selection(&mut self) {
        if let Some(matrix) = &mut self.editable_matrix {
            if let (Some(start), Some(end)) = (self.selection.start, self.selection.end) {
                let min_row = start.0.min(end.0);
                let max_row = start.0.max(end.0);
                let min_col = start.1.min(end.1);
                let max_col = start.1.max(end.1);

                for row in min_row..=max_row {
                    if row < matrix.len() {
                        for col in min_col..=max_col {
                            if col < matrix[row].len() {
                                matrix[row][col] = ' ';
                            }
                        }
                    }
                }

                self.matrix_modified = true;
                self.selection.clear();
                self.status_message = "Deleted selection".to_string();
            }
        }
    }

    fn paste_text_directly(&mut self, text: String) {
        // Direct paste without clipboard provider (already clean from pbpaste)
        let sanitized_text = self.sanitize_clipboard_text(&text);

        if let Some(matrix) = &mut self.editable_matrix {
            let (start_row, start_col) = self.cursor;
            let lines: Vec<&str> = sanitized_text.lines().collect();

            for (row_offset, line) in lines.iter().enumerate() {
                let target_row = start_row + row_offset;
                if target_row >= matrix.len() {
                    let width = if matrix.is_empty() {
                        80
                    } else {
                        matrix[0].len()
                    };
                    matrix.resize(target_row + 1, vec![' '; width]);
                }

                for (col_offset, ch) in line.chars().enumerate() {
                    let target_col = start_col + col_offset;
                    if target_col >= matrix[target_row].len() {
                        matrix[target_row].resize(target_col + 1, ' ');
                    }
                    matrix[target_row][target_col] = ch;
                }
            }

            self.matrix_modified = true;
            self.status_message = format!("Pasted {} lines (direct)", lines.len());
        }
    }

    fn paste_clipboard(&mut self) {
        // Try to get from system clipboard first
        let clipboard_text = if let Ok(mut ctx) = ClipboardContext::new() {
            ctx.get_contents().ok()
        } else {
            None
        };

        if let Some(text) = clipboard_text {
            // Sanitize the text to remove control codes
            let sanitized_text = self.sanitize_clipboard_text(&text);

            // Ensure we have a matrix to paste into
            if self.editable_matrix.is_none() {
                // Initialize empty matrix if needed
                self.editable_matrix = Some(vec![vec![' '; 80]; 25]);
            }

            // Check if this is a rectangular block first (before borrowing matrix)
            let is_rect_block = self.is_rectangular_block(&sanitized_text);

            // Use system clipboard content - paste as a block
            if let Some(matrix) = &mut self.editable_matrix {
                let (start_row, start_col) = self.cursor;
                let lines: Vec<&str> = sanitized_text.lines().collect();

                // If empty text, treat as single space
                let lines = if lines.is_empty() { vec![" "] } else { lines };

                // If rectangular block, preserve the column structure
                if is_rect_block && !lines.is_empty() {
                    // Find the minimum leading spaces (leftmost column position)
                    let min_leading = lines
                        .iter()
                        .map(|line| line.chars().take_while(|c| *c == ' ').count())
                        .min()
                        .unwrap_or(0);

                    // Paste preserving relative column positions
                    for (row_offset, line) in lines.iter().enumerate() {
                        let target_row = start_row + row_offset;
                        if target_row >= matrix.len() {
                            let width = if matrix.is_empty() {
                                80
                            } else {
                                matrix[0].len().max(80)
                            };
                            matrix.resize(target_row + 1, vec![' '; width]);
                        }

                        // Skip the minimum leading spaces, then paste from cursor
                        let trimmed_line = if min_leading < line.len() {
                            &line[min_leading..]
                        } else {
                            ""
                        };

                        for (col_offset, ch) in trimmed_line.chars().enumerate() {
                            let target_col = start_col + col_offset;
                            if target_row < matrix.len() {
                                if target_col >= matrix[target_row].len() {
                                    matrix[target_row].resize(target_col + 1, ' ');
                                }
                                matrix[target_row][target_col] = ch;
                            }
                        }
                    }
                } else {
                    // Regular paste for non-rectangular content
                    for (row_offset, line) in lines.iter().enumerate() {
                        let target_row = start_row + row_offset;
                        if target_row >= matrix.len() {
                            let width = if matrix.is_empty() {
                                80
                            } else {
                                matrix[0].len().max(80)
                            };
                            matrix.resize(target_row + 1, vec![' '; width]);
                        }

                        // Paste each character of the line starting at start_col
                        for (col_offset, ch) in line.chars().enumerate() {
                            let target_col = start_col + col_offset;
                            if target_row < matrix.len() {
                                if target_col >= matrix[target_row].len() {
                                    matrix[target_row].resize(target_col + 1, ' ');
                                }
                                matrix[target_row][target_col] = ch;
                            }
                        }
                    }
                }

                self.matrix_modified = true;
                self.status_message = format!("Pasted {} lines", lines.len());
            }
        } else if !self.clipboard.is_empty() {
            // Fallback to internal clipboard
            if let Some(matrix) = &mut self.editable_matrix {
                let (start_row, start_col) = self.cursor;

                for (row_offset, clip_row) in self.clipboard.iter().enumerate() {
                    let target_row = start_row + row_offset;
                    if target_row >= matrix.len() {
                        matrix.resize(target_row + 1, vec![' '; matrix[0].len()]);
                    }

                    for (col_offset, &ch) in clip_row.iter().enumerate() {
                        let target_col = start_col + col_offset;
                        if target_col >= matrix[target_row].len() {
                            matrix[target_row].resize(target_col + 1, ' ');
                        }
                        matrix[target_row][target_col] = ch;
                    }
                }

                self.matrix_modified = true;
                self.status_message = "Pasted from internal clipboard".to_string();
            }
        } else {
            self.status_message = "Nothing to paste".to_string();
        }
    }

    fn export_matrix(&mut self) -> Result<()> {
        if let Some(matrix) = &self.editable_matrix {
            // Use native save dialog
            let default_name = format!(
                "matrix_export_{}.txt",
                chrono::Local::now().format("%Y%m%d_%H%M%S")
            );

            if let Some(export_path) = FileDialog::new()
                .set_file_name(&default_name)
                .add_filter("Text files", &["txt"])
                .add_filter("All files", &["*"])
                .save_file()
            {
                let mut content = String::new();
                for (idx, row) in matrix.iter().enumerate() {
                    if self.show_line_numbers {
                        content.push_str(&format!("{:4} ", idx + 1));
                    }
                    content.push_str(&row.iter().collect::<String>());
                    content.push('\n');
                }

                std::fs::write(&export_path, content)?;
                self.status_message = format!("Exported to {}", export_path.display());
            } else {
                self.status_message = "Export cancelled".to_string();
            }
        } else {
            self.status_message = "No matrix to export".to_string();
        }
        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> Result<bool> {
        // Update cursor blink (faster at 300ms)
        if self.last_blink_time.elapsed() > Duration::from_millis(300) {
            self.cursor_blink_state = !self.cursor_blink_state;
            self.last_blink_time = Instant::now();
        }

        // Handle file input mode
        if self.file_input_active {
            match event {
                Event::Key(key) => match key.code {
                    KeyCode::Enter => {
                        let path = PathBuf::from(&self.file_input_buffer);
                        self.open_pdf(path)?;
                        self.file_input_active = false;
                        self.file_input_buffer.clear();
                    }
                    KeyCode::Esc => {
                        self.file_input_active = false;
                        self.file_input_buffer.clear();
                        self.status_message = "Cancelled".to_string();
                    }
                    KeyCode::Backspace => {
                        self.file_input_buffer.pop();
                    }
                    KeyCode::Char(c) => {
                        self.file_input_buffer.push(c);
                    }
                    _ => {}
                },
                _ => {}
            }
            return Ok(false);
        }

        // Handle search input mode
        if self.search_input_active {
            match event {
                Event::Key(key) => match key.code {
                    KeyCode::Enter => {
                        self.perform_search();
                        self.search_input_active = false;
                    }
                    KeyCode::Esc => {
                        self.search_input_active = false;
                        self.search_query.clear();
                        self.status_message = "Search cancelled".to_string();
                    }
                    KeyCode::Backspace => {
                        self.search_query.pop();
                    }
                    KeyCode::Char(c) => {
                        self.search_query.push(c);
                    }
                    _ => {}
                },
                _ => {}
            }
            return Ok(false);
        }

        match event {
            Event::Key(key) => {
                // Block problematic Cmd/Super key combinations that can interfere with terminal
                if key.modifiers.contains(KeyModifiers::SUPER) {
                    match key.code {
                        KeyCode::Char('t') | KeyCode::Char('n') | KeyCode::Char('w') => {
                            // Block Cmd+T (new tab), Cmd+N (new window), Cmd+W (close)
                            self.status_message = "Terminal shortcuts disabled in app".to_string();
                            return Ok(false);
                        }
                        _ => {}
                    }
                }

                // Handle Ctrl key combinations
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    || key.modifiers.contains(KeyModifiers::SUPER)
                {
                    match key.code {
                        KeyCode::Char('q') => return Ok(true),
                        KeyCode::Char('o') => {
                            // Use native file dialog on macOS
                            if let Some(path) = FileDialog::new()
                                .add_filter("PDF files", &["pdf"])
                                .add_filter("All files", &["*"])
                                .pick_file()
                            {
                                self.open_pdf(path)?;
                            } else {
                                self.status_message = "No file selected".to_string();
                            }
                        }
                        KeyCode::Char('e') => self.extract_matrix()?,
                        KeyCode::Char('s') => self.export_matrix()?,
                        KeyCode::Char('f') => {
                            self.search_input_active = true;
                            self.search_query.clear();
                            self.status_message = "Search: ".to_string();
                        }
                        KeyCode::Char('c') => {
                            if self.selection.start.is_some() {
                                self.copy_selection();
                            }
                        }
                        KeyCode::Char('x') => {
                            if self.selection.start.is_some() {
                                self.cut_selection();
                            }
                        }
                        KeyCode::Char('v') => {
                            self.paste_clipboard();
                            self.status_message = "Pasted from clipboard".to_string();
                        }
                        KeyCode::Char('p') => {
                            // Alternative paste using pbpaste command (macOS only)
                            if let Ok(output) = std::process::Command::new("pbpaste").output() {
                                if let Ok(text) = String::from_utf8(output.stdout) {
                                    self.paste_text_directly(text);
                                }
                            }
                        }
                        KeyCode::Char('h') => self.show_help = !self.show_help,
                        KeyCode::Char('l') => {
                            self.show_line_numbers = !self.show_line_numbers;
                            self.status_message = format!(
                                "Line numbers: {}",
                                if self.show_line_numbers { "ON" } else { "OFF" }
                            );
                        }
                        // Zoom controls with Ctrl modifier
                        KeyCode::Char('+') | KeyCode::Char('=') if self.pdf_path.is_some() => {
                            // Zoom in PDF - max 120% to prevent issues
                            let new_zoom = (self.zoom_level * 1.05).min(1.2);
                            if new_zoom != self.zoom_level {
                                self.zoom_level = new_zoom;
                                self.pdf_image = None; // Clear old image
                                                       // Re-render the page with new zoom level
                                if let Err(e) = self.render_current_page() {
                                    self.status_message = format!("Zoom failed: {}", e);
                                    self.zoom_level = 1.0; // Reset to safe default
                                    let _ = self.render_current_page(); // Try to render at safe zoom
                                } else {
                                    self.status_message =
                                        format!("Zoom: {:.0}%", self.zoom_level * 100.0);
                                }
                            } else {
                                self.status_message = "Maximum zoom reached (120%)".to_string();
                            }
                        }
                        KeyCode::Char('-') | KeyCode::Char('_') if self.pdf_path.is_some() => {
                            // Zoom out PDF - minimum 80% to prevent crashes
                            let new_zoom = (self.zoom_level / 1.05).max(0.8);
                            if new_zoom != self.zoom_level {
                                self.zoom_level = new_zoom;
                                self.pdf_image = None; // Clear old image
                                                       // Re-render the page with new zoom level
                                if let Err(e) = self.render_current_page() {
                                    self.status_message = format!("Zoom failed: {}", e);
                                    self.zoom_level = 1.0; // Reset to safe default
                                    let _ = self.render_current_page(); // Try to render at safe zoom
                                } else {
                                    self.status_message =
                                        format!("Zoom: {:.0}%", self.zoom_level * 100.0);
                                }
                            } else {
                                self.status_message = "Minimum zoom reached (80%)".to_string();
                            }
                        }
                        KeyCode::Char('0') if self.pdf_path.is_some() => {
                            // Reset zoom to safe default
                            self.zoom_level = 1.0; // 100% zoom
                            self.pdf_image = None; // Clear old image
                                                   // Re-render the page with new zoom level
                            if let Err(e) = self.render_current_page() {
                                self.status_message = format!("Zoom reset failed: {}", e);
                            } else {
                                self.status_message = "Zoom reset to 100%".to_string();
                            }
                        }
                        KeyCode::Char('[') => {
                            // Adjust split ratio left
                            self.split_ratio = self.split_ratio.saturating_sub(5).max(20);
                            self.status_message = format!("Split: {}%", self.split_ratio);
                        }
                        KeyCode::Char(']') => {
                            // Adjust split ratio right
                            self.split_ratio = (self.split_ratio + 5).min(80);
                            self.status_message = format!("Split: {}%", self.split_ratio);
                        }
                        _ => {}
                    }
                    return Ok(false);
                }

                // Handle Shift key for selection in raw matrix mode
                if key.modifiers.contains(KeyModifiers::SHIFT)
                    && self.text_view_mode == TextViewMode::RawMatrix
                {
                    let new_pos = match key.code {
                        KeyCode::Left => (self.cursor.0, self.cursor.1.saturating_sub(1)),
                        KeyCode::Right => {
                            if let Some(matrix) = &self.editable_matrix {
                                if self.cursor.0 < matrix.len() {
                                    (
                                        self.cursor.0,
                                        (self.cursor.1 + 1)
                                            .min(matrix[self.cursor.0].len().saturating_sub(1)),
                                    )
                                } else {
                                    self.cursor
                                }
                            } else {
                                self.cursor
                            }
                        }
                        KeyCode::Up => (self.cursor.0.saturating_sub(1), self.cursor.1),
                        KeyCode::Down => {
                            if let Some(matrix) = &self.editable_matrix {
                                (
                                    (self.cursor.0 + 1).min(matrix.len().saturating_sub(1)),
                                    self.cursor.1,
                                )
                            } else {
                                self.cursor
                            }
                        }
                        _ => self.cursor,
                    };

                    if new_pos != self.cursor {
                        if self.selection.start.is_none() {
                            self.selection.start = Some(self.cursor);
                        }
                        self.cursor = new_pos;
                        self.selection.end = Some(self.cursor);
                        self.is_selecting = true;
                    }
                    return Ok(false);
                }

                // Regular key handling
                match key.code {
                    KeyCode::Tab => {
                        // Toggle between raw matrix and smart layout views
                        self.text_view_mode = match self.text_view_mode {
                            TextViewMode::RawMatrix => {
                                // Extract smart layout if not already done
                                if self.smart_layout_text.is_none() && self.pdf_path.is_some() {
                                    self.extract_smart_layout()?;
                                }
                                TextViewMode::SmartLayout
                            }
                            TextViewMode::SmartLayout => TextViewMode::RawMatrix,
                        };
                        self.status_message = format!(
                            "Switched to {} view",
                            match self.text_view_mode {
                                TextViewMode::RawMatrix => "raw matrix",
                                TextViewMode::SmartLayout => "smart layout",
                            }
                        );
                    }
                    KeyCode::Esc => {
                        if self.is_selecting {
                            self.selection.clear();
                            self.is_selecting = false;
                            self.status_message = "Selection cleared".to_string();
                        }
                    }
                    // Arrow key navigation
                    KeyCode::Left => {
                        if self.text_view_mode == TextViewMode::RawMatrix {
                            self.cursor.1 = self.cursor.1.saturating_sub(1);
                            if !key.modifiers.contains(KeyModifiers::SHIFT) {
                                self.selection.clear();
                                self.is_selecting = false;
                            }
                        } else {
                            // In smart layout, go to previous page
                            if self.current_page > 0 {
                                self.current_page -= 1;
                                self.render_current_page()?;
                            }
                        }
                    }
                    KeyCode::Right => {
                        if self.text_view_mode == TextViewMode::RawMatrix {
                            if let Some(matrix) = &self.editable_matrix {
                                if self.cursor.0 < matrix.len() {
                                    self.cursor.1 = (self.cursor.1 + 1)
                                        .min(matrix[self.cursor.0].len().saturating_sub(1));
                                }
                            }
                            if !key.modifiers.contains(KeyModifiers::SHIFT) {
                                self.selection.clear();
                                self.is_selecting = false;
                            }
                        } else {
                            // In smart layout, go to next page
                            if self.current_page + 1 < self.total_pages {
                                self.current_page += 1;
                                self.render_current_page()?;
                            }
                        }
                    }
                    KeyCode::Up => {
                        self.cursor.0 = self.cursor.0.saturating_sub(1);
                        if !key.modifiers.contains(KeyModifiers::SHIFT) {
                            self.selection.clear();
                            self.is_selecting = false;
                        }
                    }
                    KeyCode::Down => {
                        if let Some(matrix) = &self.editable_matrix {
                            self.cursor.0 = (self.cursor.0 + 1).min(matrix.len().saturating_sub(1));
                        }
                        if !key.modifiers.contains(KeyModifiers::SHIFT) {
                            self.selection.clear();
                            self.is_selecting = false;
                        }
                    }
                    KeyCode::PageUp => {
                        self.current_page = self.current_page.saturating_sub(10);
                        self.render_current_page()?;
                    }
                    KeyCode::PageDown => {
                        self.current_page =
                            (self.current_page + 10).min(self.total_pages.saturating_sub(1));
                        self.render_current_page()?;
                    }
                    // Text input in matrix
                    KeyCode::Backspace if self.text_view_mode == TextViewMode::RawMatrix => {
                        if let Some(matrix) = &mut self.editable_matrix {
                            if self.cursor.1 > 0 {
                                self.cursor.1 -= 1;
                                if self.cursor.0 < matrix.len()
                                    && self.cursor.1 < matrix[self.cursor.0].len()
                                {
                                    matrix[self.cursor.0][self.cursor.1] = ' ';
                                    self.matrix_modified = true;
                                }
                            }
                        }
                    }
                    KeyCode::Enter if self.text_view_mode == TextViewMode::RawMatrix => {
                        if let Some(matrix) = &mut self.editable_matrix {
                            self.cursor.0 = (self.cursor.0 + 1).min(matrix.len().saturating_sub(1));
                            self.cursor.1 = 0;
                        }
                    }
                    KeyCode::Delete if self.text_view_mode == TextViewMode::RawMatrix => {
                        if let Some(matrix) = &mut self.editable_matrix {
                            if self.cursor.0 < matrix.len()
                                && self.cursor.1 < matrix[self.cursor.0].len()
                            {
                                matrix[self.cursor.0][self.cursor.1] = ' ';
                                self.matrix_modified = true;
                            }
                        }
                    }
                    KeyCode::Char('t')
                        if key.modifiers.is_empty()  // Only plain 't' key, no modifiers
                            && self.text_view_mode != TextViewMode::RawMatrix =>
                    {
                        // Toggle theme with 't' key (only when not editing matrix)
                        self.theme = match self.theme {
                            Theme::Dark => Theme::Light,
                            Theme::Light => Theme::Dark,
                        };
                        self.status_message = format!(
                            "{} mode enabled",
                            match self.theme {
                                Theme::Dark => "Dark",
                                Theme::Light => "Light",
                            }
                        );
                    }
                    KeyCode::Char('l')
                        if !key.modifiers.contains(KeyModifiers::CONTROL)
                            && !key.modifiers.contains(KeyModifiers::SUPER)
                            && self.text_view_mode != TextViewMode::RawMatrix =>
                    {
                        // Toggle line numbers with 'l' key (only when not editing matrix)
                        self.show_line_numbers = !self.show_line_numbers;
                        self.status_message = if self.show_line_numbers {
                            "Line numbers enabled".to_string()
                        } else {
                            "Line numbers disabled".to_string()
                        };
                    }
                    KeyCode::Char(c)
                        if self.text_view_mode == TextViewMode::RawMatrix
                            && !key.modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        // Type characters directly in matrix pane
                        if let Some(matrix) = &mut self.editable_matrix {
                            if self.cursor.0 < matrix.len() {
                                if self.cursor.1 >= matrix[self.cursor.0].len() {
                                    matrix[self.cursor.0].resize(self.cursor.1 + 1, ' ');
                                }
                                matrix[self.cursor.0][self.cursor.1] = c;
                                self.cursor.1 += 1;
                                self.matrix_modified = true;
                            }
                        }
                    }
                    KeyCode::F(3) => {
                        self.next_search_result();
                    }
                    KeyCode::F(2) => {
                        self.prev_search_result();
                    }
                    _ => {}
                }
            }
            Event::Mouse(mouse) => {
                match mouse.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        // Determine which pane was clicked based on split ratio
                        let term_width = crossterm::terminal::size()?.0;
                        let split_point = (term_width * self.split_ratio / 100) as u16;

                        if mouse.column >= split_point
                            && self.text_view_mode == TextViewMode::RawMatrix
                        {
                            // Calculate cursor position in matrix (fixing offset)
                            // Account for: header (5), border (1), and line numbers (5 if enabled)
                            if let Some(matrix) = &self.editable_matrix {
                                let line_num_offset = if self.show_line_numbers { 5 } else { 0 };
                                let col = (mouse
                                    .column
                                    .saturating_sub(split_point + 1 + line_num_offset))
                                    as usize;
                                let row = (mouse.row.saturating_sub(6)) as usize; // 5 for header + 1 for border

                                if row < matrix.len() && col < matrix[row].len() {
                                    self.cursor = (row, col);

                                    if mouse.modifiers.contains(KeyModifiers::SHIFT) {
                                        // Start selection
                                        if self.selection.start.is_none() {
                                            self.selection.start = Some(self.cursor);
                                        }
                                        self.selection.end = Some(self.cursor);
                                        self.is_selecting = true;
                                    } else {
                                        // Clear selection on regular click
                                        self.selection.clear();
                                        self.is_selecting = false;
                                    }
                                }
                            }
                        }
                    }
                    MouseEventKind::Drag(MouseButton::Left)
                        if self.text_view_mode == TextViewMode::RawMatrix =>
                    {
                        // Start or update selection
                        let term_width = crossterm::terminal::size()?.0;
                        let split_point = (term_width * self.split_ratio / 100) as u16;

                        if let Some(matrix) = &self.editable_matrix {
                            let line_num_offset = if self.show_line_numbers { 5 } else { 0 };
                            let col = (mouse
                                .column
                                .saturating_sub(split_point + 1 + line_num_offset))
                                as usize;
                            let row = (mouse.row.saturating_sub(6)) as usize; // 5 for header + 1 for border

                            if row < matrix.len() && col < matrix[row].len() {
                                if !self.is_selecting {
                                    self.selection.start = Some(self.cursor);
                                    self.is_selecting = true;
                                }
                                self.cursor = (row, col);
                                self.selection.end = Some(self.cursor);
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        Ok(false)
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        // Safety check: Don't render if area is too small
        if area.width < 10 || area.height < 5 {
            return;
        }
        
        let colors = self.theme.colors();

        // Fill background with theme color - with bounds checking
        let buf_area = buf.area();
        let buf_width = buf_area.width;
        let buf_height = buf_area.height;
        
        // Another safety check for buffer dimensions
        if buf_width == 0 || buf_height == 0 {
            return;
        }

        let max_row = area.y.saturating_add(area.height).min(buf_height);
        let max_col = area.x.saturating_add(area.width).min(buf_width);

        // Ensure we never go out of bounds
        if area.y < buf_height && area.x < buf_width {
            for row in area.y..max_row {
                for col in area.x..max_col {
                    // The ranges are already clamped but double-check
                    if col < buf_width && row < buf_height {
                        buf[(col, row)].set_style(Style::default().bg(colors.bg));
                    }
                }
            }
        }

        // Main layout with header
        let main_chunks = Layout::vertical([
            Constraint::Length(5), // Header with commands
            Constraint::Min(1),    // Content area
            Constraint::Length(1), // Status bar
        ])
        .split(area);

        // Render header with commands
        self.render_header(main_chunks[0], buf);

        // Always two panes: PDF on left, text view on right
        let content_chunks = Layout::horizontal([
            Constraint::Percentage(self.split_ratio),
            Constraint::Percentage(100 - self.split_ratio),
        ])
        .split(main_chunks[1]);

        // Render PDF pane
        self.render_pdf_pane(content_chunks[0], buf);

        // Render text view based on mode
        match self.text_view_mode {
            TextViewMode::RawMatrix => self.render_matrix_pane(content_chunks[1], buf),
            TextViewMode::SmartLayout => self.render_smart_layout_pane(content_chunks[1], buf),
        }

        // Render status bar
        self.render_status_bar(main_chunks[2], buf);

        // Render help overlay if active
        if self.show_help {
            self.render_help_overlay(area, buf);
        }
    }

    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        let colors = self.theme.colors();

        let header_block = Block::default()
            .borders(Borders::ALL)
            .title(" 🐹 CHONKER5 TUI ")
            .border_style(Style::default().fg(colors.teal));

        let inner = header_block.inner(area);
        header_block.render(area, buf);

        let commands = vec![
            "Cmd+O: Open PDF | Cmd+E: Extract Text | Tab: Toggle View | +/-: Zoom",
            "Cmd+C: Copy | Cmd+V: Paste | Cmd+X: Cut | Cmd+S: Save",
            "↑↓←→: Navigate | Shift+↑↓←→: Select | T: Theme (SmartLayout) | L: Line Numbers",
        ];

        for (i, cmd) in commands.iter().enumerate() {
            if i < inner.height as usize {
                let paragraph = Paragraph::new(*cmd).style(Style::default().fg(colors.fg));
                let cmd_area = Rect {
                    x: inner.x,
                    y: inner.y + i as u16,
                    width: inner.width,
                    height: 1,
                };
                paragraph.render(cmd_area, buf);
            }
        }
    }

    fn render_pdf_pane(&mut self, area: Rect, buf: &mut Buffer) {
        let colors = self.theme.colors();
        let border_style = Style::default().fg(colors.teal);

        let pdf_block = Block::default()
            .borders(Borders::ALL)
            .title(format!(
                " PDF Viewer - Page {}/{} ",
                self.current_page + 1,
                self.total_pages.max(1)
            ))
            .border_style(border_style);

        let inner = pdf_block.inner(area);
        pdf_block.render(area, buf);
        
        // Safety check: Don't render if inner area is too small
        if inner.width < 2 || inner.height < 2 {
            return;
        }

        // Try to render PDF as image if available
        if let Some(pdf_image) = &self.pdf_image {
            // Skip rendering if image is too small to prevent crashes
            let img_width = pdf_image.width();
            let img_height = pdf_image.height();

            if img_width >= 50 && img_height >= 50 {
                // Always recreate the protocol after zoom changes to ensure correct rendering
                // The old protocol holds a reference to the old image size
                if let Some(ref mut picker) = self.image_picker {
                    // Extra safety: ensure inner area is valid for image rendering
                    if inner.width > 0 && inner.height > 0 {
                        // Create a fresh protocol for the current zoomed image
                        let mut protocol = picker.new_resize_protocol(pdf_image.clone());

                        // Create the image widget
                        let image_widget = StatefulImage::new(None);

                        // Render the image widget with the fresh protocol
                        image_widget.render(inner, buf, &mut protocol);
                    }

                    // Don't store the protocol as we'll recreate it each frame
                    // This prevents stale image references after zoom
                }
            } else {
                // Show message when image is too small
                let info_text = format!(
                    "PDF Page {}/{}\nZoom: {:.0}%\nImage too small to render ({}x{})\n\nZoom in with Ctrl+ to view",
                    self.current_page + 1,
                    self.total_pages,
                    self.zoom_level * 100.0,
                    img_width,
                    img_height
                );
                let paragraph = Paragraph::new(info_text).style(Style::default().fg(colors.fg));
                paragraph.render(inner, buf);
            }
        } else if let Some(content) = &self.pdf_render_cache {
            // Fallback to text representation
            let paragraph = Paragraph::new(content.as_str())
                .style(Style::default().fg(colors.fg))
                .scroll(self.pdf_scroll);
            paragraph.render(inner, buf);
        } else {
            let paragraph = Paragraph::new("No PDF loaded\n\nPress 'o' to open a PDF file")
                .style(Style::default().fg(colors.dim));
            paragraph.render(inner, buf);
        }
    }

    fn render_matrix_pane(&mut self, area: Rect, buf: &mut Buffer) {
        let colors = self.theme.colors();
        let buf_width = buf.area().width;
        let buf_height = buf.area().height;
        let border_style = Style::default().fg(colors.teal);

        let title = if self.matrix_modified {
            " Character Matrix [Modified] "
        } else {
            " Character Matrix "
        };

        let matrix_block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style);

        let inner = matrix_block.inner(area);
        matrix_block.render(area, buf);

        // Safety check: Don't render if inner area is too small
        if inner.width < 1 || inner.height < 1 {
            return;
        }

        // Draw dot matrix background with bounds checking
        for row in 0..inner.height {
            for col in 0..inner.width {
                let x = inner.x + col;
                let y = inner.y + row;
                // Check bounds before accessing buffer
                if x < buf_width && y < buf_height {
                    buf[(x, y)]
                        .set_char('·')
                        .set_style(Style::default().fg(colors.dim));
                }
            }
        }

        if let Some(matrix) = &self.editable_matrix {
            // Render matrix with line numbers and selection
            for (row_idx, row) in matrix.iter().enumerate() {
                if row_idx >= inner.height as usize {
                    break;
                }

                let mut line = String::new();
                let mut line_styles = Vec::new();

                // Add line number if enabled
                if self.show_line_numbers {
                    let line_num = format!("{:4} ", row_idx + 1);
                    line.push_str(&line_num);
                    line_styles.push((line_num.len(), Style::default().fg(colors.dim)));
                }

                // Add matrix content
                for (col_idx, &ch) in row.iter().enumerate() {
                    if col_idx
                        >= (inner.width as usize - if self.show_line_numbers { 5 } else { 0 })
                    {
                        break;
                    }

                    line.push(ch);

                    // Apply selection highlighting
                    let style = if self.selection.is_selected(row_idx, col_idx) {
                        Style::default().bg(colors.highlight).fg(Color::Black)
                    } else if row_idx == self.cursor.0
                        && col_idx == self.cursor.1
                        && self.cursor_blink_state
                    {
                        Style::default().bg(colors.teal).fg(Color::Black)
                    } else if self.search_results.contains(&(row_idx, col_idx)) {
                        Style::default().bg(colors.yellow).fg(Color::Black)
                    } else {
                        Style::default().fg(colors.fg)
                    };

                    line_styles.push((1, style));
                }

                // Render the line
                let y = inner.y + row_idx as u16;
                let x = inner.x;

                let mut current_x = x;
                let mut char_iter = line.chars();
                for (len, style) in line_styles {
                    for _ in 0..len {
                        if let Some(ch) = char_iter.next() {
                            // Check bounds before writing to buffer
                            if current_x < buf_width && y < buf_height {
                                let _ = &mut buf[(current_x, y)].set_char(ch).set_style(style);
                            }
                            current_x += 1;
                        }
                    }
                }
            }
        } else {
            let paragraph = Paragraph::new(
                "No matrix extracted\n\nPress Ctrl+M to extract matrix from current PDF page",
            )
            .style(Style::default().fg(colors.dim));
            paragraph.render(inner, buf);
        }
    }

    fn render_status_bar(&self, area: Rect, buf: &mut Buffer) {
        let colors = self.theme.colors();
        let pos_str = format!(" {}:{} ", self.cursor.0 + 1, self.cursor.1 + 1);

        let status_content = if self.file_input_active {
            format!("Enter path: {}", self.file_input_buffer)
        } else if self.search_input_active {
            format!("Search: {}", self.search_query)
        } else {
            self.status_message.clone()
        };

        let help_hint = " Ctrl+H: Help ";

        let status_line =
            format!(
                " {} | {}{}{}",
                status_content,
                pos_str,
                help_hint,
                " ".repeat(area.width.saturating_sub(
                    (status_content.len() + pos_str.len() + help_hint.len() + 4) as u16
                ) as usize)
            );

        let paragraph =
            Paragraph::new(status_line).style(Style::default().bg(colors.chrome).fg(colors.fg));
        paragraph.render(area, buf);
    }

    fn render_smart_layout_pane(&self, area: Rect, buf: &mut Buffer) {
        let colors = self.theme.colors();
        let buf_width = buf.area().width;
        let buf_height = buf.area().height;
        let border_style = Style::default().fg(colors.teal);

        let smart_block = Block::default()
            .borders(Borders::ALL)
            .title(" Smart Layout ")
            .border_style(border_style);

        let inner = smart_block.inner(area);
        smart_block.render(area, buf);

        // Safety check: Don't render if inner area is too small
        if inner.width < 1 || inner.height < 1 {
            return;
        }

        // Draw dot matrix background with bounds checking
        for row in 0..inner.height {
            for col in 0..inner.width {
                let x = inner.x + col;
                let y = inner.y + row;
                // Check bounds before accessing buffer
                if x < buf_width && y < buf_height {
                    buf[(x, y)]
                        .set_char('·')
                        .set_style(Style::default().fg(colors.dim));
                }
            }
        }

        if let Some(layout_text) = &self.smart_layout_text {
            let paragraph = Paragraph::new(layout_text.as_str())
                .style(Style::default().fg(colors.fg))
                .scroll((self.smart_layout_scroll, 0));
            paragraph.render(inner, buf);
        } else {
            let paragraph = Paragraph::new(
                "Smart layout extraction not available\n\nPress 's' to extract smart layout",
            )
            .style(Style::default().fg(colors.dim));
            paragraph.render(inner, buf);
        }
    }

    fn render_help_overlay(&self, area: Rect, buf: &mut Buffer) {
        let help_text = r#"
╭─────────────── Chonker5 TUI Help ───────────────╮
│                                                  │
│ Navigation:                                      │
│   Tab           Switch between panes            │
│   Arrow Keys    Navigate (PDF pages or cursor)  │
│   PageUp/Down   Jump 10 pages (PDF pane)        │
│                                                  │
│ File Operations:                                │
│   Ctrl+O        Open PDF file                   │
│   Ctrl+M        Extract character matrix        │
│   Ctrl+E        Export matrix to file           │
│   Ctrl+Q        Quit application                │
│                                                  │
│ Editing (Matrix Pane):                          │
│   Type          Insert characters directly      │
│   Backspace     Delete character and move left  │
│   Delete        Delete character at cursor      │
│   Enter         Move to next line               │
│                                                  │
│ Selection & Clipboard:                          │
│   Shift+Arrows  Select text                     │
│   Mouse Drag    Select with mouse               │
│   Ctrl+C        Copy selection                  │
│   Ctrl+X        Cut selection                   │
│   Ctrl+V        Paste clipboard                 │
│   Esc           Clear selection                 │
│                                                  │
│ Search:                                          │
│   Ctrl+F        Search in matrix                │
│   F3            Next search result              │
│   F2            Previous search result          │
│                                                  │
│ Display:                                         │
│   Ctrl+L        Toggle line numbers             │
│   Ctrl +/-      Adjust split ratio              │
│   Ctrl+H        Toggle this help                │
│                                                  │
│ Mouse Support:                                   │
│   Click         Set cursor position             │
│   Shift+Click   Start selection                 │
│   Drag          Select text region              │
│                                                  │
│ Press Ctrl+H to close help                      │
╰──────────────────────────────────────────────────╯
"#;

        // Calculate centered position
        let help_width = 52;
        let help_height = 44;
        let x = (area.width.saturating_sub(help_width)) / 2;
        let y = (area.height.saturating_sub(help_height)) / 2;

        let help_area = Rect {
            x,
            y,
            width: help_width.min(area.width),
            height: help_height.min(area.height),
        };

        // Clear background with bounds checking
        let buf_width = buf.area().width;
        let buf_height = buf.area().height;
        let help_end_row = help_area.y.saturating_add(help_area.height).min(buf_height);
        let help_end_col = help_area.x.saturating_add(help_area.width).min(buf_width);

        for row in help_area.y..help_end_row {
            for col in help_area.x..help_end_col {
                if col < buf_width && row < buf_height {
                    buf[(col, row)]
                        .set_char(' ')
                        .set_style(Style::default().bg(Color::Rgb(10, 15, 20)));
                }
            }
        }

        // Render help text
        let paragraph = Paragraph::new(help_text)
            .style(
                Style::default()
                    .fg(Color::Rgb(26, 188, 156))
                    .bg(Color::Rgb(10, 15, 20)),
            )
            .alignment(Alignment::Left);
        paragraph.render(help_area, buf);
    }
}

// ============= MAIN =============
fn main() -> Result<()> {
    // Terminal setup
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // App state
    let mut app = ChonkerTUI::new();

    // Main loop
    let mut should_quit = false;
    while !should_quit {
        // Draw
        terminal.draw(|f| {
            app.render(f.area(), f.buffer_mut());
        })?;

        // Handle events with short timeout for responsive UI
        if event::poll(Duration::from_millis(50))? {
            should_quit = app.handle_event(event::read()?)?;
        }
    }

    // Cleanup
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;

    // Print summary
    if app.matrix_modified {
        println!("\nMatrix was modified but not saved.");
        println!("Use Ctrl+E to export changes next time.");
    }

    Ok(())
}
