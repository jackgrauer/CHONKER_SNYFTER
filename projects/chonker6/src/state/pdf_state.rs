use crate::actions::{PdfMetadata, PageDirection};

#[derive(Debug, Clone)]
pub struct PdfState {
    pub metadata: Option<PdfMetadata>,
    pub current_page: usize,
    pub page_count: usize,
    pub zoom_level: f32,
    pub auto_fit: bool,
    pub dark_mode: bool,
    pub image_data: Option<Vec<u8>>, // For terminal image rendering
    pub image_width: u32,
    pub image_height: u32,
}

impl Default for PdfState {
    fn default() -> Self {
        Self {
            metadata: None,
            current_page: 0,
            page_count: 0,
            zoom_level: 1.0,
            auto_fit: true,
            dark_mode: false,
            image_data: None,
            image_width: 0,
            image_height: 0,
        }
    }
}

impl PdfState {
    pub fn is_loaded(&self) -> bool {
        self.metadata.is_some()
    }
    
    pub fn load(&mut self, metadata: PdfMetadata) {
        self.page_count = metadata.page_count;
        self.metadata = Some(metadata);
        self.current_page = 0;
    }
    
    pub fn navigate(&mut self, direction: PageDirection) {
        if self.page_count == 0 {
            return;
        }
        
        match direction {
            PageDirection::Next => {
                if self.current_page < self.page_count - 1 {
                    self.current_page += 1;
                }
            }
            PageDirection::Previous => {
                if self.current_page > 0 {
                    self.current_page -= 1;
                }
            }
        }
    }
    
    pub fn zoom_in(&mut self) -> bool {
        if !self.auto_fit {
            let new_zoom = (self.zoom_level * 1.05).min(1.2);
            if (new_zoom - self.zoom_level).abs() > 0.001 && new_zoom <= 1.2 {
                self.zoom_level = (new_zoom * 100.0).round() / 100.0;
                if self.zoom_level > 1.2 {
                    self.zoom_level = 1.2;
                }
                self.image_data = None; // Clear cached image
                return true;
            }
        }
        false
    }
    
    pub fn zoom_out(&mut self) -> bool {
        if !self.auto_fit {
            let new_zoom = (self.zoom_level / 1.05).max(0.9);
            if (new_zoom - self.zoom_level).abs() > 0.001 && new_zoom >= 0.9 {
                self.zoom_level = (new_zoom * 100.0).round() / 100.0;
                if self.zoom_level < 0.9 {
                    self.zoom_level = 0.9;
                }
                self.image_data = None; // Clear cached image
                return true;
            }
        }
        false
    }
    
    pub fn zoom_reset(&mut self) {
        if !self.auto_fit {
            self.zoom_level = 1.0;
            self.image_data = None; // Clear cached image
        }
    }
    
    pub fn toggle_auto_fit(&mut self) {
        self.auto_fit = !self.auto_fit;
        self.image_data = None; // Clear cached image
    }
    
    pub fn toggle_dark_mode(&mut self) {
        self.dark_mode = !self.dark_mode;
        self.image_data = None; // Clear cached image
    }
    
    pub fn set_image_data(&mut self, data: Vec<u8>, width: u32, height: u32) {
        self.image_data = Some(data);
        self.image_width = width;
        self.image_height = height;
    }
    
    pub fn clear_image_data(&mut self) {
        self.image_data = None;
        self.image_width = 0;
        self.image_height = 0;
    }
}