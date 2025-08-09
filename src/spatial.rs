use anyhow::Result;
use pdfium_render::prelude::*;

pub struct Spatial;

impl Spatial {
    pub fn extract(doc: &PdfDocument, pg: usize, tw: usize, th: usize) -> Result<Vec<Vec<char>>> {
        let page = doc.pages().get(pg as u16)?;
        let ph = page.height().value;
        let txt = page.text()?;

        let mut segs = vec![];
        for seg in txt.segments().iter() {
            let b = seg.bounds();
            let t = seg.text();
            if !t.trim().is_empty() {
                segs.push((
                    t,
                    b.left().value,
                    ph - b.top().value,
                    b.right().value - b.left().value,
                    b.top().value - b.bottom().value,
                ));
            }
        }

        if segs.is_empty() {
            return Ok(vec![vec![' '; tw]; th]);
        }

        // Use fixed character dimensions like the GUI does
        let cw = 6.0; // Fixed character width
        let ch = 12.0; // Fixed character height

        let minx = segs
            .iter()
            .map(|s| s.1)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let miny = segs
            .iter()
            .map(|s| s.2)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let maxx = segs
            .iter()
            .map(|s| s.1 + s.3)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(100.0);
        let maxy = segs
            .iter()
            .map(|s| s.2 + s.4)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(100.0);

        // Create grid without scaling - use the requested dimensions directly
        let mut grid = vec![vec![' '; tw]; th];

        for (txt, x, y, _w, h) in segs {
            let z = if h > 14.0 && y < 100.0 {
                150
            } else if h > 14.0 {
                125
            } else if y > maxy - 100.0 {
                75
            } else {
                100
            };

            let sx = ((x - minx) / cw) as usize;
            let sy = ((y - miny) / ch) as usize;

            for (i, ch) in txt.chars().enumerate() {
                let gx = sx + i;
                let gy = sy;
                if gx < grid[0].len() && gy < grid.len() {
                    if grid[gy][gx] == ' ' || z > 100 {
                        grid[gy][gx] = ch;
                    }
                }
            }
        }

        Ok(grid)
    }
}
