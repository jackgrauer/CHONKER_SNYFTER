#!/usr/bin/env python3
"""
MLX-VLM OCR Testing Script
Tests MLX-optimized vision models for document OCR on Apple Silicon
"""

import time
import sys
import os
from pathlib import Path

def test_mlx_vlm_ocr(pdf_path, output_file):
    """Test MLX-VLM for OCR with document understanding"""
    start_time = time.time()
    
    try:
        # Try to import MLX-VLM
        try:
            from mlx_vlm import load, generate
            from PIL import Image
            import fitz  # PyMuPDF for PDF to image conversion
        except ImportError as e:
            print(f"MLX-VLM not available: {e}")
            return None, 0
        
        # Convert PDF first page to image
        pdf_doc = fitz.open(pdf_path)
        page = pdf_doc[0]
        pix = page.get_pixmap(matrix=fitz.Matrix(2.0, 2.0))  # 2x resolution
        img_path = "/tmp/mlx_test_page.png"
        pix.save(img_path)
        pdf_doc.close()
        
        # Load MLX-optimized vision model
        print("Loading MLX-VLM model...")
        model, processor = load("mlx-community/Qwen2-VL-2B-Instruct-4bit")
        
        # Generate OCR with document understanding  
        prompt = "<image>\nPlease extract all the text from this document image. Maintain the original formatting and structure as much as possible."
        
        image = Image.open(img_path)
        
        print("Processing with MLX-VLM...")
        # Use the correct API for MLX-VLM
        result = generate(
            model=model,
            processor=processor,
            image=image,
            prompt=prompt,
            max_tokens=2048,
            verbose=False
        )
        
        processing_time = time.time() - start_time
        
        # Save result - handle tuple or string
        result_text = ""
        if isinstance(result, tuple):
            result_text = str(result[0]) if result else ""
        else:
            result_text = str(result)
        
        with open(output_file, 'w') as f:
            f.write(result_text)
        
        # Cleanup
        os.remove(img_path)
        
        return processing_time, len(result_text)
        
    except Exception as e:
        print(f"MLX-VLM test failed: {e}")
        return None, 0

def test_surya_ocr(pdf_path, output_file):
    """Test Surya OCR with MPS acceleration"""
    start_time = time.time()
    
    try:
        # Try to import Surya
        try:
            import torch
            from surya.detection import DetectionPredictor
            from surya.recognition import RecognitionPredictor
            from PIL import Image
            import fitz  # PyMuPDF
        except ImportError as e:
            print(f"Surya OCR not available: {e}")
            return None, 0
        
        # Convert PDF first page to image
        pdf_doc = fitz.open(pdf_path)
        page = pdf_doc[0]
        pix = page.get_pixmap(matrix=fitz.Matrix(2.0, 2.0))  # 2x resolution
        img_path = "/tmp/surya_test_page.png"
        pix.save(img_path)
        pdf_doc.close()
        
        # Set device to MPS for Apple Silicon
        device = "mps" if torch.backends.mps.is_available() else "cpu"
        print(f"Using device: {device}")
        
        # Load models
        print("Loading Surya OCR models...")
        det_predictor = DetectionPredictor()
        rec_predictor = RecognitionPredictor()
        
        # Process image
        print("Processing with Surya OCR...")
        image = Image.open(img_path)
        predictions = rec_predictor(
            [image],
            task_names=["ocr_with_boxes"],
            det_predictor=det_predictor,
            math_mode=True
        )
        
        # Extract text
        result_text = "\n".join([line.text for line in predictions[0].text_lines])
        
        processing_time = time.time() - start_time
        
        # Save result
        with open(output_file, 'w') as f:
            f.write(result_text)
        
        # Cleanup
        os.remove(img_path)
        
        return processing_time, len(result_text)
        
    except Exception as e:
        print(f"Surya OCR test failed: {e}")
        return None, 0

def main():
    if len(sys.argv) != 2:
        print("Usage: python test_mlx_vlm_ocr.py <pdf_file>")
        sys.exit(1)
    
    pdf_path = sys.argv[1]
    if not os.path.exists(pdf_path):
        print(f"PDF file not found: {pdf_path}")
        sys.exit(1)
    
    print("=== Apple Silicon OCR Testing ===")
    print(f"Testing: {pdf_path}")
    print()
    
    # Test MLX-VLM
    print("1. Testing MLX-VLM...")
    mlx_time, mlx_size = test_mlx_vlm_ocr(pdf_path, "output_mlx_vlm.md")
    if mlx_time:
        print(f"   MLX-VLM: {mlx_time:.1f}s, {mlx_size} bytes")
    else:
        print("   MLX-VLM: Failed or not available")
    print()
    
    # Test Surya OCR
    print("2. Testing Surya OCR...")
    surya_time, surya_size = test_surya_ocr(pdf_path, "output_surya.md")
    if surya_time:
        print(f"   Surya OCR: {surya_time:.1f}s, {surya_size} bytes")
    else:
        print("   Surya OCR: Failed or not available")
    print()
    
    # Summary
    print("=== Results Summary ===")
    if mlx_time:
        print(f"MLX-VLM: {mlx_time:.1f}s, {mlx_size} bytes")
    if surya_time:
        print(f"Surya OCR: {surya_time:.1f}s, {surya_size} bytes")
    
    print("\nBaseline comparison (Tesseract): 11.7s, 8671 bytes")
    print("Target: Beat 11.7s with similar or better quality")

if __name__ == "__main__":
    main()
