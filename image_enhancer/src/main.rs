use anyhow::{Context, Result};
use clap::Parser;
use image::{DynamicImage, GrayImage, Luma, GenericImage, GenericImageView};
use std::path::Path;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input image path
    input: String,
    
    /// Output image path (optional, defaults to input_enhanced.png)
    #[arg(short, long)]
    output: Option<String>,
    
    /// Apply deskewing
    #[arg(long, default_value_t = true)]
    deskew: bool,
    
    /// Apply contrast enhancement
    #[arg(long, default_value_t = true)]
    contrast: bool,
    
    /// Apply noise reduction
    #[arg(long, default_value_t = true)]
    denoise: bool,
    
    /// Apply sharpening
    #[arg(long, default_value_t = true)]
    sharpen: bool,
    
    /// Contrast stretch factor (0.0-1.0)
    #[arg(long, default_value_t = 0.02)]
    contrast_factor: f32,
    
    /// Gaussian blur radius for denoising
    #[arg(long, default_value_t = 0.5)]
    blur_radius: f32,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    println!("🔧 Loading image: {}", args.input);
    let img = image::open(&args.input)
        .with_context(|| format!("Failed to open image: {}", args.input))?;
    
    let mut processed_img = img.clone();
    
    if args.deskew {
        println!("📐 Applying deskewing...");
        processed_img = deskew_image(processed_img)?;
    }
    
    if args.contrast {
        println!("🔆 Enhancing contrast...");
        processed_img = enhance_contrast(processed_img, args.contrast_factor)?;
    }
    
    if args.denoise {
        println!("🧹 Reducing noise...");
        processed_img = reduce_noise(processed_img, args.blur_radius)?;
    }
    
    if args.sharpen {
        println!("🔍 Sharpening image...");
        processed_img = sharpen_image(processed_img)?;
    }
    
    let output_path = args.output.unwrap_or_else(|| {
        let input_path = Path::new(&args.input);
        let stem = input_path.file_stem().unwrap().to_str().unwrap();
        let extension = input_path.extension().unwrap_or_default().to_str().unwrap();
        format!("{}_enhanced.{}", stem, extension)
    });
    
    println!("💾 Saving enhanced image to: {}", output_path);
    processed_img.save(&output_path)
        .with_context(|| format!("Failed to save image: {}", output_path))?;
    
    println!("✅ Image enhancement complete!");
    Ok(())
}

fn deskew_image(img: DynamicImage) -> Result<DynamicImage> {
    // Simple deskewing using Hough transform approximation
    // Convert to grayscale for processing
    let gray = img.to_luma8();
    
    // Apply edge detection to find text lines
    let edges = detect_edges(&gray);
    
    // Find the dominant angle
    let angle = find_skew_angle(&edges);
    
    if angle.abs() > 0.5 {
        println!("📐 Detected skew angle: {:.2}°", angle);
        rotate_image(img, -angle)
    } else {
        Ok(img)
    }
}

fn detect_edges(gray: &GrayImage) -> GrayImage {
    // Simple Sobel edge detection
    let width = gray.width();
    let height = gray.height();
    let mut edges = GrayImage::new(width, height);
    
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let gx = (gray.get_pixel(x + 1, y - 1)[0] as i32) + 
                    (2 * gray.get_pixel(x + 1, y)[0] as i32) + 
                    (gray.get_pixel(x + 1, y + 1)[0] as i32) - 
                    (gray.get_pixel(x - 1, y - 1)[0] as i32) - 
                    (2 * gray.get_pixel(x - 1, y)[0] as i32) - 
                    (gray.get_pixel(x - 1, y + 1)[0] as i32);
            
            let gy = (gray.get_pixel(x - 1, y + 1)[0] as i32) + 
                    (2 * gray.get_pixel(x, y + 1)[0] as i32) + 
                    (gray.get_pixel(x + 1, y + 1)[0] as i32) - 
                    (gray.get_pixel(x - 1, y - 1)[0] as i32) - 
                    (2 * gray.get_pixel(x, y - 1)[0] as i32) - 
                    (gray.get_pixel(x + 1, y - 1)[0] as i32);
            
            let magnitude = ((gx * gx + gy * gy) as f32).sqrt() as u8;
            edges.put_pixel(x, y, Luma([magnitude]));
        }
    }
    
    edges
}

fn find_skew_angle(edges: &GrayImage) -> f32 {
    // Simplified Hough transform to find dominant line angle
    let mut angle_votes = vec![0; 360];
    let width = edges.width();
    let height = edges.height();
    
    for y in 0..height {
        for x in 0..width {
            if edges.get_pixel(x, y)[0] > 50 {
                // Vote for angles from -45 to +45 degrees
                for angle_deg in -45..=45 {
                    let angle_rad = (angle_deg as f32) * std::f32::consts::PI / 180.0;
                    let rho = (x as f32) * angle_rad.cos() + (y as f32) * angle_rad.sin();
                    
                    if rho > 0.0 && rho < (width + height) as f32 {
                        angle_votes[(angle_deg + 180) as usize] += 1;
                    }
                }
            }
        }
    }
    
    // Find the angle with the most votes
    let max_votes_angle = angle_votes.iter()
        .enumerate()
        .max_by_key(|(_, &votes)| votes)
        .map(|(angle, _)| angle as f32 - 180.0)
        .unwrap_or(0.0);
    
    max_votes_angle
}

fn rotate_image(img: DynamicImage, angle_deg: f32) -> Result<DynamicImage> {
    // Simple rotation using nearest neighbor
    let angle_rad = angle_deg * std::f32::consts::PI / 180.0;
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();
    
    let width = img.width();
    let height = img.height();
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    
    let mut rotated = DynamicImage::new_rgb8(width, height);
    
    for y in 0..height {
        for x in 0..width {
            let x_centered = x as f32 - center_x;
            let y_centered = y as f32 - center_y;
            
            let x_rot = x_centered * cos_a - y_centered * sin_a + center_x;
            let y_rot = x_centered * sin_a + y_centered * cos_a + center_y;
            
            if x_rot >= 0.0 && x_rot < width as f32 && y_rot >= 0.0 && y_rot < height as f32 {
                let pixel = img.get_pixel(x_rot as u32, y_rot as u32);
                rotated.put_pixel(x, y, pixel);
            }
        }
    }
    
    Ok(rotated)
}

fn enhance_contrast(img: DynamicImage, factor: f32) -> Result<DynamicImage> {
    let gray = img.to_luma8();
    let width = gray.width();
    let height = gray.height();
    
    // Simple contrast enhancement
    let mut enhanced = GrayImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let pixel = gray.get_pixel(x, y)[0];
            let contrast_pixel = ((pixel as f32 - 128.0) * (1.0 + factor) + 128.0)
                .max(0.0).min(255.0) as u8;
            enhanced.put_pixel(x, y, Luma([contrast_pixel]));
        }
    }
    
    Ok(DynamicImage::ImageLuma8(enhanced))
}

fn reduce_noise(img: DynamicImage, _radius: f32) -> Result<DynamicImage> {
    let gray = img.to_luma8();
    let width = gray.width();
    let height = gray.height();
    
    // Simple median filter for noise reduction
    let mut denoised = GrayImage::new(width, height);
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let mut neighbors = Vec::new();
            for dy in -1..=1 {
                for dx in -1..=1 {
                    neighbors.push(gray.get_pixel((x as i32 + dx) as u32, (y as i32 + dy) as u32)[0]);
                }
            }
            neighbors.sort_unstable();
            let median = neighbors[4]; // middle element of 9
            denoised.put_pixel(x, y, Luma([median]));
        }
    }
    
    Ok(DynamicImage::ImageLuma8(denoised))
}

fn sharpen_image(img: DynamicImage) -> Result<DynamicImage> {
    let gray = img.to_luma8();
    let width = gray.width();
    let height = gray.height();
    
    // Simple sharpening kernel
    let kernel = [
        [0, -1, 0],
        [-1, 5, -1],
        [0, -1, 0]
    ];
    
    let mut sharpened = GrayImage::new(width, height);
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let mut sum = 0i32;
            for ky in 0..3 {
                for kx in 0..3 {
                    let pixel = gray.get_pixel(x + kx - 1, y + ky - 1)[0] as i32;
                    sum += pixel * kernel[ky as usize][kx as usize];
                }
            }
            let sharpened_value = sum.max(0).min(255) as u8;
            sharpened.put_pixel(x, y, Luma([sharpened_value]));
        }
    }
    
    Ok(DynamicImage::ImageLuma8(sharpened))
}
