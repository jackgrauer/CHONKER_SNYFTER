#!/usr/bin/env cargo run --bin qwen_fixer --
//! Qwen-7B Second Pass Table Fixer CLI
//! 
//! This binary provides a command-line interface to the Qwen-7B table fixing functionality,
//! completing the pipeline: PDF → Docling → Broken Tables → Qwen-7B → Fixed Tables

use clap::Parser;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(name = "qwen_fixer")]
#[command(about = "Fix environmental lab tables using Qwen-7B second pass")]
#[command(version = "1.0.0")]
struct Cli {
    /// Input markdown file from extraction
    input: PathBuf,
    
    /// Output path for fixed document
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Timeout for Qwen calls in seconds
    #[arg(long, default_value = "60")]
    timeout: u64,
    
    /// Generate final QC report after fixing
    #[arg(long)]
    generate_qc: bool,
    
    /// Open result in Inlyne for viewing
    #[arg(long)]
    view: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    if !cli.input.exists() {
        eprintln!("❌ Input file not found: {:?}", cli.input);
        return Err("Input file not found".into());
    }
    
    // Determine output path
    let output_path = if let Some(output) = &cli.output {
        output.clone()
    } else {
        let input_stem = cli.input.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        cli.input.with_file_name(format!("{}_QWEN_FIXED.md", input_stem))
    };
    
    println!("🚀 Starting Qwen-7B Environmental Lab Table Fixer");
    println!("📄 Input: {:?}", cli.input);
    println!("📝 Output: {:?}", output_path);
    
    // Call Python script
    let mut python_cmd = Command::new("python3");
    python_cmd.arg("python/qwen_production_fixer.py")
        .arg(&cli.input)
        .arg("-o")
        .arg(&output_path)
        .arg("--timeout")
        .arg(cli.timeout.to_string());
    
    let start_time = std::time::Instant::now();
    let output = python_cmd.output()?;
    let duration = start_time.elapsed();
    
    if output.status.success() {
        println!("✅ Qwen second pass completed successfully!");
        println!("⏱️  Processing time: {:.1}s", duration.as_secs_f64());
        println!("📝 Fixed document: {:?}", output_path);
        
        // Print Python script output
        if !output.stdout.is_empty() {
            println!("\n📋 Processing details:");
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
        
        // Generate QC report if requested
        if cli.generate_qc {
            println!("📊 Generating final QC report...");
            
            // Run the main chonker extraction on fixed file to generate new QC report
            let mut qc_cmd = Command::new("./target/release/chonker");
            qc_cmd.arg("extract").arg(&output_path);
            
            match qc_cmd.output() {
                Ok(qc_output) => {
                    if qc_output.status.success() {
                        println!("✅ Final QC report generated!");
                    } else {
                        println!("⚠️ QC report generation failed: {}", 
                               String::from_utf8_lossy(&qc_output.stderr));
                    }
                }
                Err(e) => {
                    println!("⚠️ Could not run QC report generation: {}", e);
                    println!("💡 Run manually: ./target/release/chonker extract {:?}", output_path);
                }
            }
        }
        
        // Open in Inlyne if requested
        if cli.view {
            println!("👀 Opening in Inlyne...");
            match Command::new("inlyne").arg(&output_path).spawn() {
                Ok(_) => println!("📖 Opened in Inlyne for review"),
                Err(e) => println!("⚠️ Could not open in Inlyne: {}. Install with: cargo install inlyne", e),
            }
        }
        
        // Print the complete pipeline summary
        println!("\n🎉 Complete Environmental Lab Processing Pipeline:");
        println!("   📄 PDF → Docling v2 → {:?}", cli.input);
        println!("   🔧 Qwen-7B fixes → {:?}", output_path);
        if cli.generate_qc {
            println!("   📊 Final QC report → pdf_table_qc_report.md");
        }
        if cli.view {
            println!("   👀 Visual review → Inlyne");
        }
        
        Ok(())
    } else {
        eprintln!("❌ Qwen second pass failed");
        if !output.stderr.is_empty() {
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
        }
        if !output.stdout.is_empty() {
            eprintln!("Output: {}", String::from_utf8_lossy(&output.stdout));
        }
        Err("Qwen processing failed".into())
    }
}
