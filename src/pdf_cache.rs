use anyhow::Result;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

// ============= PDF CACHE SYSTEM =============

#[derive(Clone)]
pub struct PdfPageData {
    pub page_num: usize,
    pub rendered_text: String,
    pub render_time: Instant,
    pub dpi: u16,
}

pub struct PdfCache {
    // Current page cache
    pub current: Option<PdfPageData>,

    // Adjacent pages pre-rendered
    pub next: Option<PdfPageData>,
    pub prev: Option<PdfPageData>,

    // Background rendering thread
    background_handle: Option<JoinHandle<()>>,

    // LRU cache for all rendered pages
    rendered_pages: Arc<Mutex<LruCache>>,

    // PDF path for rendering
    pdf_path: Option<PathBuf>,
}

struct LruCache {
    pages: HashMap<usize, PdfPageData>,
    access_order: VecDeque<usize>,
    max_size: usize,
}

impl LruCache {
    fn new(max_size: usize) -> Self {
        Self {
            pages: HashMap::new(),
            access_order: VecDeque::new(),
            max_size,
        }
    }

    fn get(&mut self, page: usize) -> Option<PdfPageData> {
        if let Some(data) = self.pages.get(&page) {
            // Move to front (most recently used)
            self.access_order.retain(|&p| p != page);
            self.access_order.push_front(page);
            Some(data.clone())
        } else {
            None
        }
    }

    fn insert(&mut self, page: usize, data: PdfPageData) {
        // Remove if already exists
        self.access_order.retain(|&p| p != page);

        // Add to front
        self.access_order.push_front(page);
        self.pages.insert(page, data);

        // Evict LRU if over capacity
        while self.pages.len() > self.max_size {
            if let Some(lru_page) = self.access_order.pop_back() {
                self.pages.remove(&lru_page);
            }
        }
    }
}

impl PdfCache {
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            current: None,
            next: None,
            prev: None,
            background_handle: None,
            rendered_pages: Arc::new(Mutex::new(LruCache::new(max_cache_size))),
            pdf_path: None,
        }
    }

    pub fn set_pdf_path(&mut self, path: PathBuf) {
        self.pdf_path = Some(path);
    }

    pub fn change_page(&mut self, new_page: usize, total_pages: usize) -> Result<Option<String>> {
        // Stop any existing background render
        if let Some(handle) = self.background_handle.take() {
            // Don't wait, just detach
            drop(handle);
        }

        // Check if we have it pre-rendered
        let cached_data = if new_page
            == self
                .current
                .as_ref()
                .map(|d| d.page_num + 1)
                .unwrap_or(usize::MAX)
        {
            self.next.take()
        } else if new_page
            == self
                .current
                .as_ref()
                .map(|d| d.page_num.saturating_sub(1))
                .unwrap_or(usize::MAX)
        {
            self.prev.take()
        } else {
            // Check LRU cache
            self.rendered_pages.lock().unwrap().get(new_page)
        };

        if let Some(data) = cached_data {
            self.current = Some(data.clone());

            // Start pre-rendering adjacent pages
            self.prerender_adjacent_pages(new_page, total_pages);

            return Ok(Some(data.rendered_text));
        }

        // Need to render current page
        if let Some(pdf_path) = &self.pdf_path {
            let rendered = self.render_page_sync(pdf_path, new_page)?;
            let data = PdfPageData {
                page_num: new_page,
                rendered_text: rendered.clone(),
                render_time: Instant::now(),
                dpi: self.calculate_optimal_dpi(),
            };

            self.current = Some(data.clone());
            self.rendered_pages.lock().unwrap().insert(new_page, data);

            // Start pre-rendering adjacent pages
            self.prerender_adjacent_pages(new_page, total_pages);

            Ok(Some(rendered))
        } else {
            Ok(None)
        }
    }

    fn prerender_adjacent_pages(&mut self, current_page: usize, total_pages: usize) {
        let pdf_path = match &self.pdf_path {
            Some(p) => p.clone(),
            None => return,
        };

        let cache = Arc::clone(&self.rendered_pages);

        // Clone current adjacent pages to check what needs rendering
        let need_prev =
            current_page > 0 && self.prev.as_ref().map(|d| d.page_num) != Some(current_page - 1);
        let need_next = current_page + 1 < total_pages
            && self.next.as_ref().map(|d| d.page_num) != Some(current_page + 1);

        if !need_prev && !need_next {
            return; // Nothing to pre-render
        }

        let handle = thread::spawn(move || {
            // Render previous page
            if need_prev && current_page > 0 {
                let prev_page = current_page - 1;
                if let Ok(rendered) = Self::render_page_static(&pdf_path, prev_page) {
                    let data = PdfPageData {
                        page_num: prev_page,
                        rendered_text: rendered,
                        render_time: Instant::now(),
                        dpi: Self::calculate_optimal_dpi_static(),
                    };
                    cache.lock().unwrap().insert(prev_page, data);
                }
            }

            // Small delay to not overwhelm
            thread::sleep(Duration::from_millis(50));

            // Render next page
            if need_next && current_page + 1 < total_pages {
                let next_page = current_page + 1;
                if let Ok(rendered) = Self::render_page_static(&pdf_path, next_page) {
                    let data = PdfPageData {
                        page_num: next_page,
                        rendered_text: rendered,
                        render_time: Instant::now(),
                        dpi: Self::calculate_optimal_dpi_static(),
                    };
                    cache.lock().unwrap().insert(next_page, data);
                }
            }
        });

        self.background_handle = Some(handle);
    }

    fn render_page_sync(&self, pdf_path: &PathBuf, page: usize) -> Result<String> {
        Self::render_page_static(pdf_path, page)
    }

    fn render_page_static(pdf_path: &PathBuf, page: usize) -> Result<String> {
        // Check if mutool is available
        if Command::new("mutool").arg("--version").output().is_ok() {
            let dpi = Self::calculate_optimal_dpi_static();

            // Render to text with optimal settings
            let output = Command::new("mutool")
                .args([
                    "draw",
                    "-F",
                    "txt", // Text output
                    "-r",
                    &dpi.to_string(), // Optimal DPI
                    "-c",
                    "gray", // Grayscale for speed
                    "-o",
                    "-", // Output to stdout
                    pdf_path.to_str().unwrap(),
                    &format!("{}", page + 1), // mutool uses 1-based pages
                ])
                .output()?;

            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Ok(format!(
                    "PDF Page {}\n\n[Render failed]\n\nUse ← → to navigate",
                    page + 1
                ))
            }
        } else {
            // Fallback
            Ok(format!(
                "PDF Page {}\n\n[mutool not available]\n\nUse ← → to navigate",
                page + 1
            ))
        }
    }

    fn calculate_optimal_dpi(&self) -> u16 {
        Self::calculate_optimal_dpi_static()
    }

    fn calculate_optimal_dpi_static() -> u16 {
        // Get terminal size
        if let Ok((cols, _rows)) = crossterm::terminal::size() {
            match cols {
                0..=80 => 72,     // Small terminal
                81..=120 => 96,   // Medium terminal
                121..=200 => 120, // Large terminal
                _ => 150,         // Very large terminal
            }
        } else {
            96 // Default
        }
    }

    pub fn get_cache_stats(&self) -> (usize, usize) {
        let cache = self.rendered_pages.lock().unwrap();
        (cache.pages.len(), cache.max_size)
    }
}

// ============= PROGRESSIVE LOADING =============

pub struct ProgressiveLoader {
    low_res_cache: HashMap<usize, String>,
    high_res_cache: HashMap<usize, String>,
}

impl ProgressiveLoader {
    pub fn new() -> Self {
        Self {
            low_res_cache: HashMap::new(),
            high_res_cache: HashMap::new(),
        }
    }

    pub fn load_progressive(&mut self, pdf_path: &PathBuf, page: usize) -> Result<(String, bool)> {
        // Check if we have high-res version
        if let Some(high_res) = self.high_res_cache.get(&page) {
            return Ok((high_res.clone(), true));
        }

        // Check if we have low-res version
        if let Some(low_res) = self.low_res_cache.get(&page) {
            // Start high-res render in background
            self.start_high_res_render(pdf_path.clone(), page);
            return Ok((low_res.clone(), false));
        }

        // Render low-res immediately
        let low_res = self.render_low_res(pdf_path, page)?;
        self.low_res_cache.insert(page, low_res.clone());

        // Start high-res render in background
        self.start_high_res_render(pdf_path.clone(), page);

        Ok((low_res, false))
    }

    fn render_low_res(&self, pdf_path: &PathBuf, page: usize) -> Result<String> {
        // Very fast, low quality render
        let output = Command::new("mutool")
            .args([
                "draw",
                "-F",
                "txt",
                "-r",
                "36", // Very low DPI
                "-c",
                "gray",
                "-o",
                "-",
                pdf_path.to_str().unwrap(),
                &format!("{}", page + 1),
            ])
            .output()?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn start_high_res_render(&self, pdf_path: PathBuf, page: usize) {
        thread::spawn(move || {
            // Render at full quality
            let _ = Command::new("mutool")
                .args([
                    "draw",
                    "-F",
                    "txt",
                    "-r",
                    "150", // High quality
                    "-o",
                    "-",
                    pdf_path.to_str().unwrap(),
                    &format!("{}", page + 1),
                ])
                .output();

            // In real impl, would update high_res_cache through a channel
        });
    }
}
