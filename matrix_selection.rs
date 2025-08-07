use egui::{Color32, Rect, Response, Sense, Vec2};
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct MatrixSelection {
    pub start: Option<(usize, usize)>, // (row, col)
    pub end: Option<(usize, usize)>,   // (row, col)
}

impl MatrixSelection {
    pub fn new() -> Self {
        Self {
            start: None,
            end: None,
        }
    }

    pub fn clear(&mut self) {
        self.start = None;
        self.end = None;
    }

    pub fn is_selected(&self, row: usize, col: usize) -> bool {
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

    pub fn get_selected_text(&self, matrix: &[Vec<char>]) -> String {
        if let (Some(start), Some(end)) = (self.start, self.end) {
            let min_row = start.0.min(end.0);
            let max_row = start.0.max(end.0);
            let min_col = start.1.min(end.1);
            let max_col = start.1.max(end.1);

            let mut result = String::new();
            for row in min_row..=max_row {
                if row < matrix.len() {
                    for col in min_col..=max_col {
                        if col < matrix[row].len() {
                            result.push(matrix[row][col]);
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
}

pub struct MatrixGrid {
    pub matrix: Vec<Vec<char>>,
    pub selection: MatrixSelection,
    pub char_size: Vec2,
    pub cursor_pos: Option<(usize, usize)>, // Current cursor position
    pub last_blink: Instant,
    pub cursor_visible: bool,
}

impl MatrixGrid {
    pub fn new(text: &str) -> Self {
        let matrix: Vec<Vec<char>> = text
            .lines()
            .map(|line| {
                // Skip the row number prefix (e.g., "  0 ")
                if let Some(pos) = line.find(' ') {
                    line[pos + 1..].chars().collect()
                } else {
                    line.chars().collect()
                }
            })
            .collect();

        Self {
            matrix,
            selection: MatrixSelection::new(),
            char_size: Vec2::new(6.0, 10.0), // Slightly wider for character spacing
            cursor_pos: None,
            last_blink: Instant::now(),
            cursor_visible: true,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Response {
        // Teal theme colors - matching the terminal theme
        const TERM_TEAL: Color32 = Color32::from_rgb(26, 188, 156);
        const TERM_TEAL_FADED: Color32 = Color32::from_rgba_premultiplied(26, 188, 156, 80);
        const TERM_BG: Color32 = Color32::from_rgb(10, 15, 20);
        const TERM_FG: Color32 = Color32::from_rgb(200, 200, 200);

        // Note: We're custom drawing, so no text input to spell check

        let (response, painter) = ui.allocate_painter(
            Vec2::new(
                self.matrix.get(0).map_or(0.0, |row| row.len() as f32) * self.char_size.x,
                self.matrix.len() as f32 * self.char_size.y,
            ),
            Sense::click_and_drag(),
        );

        let rect = response.rect;
        let font_id = egui::FontId::monospace(9.0); // Smaller font to match smaller grid

        // Update cursor blink
        let now = Instant::now();
        if now.duration_since(self.last_blink).as_millis() > 530 {
            self.cursor_visible = !self.cursor_visible;
            self.last_blink = now;
            ui.ctx().request_repaint(); // Keep animating
        }

        // Handle mouse click for cursor position
        if response.clicked() {
            if let Some(pos) = response.hover_pos() {
                let local_pos = pos - rect.min;
                let row = (local_pos.y / self.char_size.y) as usize;
                let col = (local_pos.x / self.char_size.x) as usize;
                if row < self.matrix.len() && col < self.matrix.get(row).map_or(0, |r| r.len()) {
                    self.cursor_pos = Some((row, col));
                    self.cursor_visible = true; // Reset blink on move
                    self.last_blink = Instant::now();
                }
            }
        }

        // Handle selection
        if response.drag_started() {
            if let Some(pos) = response.hover_pos() {
                let local_pos = pos - rect.min;
                let row = (local_pos.y / self.char_size.y) as usize;
                let col = (local_pos.x / self.char_size.x) as usize;
                self.selection.start = Some((row, col));
                self.selection.end = Some((row, col));
                self.cursor_pos = None; // Hide cursor during selection
            }
        }

        if response.dragged() {
            if let Some(pos) = response.hover_pos() {
                let local_pos = pos - rect.min;
                let row = (local_pos.y / self.char_size.y) as usize;
                let col = (local_pos.x / self.char_size.x) as usize;
                self.selection.end = Some((row, col));
            }
        }

        // Draw background first
        painter.rect_filled(rect, 0.0, TERM_BG);

        // Draw matrix with selection
        for (row_idx, row) in self.matrix.iter().enumerate() {
            for (col_idx, &ch) in row.iter().enumerate() {
                let pos = rect.min
                    + Vec2::new(
                        col_idx as f32 * self.char_size.x,
                        row_idx as f32 * self.char_size.y,
                    );

                // Highlight if selected with teal color and slightly taller box
                if self.selection.is_selected(row_idx, col_idx) {
                    let selection_rect = Rect::from_min_size(
                        pos - Vec2::new(0.0, self.char_size.y * 0.1), // Slightly higher
                        Vec2::new(self.char_size.x, self.char_size.y * 1.2), // Slightly taller
                    );
                    painter.rect_filled(
                        selection_rect,
                        2.0, // Slight rounding for that retro feel
                        TERM_TEAL_FADED,
                    );
                }

                // Draw character
                let char_color = if self.selection.is_selected(row_idx, col_idx) {
                    Color32::BLACK // Black text on teal selection for contrast
                } else if ch == '·' {
                    Color32::from_gray(80) // Dimmer for space dots
                } else {
                    TERM_FG
                };

                // Draw each character individually with slight spacing to prevent spell check
                painter.text(
                    pos + Vec2::new(self.char_size.x * 0.45, self.char_size.y * 0.5),
                    egui::Align2::CENTER_CENTER,
                    ch.to_string(),
                    font_id.clone(),
                    char_color,
                );
            }
        }

        // Draw blinking cursor if visible
        if let Some((cursor_row, cursor_col)) = self.cursor_pos {
            if self.cursor_visible && cursor_row < self.matrix.len() {
                let cursor_pos = rect.min
                    + Vec2::new(
                        cursor_col as f32 * self.char_size.x,
                        cursor_row as f32 * self.char_size.y,
                    );

                // Old-school block cursor
                painter.rect_filled(
                    Rect::from_min_size(
                        cursor_pos - Vec2::new(0.0, self.char_size.y * 0.1),
                        Vec2::new(self.char_size.x * 0.8, self.char_size.y * 1.2),
                    ),
                    0.0,
                    TERM_TEAL,
                );

                // Draw character in inverted color if cursor is over it
                if cursor_col < self.matrix[cursor_row].len() {
                    let ch = self.matrix[cursor_row][cursor_col];
                    painter.text(
                        cursor_pos + Vec2::new(self.char_size.x * 0.5, self.char_size.y * 0.5),
                        egui::Align2::CENTER_CENTER,
                        ch.to_string(),
                        font_id.clone(),
                        TERM_BG, // Inverted color
                    );
                }
            }
        }

        // Handle copy on Ctrl+C
        if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::C)) {
            let selected_text = self.selection.get_selected_text(&self.matrix);
            if !selected_text.is_empty() {
                ui.output_mut(|o| o.copied_text = selected_text);
            }
        }

        response
    }
}
