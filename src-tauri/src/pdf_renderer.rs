use std::ffi::CString;
use std::ptr;
use anyhow::{Result, anyhow};
use base64::Engine;

pub struct PdfRenderer {
    context: *mut mupdf_sys::fz_context,
}

impl PdfRenderer {
    pub fn new() -> Result<Self> {
        unsafe {
            // MuPDF context creation might be different in this version
            let context = mupdf_sys::fz_new_context_imp(
                ptr::null(),
                ptr::null(),
                mupdf_sys::FZ_STORE_UNLIMITED as usize,
                mupdf_sys::FZ_VERSION.as_ptr() as *const i8
            );
            if context.is_null() {
                return Err(anyhow!("Failed to create MuPDF context"));
            }
            
            Ok(PdfRenderer { context })
        }
    }
    
    pub fn render_page_to_base64(&self, pdf_path: &str, page_num: i32, zoom: f32) -> Result<String> {
        unsafe {
            // Open document
            let path_c = CString::new(pdf_path)?;
            let doc = mupdf_sys::fz_open_document(self.context, path_c.as_ptr());
            if doc.is_null() {
                return Err(anyhow!("Failed to open PDF document"));
            }
            
            // Get page count
            let page_count = mupdf_sys::fz_count_pages(self.context, doc);
            if page_num >= page_count {
                mupdf_sys::fz_drop_document(self.context, doc);
                return Err(anyhow!("Page number {} exceeds document page count {}", page_num, page_count));
            }
            
            // Load page
            let page = mupdf_sys::fz_load_page(self.context, doc, page_num);
            if page.is_null() {
                mupdf_sys::fz_drop_document(self.context, doc);
                return Err(anyhow!("Failed to load page {}", page_num));
            }
            
            // Get page bounds
            let page_bounds = mupdf_sys::fz_bound_page(self.context, page);
            
            // Calculate matrix for zoom
            let matrix = mupdf_sys::fz_matrix {
                a: zoom, b: 0.0, c: 0.0, d: zoom, e: 0.0, f: 0.0
            };
            
            // Transform bounds
            let bounds = mupdf_sys::fz_transform_rect(page_bounds, matrix);
            
            // Create pixmap
            let colorspace = mupdf_sys::fz_device_rgb(self.context);
            let bbox = mupdf_sys::fz_irect {
                x0: bounds.x0 as i32,
                y0: bounds.y0 as i32,
                x1: bounds.x1 as i32,
                y1: bounds.y1 as i32,
            };
            let pixmap = mupdf_sys::fz_new_pixmap_with_bbox(
                self.context,
                colorspace,
                bbox,
                ptr::null_mut(),
                1
            );
            
            if pixmap.is_null() {
                mupdf_sys::fz_drop_page(self.context, page);
                mupdf_sys::fz_drop_document(self.context, doc);
                return Err(anyhow!("Failed to create pixmap"));
            }
            
            // Clear pixmap to white
            mupdf_sys::fz_clear_pixmap_with_value(self.context, pixmap, 0xff);
            
            // Create device
            let device = mupdf_sys::fz_new_draw_device(self.context, matrix, pixmap);
            if device.is_null() {
                mupdf_sys::fz_drop_pixmap(self.context, pixmap);
                mupdf_sys::fz_drop_page(self.context, page);
                mupdf_sys::fz_drop_document(self.context, doc);
                return Err(anyhow!("Failed to create draw device"));
            }
            
            // Render page
            mupdf_sys::fz_run_page(self.context, page, device, matrix, ptr::null_mut());
            mupdf_sys::fz_close_device(self.context, device);
            mupdf_sys::fz_drop_device(self.context, device);
            
            // Get raw pixmap data and convert to PNG manually
            let samples = mupdf_sys::fz_pixmap_samples(self.context, pixmap);
            let width = mupdf_sys::fz_pixmap_width(self.context, pixmap) as u32;
            let height = mupdf_sys::fz_pixmap_height(self.context, pixmap) as u32;
            let stride = mupdf_sys::fz_pixmap_stride(self.context, pixmap) as usize;
            let n = mupdf_sys::fz_pixmap_components(self.context, pixmap) as usize;
            
            // Create RGB buffer
            let data_size = (width * height * 3) as usize;
            let mut rgb_data = Vec::with_capacity(data_size);
            
            // Convert RGBA to RGB if needed
            let raw_data = std::slice::from_raw_parts(samples, (height as usize) * stride);
            
            for y in 0..height {
                for x in 0..width {
                    let src_idx = (y as usize * stride + x as usize * n) as usize;
                    if src_idx + 2 < raw_data.len() {
                        rgb_data.push(raw_data[src_idx]);     // R
                        rgb_data.push(raw_data[src_idx + 1]); // G 
                        rgb_data.push(raw_data[src_idx + 2]); // B
                    }
                }
            }
            
            // Encode as base64 PNG (simplified - just encode raw RGB for now)
            let base64_data = base64::engine::general_purpose::STANDARD.encode(&rgb_data);
            mupdf_sys::fz_drop_pixmap(self.context, pixmap);
            mupdf_sys::fz_drop_page(self.context, page);
            mupdf_sys::fz_drop_document(self.context, doc);
            
            Ok(base64_data)
        }
    }
    
    pub fn get_page_count(&self, pdf_path: &str) -> Result<i32> {
        unsafe {
            let path_c = CString::new(pdf_path)?;
            let doc = mupdf_sys::fz_open_document(self.context, path_c.as_ptr());
            if doc.is_null() {
                return Err(anyhow!("Failed to open PDF document"));
            }
            
            let page_count = mupdf_sys::fz_count_pages(self.context, doc);
            mupdf_sys::fz_drop_document(self.context, doc);
            
            Ok(page_count)
        }
    }
}

impl Drop for PdfRenderer {
    fn drop(&mut self) {
        unsafe {
            if !self.context.is_null() {
                mupdf_sys::fz_drop_context(self.context);
            }
        }
    }
}

unsafe impl Send for PdfRenderer {}
unsafe impl Sync for PdfRenderer {}
