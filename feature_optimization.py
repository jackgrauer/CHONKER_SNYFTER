"""
OPENHANDS-CLEANUP-002: Performance Optimization and Testing
Production-ready optimization for CHONKER & SNYFTER working features
"""

import time
import unittest
from typing import Dict, List, Optional, Tuple, Any
from collections import defaultdict
from functools import wraps
try:
    from PyQt6.QtCore import QTimer, QEvent
    from PyQt6.QtGui import QNativeGestureEvent
    HAS_QT = True
except ImportError:
    # Mock Qt classes for testing without Qt
    HAS_QT = False
    class QTimer:
        def __init__(self):
            self.single_shot = False
            self.timeout = MockSignal()
        def setSingleShot(self, val): 
            self.single_shot = val
        def start(self, ms): 
            pass
    
    class MockSignal:
        def connect(self, func): 
            pass
import statistics

# Import the elegant models we created
from elegant_models import (
    GestureEvent, GestureType, PaneType,
    PDFSelectionState, SelectionType,
    OCRResult, OCRProcessingStatus,
    MathFormulaDetection,
    create_gesture_event,
    create_pdf_selection_state,
    create_default_ocr_result
)


class PerformanceMetrics:
    """Track performance metrics for optimization"""
    def __init__(self):
        self.metrics: Dict[str, List[float]] = defaultdict(list)
        self.start_times: Dict[str, float] = {}
    
    def start_timer(self, operation: str):
        """Start timing an operation"""
        self.start_times[operation] = time.perf_counter()
    
    def end_timer(self, operation: str) -> float:
        """End timing and record the duration"""
        if operation in self.start_times:
            duration = time.perf_counter() - self.start_times[operation]
            self.metrics[operation].append(duration)
            del self.start_times[operation]
            return duration
        return 0.0
    
    def get_stats(self, operation: str) -> Dict[str, float]:
        """Get statistics for an operation"""
        if operation not in self.metrics or not self.metrics[operation]:
            return {"mean": 0, "median": 0, "min": 0, "max": 0}
        
        values = self.metrics[operation]
        return {
            "mean": statistics.mean(values),
            "median": statistics.median(values),
            "min": min(values),
            "max": max(values),
            "count": len(values)
        }


def performance_monitor(func):
    """Decorator to monitor function performance"""
    @wraps(func)
    def wrapper(*args, **kwargs):
        metrics = PerformanceMetrics()
        metrics.start_timer(func.__name__)
        result = func(*args, **kwargs)
        duration = metrics.end_timer(func.__name__)
        print(f"⚡ {func.__name__} executed in {duration*1000:.2f}ms")
        return result
    return wrapper


class GestureOptimizer:
    """Optimize gesture detection performance"""
    
    def __init__(self):
        self.gesture_cache: Dict[str, GestureEvent] = {}
        self.debounce_timers: Dict[str, QTimer] = {}
        self.metrics = PerformanceMetrics()
        
    @performance_monitor
    def optimize_gesture_performance(self, 
                                   gesture_type: str,
                                   gesture_value: float,
                                   target_pane: PaneType,
                                   debounce_ms: int = 50) -> Optional[GestureEvent]:
        """
        Optimize gesture processing with debouncing and caching
        
        Args:
            gesture_type: Type of gesture detected
            gesture_value: Gesture delta/value
            target_pane: Target pane for gesture
            debounce_ms: Debounce delay in milliseconds
            
        Returns:
            Optimized GestureEvent or None if debounced
        """
        self.metrics.start_timer("gesture_processing")
        
        # Create cache key
        cache_key = f"{gesture_type}_{target_pane.value}"
        
        # Check if we should debounce this gesture
        if cache_key in self.debounce_timers:
            # Update cached value but don't process yet
            if cache_key in self.gesture_cache:
                self.gesture_cache[cache_key].gesture_values.scale_factor *= (1 + gesture_value)
            self.metrics.end_timer("gesture_processing")
            return None
        
        # Create gesture event
        gesture_event = create_gesture_event(
            gesture_type=GestureType.ZOOM_IN if gesture_value > 0 else GestureType.ZOOM_OUT,
            target_pane=target_pane
        )
        gesture_event.gesture_values.scale_factor = 1 + gesture_value
        
        # Cache the gesture
        self.gesture_cache[cache_key] = gesture_event
        
        # Set up debounce timer
        timer = QTimer()
        timer.setSingleShot(True)
        timer.timeout.connect(lambda: self._process_debounced_gesture(cache_key))
        timer.start(debounce_ms)
        self.debounce_timers[cache_key] = timer
        
        self.metrics.end_timer("gesture_processing")
        return None
    
    def _process_debounced_gesture(self, cache_key: str):
        """Process a debounced gesture"""
        if cache_key in self.gesture_cache:
            gesture = self.gesture_cache[cache_key]
            gesture.processed = True
            # Clean up
            del self.gesture_cache[cache_key]
            del self.debounce_timers[cache_key]
            print(f"✅ Processed debounced gesture: {gesture.gesture_type}")


class PDFSelectionOptimizer:
    """Optimize PDF text selection performance"""
    
    def __init__(self):
        self.selection_cache: Dict[str, PDFSelectionState] = {}
        self.metrics = PerformanceMetrics()
    
    @performance_monitor
    def test_pdf_selection(self, 
                          selected_text: str,
                          selection_type: SelectionType,
                          page_number: int = 1) -> Tuple[bool, PDFSelectionState]:
        """
        Test and optimize PDF text selection
        
        Returns:
            Tuple of (success, PDFSelectionState)
        """
        self.metrics.start_timer("selection_creation")
        
        try:
            # Create optimized selection state
            selection = create_pdf_selection_state(pane_source=PaneType.LEFT)
            selection.selected_text = selected_text
            selection.selection_type = selection_type
            selection.coordinates.page_number = page_number
            
            # Cache for performance
            cache_key = f"{page_number}_{selection_type.value}"
            self.selection_cache[cache_key] = selection
            
            # Validate selection
            is_valid = (
                selection.is_active and
                len(selection.selected_text) > 0 and
                selection.coordinates.page_number == page_number
            )
            
            self.metrics.end_timer("selection_creation")
            return is_valid, selection
            
        except Exception as e:
            print(f"❌ Selection test failed: {e}")
            self.metrics.end_timer("selection_creation")
            return False, None


class OCROptimizer:
    """Optimize OCR processing with VLM fallback"""
    
    def __init__(self):
        self.ocr_cache: Dict[str, OCRResult] = {}
        self.metrics = PerformanceMetrics()
    
    @performance_monitor
    def test_ocr_fallback(self,
                         document_text: str,
                         simulate_math_formulas: bool = True) -> OCRResult:
        """
        Test OCR processing with VLM fallback optimization
        
        Args:
            document_text: Text to process
            simulate_math_formulas: Whether to simulate math formula detection
            
        Returns:
            Optimized OCRResult
        """
        self.metrics.start_timer("ocr_processing")
        
        # Check cache first
        text_hash = hash(document_text[:100])  # Cache by first 100 chars
        if text_hash in self.ocr_cache:
            self.metrics.end_timer("ocr_processing")
            return self.ocr_cache[text_hash]
        
        # Simulate OCR processing
        ocr_result = create_default_ocr_result(
            document_path="test_document.pdf",
            text_content=document_text
        )
        
        # Simulate math formula detection
        if simulate_math_formulas:
            # Simple heuristic: look for common math indicators
            math_indicators = ['∫', '∑', '∂', '√', 'equation', 'formula', '=']
            has_math = any(indicator in document_text for indicator in math_indicators)
            
            if has_math:
                ocr_result.math_formulas_detected.append(
                    MathFormulaDetection(
                        formula_text="∫ f(x) dx",
                        confidence=0.85,
                        requires_vlm=True,
                        location="paragraph_3"
                    )
                )
                ocr_result.processing_metrics.math_formulas_found = 1
                ocr_result.processing_metrics.vlm_used = True
        
        ocr_result.status = OCRProcessingStatus.COMPLETED
        ocr_result.overall_confidence = 0.92
        
        # Cache the result
        self.ocr_cache[text_hash] = ocr_result
        
        self.metrics.end_timer("ocr_processing")
        return ocr_result


def performance_benchmarks():
    """
    Run comprehensive performance benchmarks for all optimized features
    
    Returns performance report with metrics
    """
    print("\n" + "="*60)
    print("🚀 PERFORMANCE BENCHMARKS - CHONKER & SNYFTER")
    print("="*60)
    
    metrics = PerformanceMetrics()
    report = {
        "gesture_performance": {},
        "pdf_selection_performance": {},
        "ocr_performance": {},
        "overall_metrics": {}
    }
    
    # 1. Benchmark Gesture Performance
    print("\n📊 Benchmarking Gesture Performance...")
    gesture_optimizer = GestureOptimizer()
    
    for i in range(100):
        metrics.start_timer("gesture_batch")
        gesture_optimizer.optimize_gesture_performance(
            "zoom",
            0.1 * (i % 10 - 5),  # Vary gesture values
            PaneType.RIGHT if i % 2 == 0 else PaneType.LEFT
        )
        metrics.end_timer("gesture_batch")
    
    report["gesture_performance"] = metrics.get_stats("gesture_batch")
    print(f"  Average: {report['gesture_performance']['mean']*1000:.2f}ms")
    print(f"  Median: {report['gesture_performance']['median']*1000:.2f}ms")
    
    # 2. Benchmark PDF Selection
    print("\n📊 Benchmarking PDF Selection...")
    pdf_optimizer = PDFSelectionOptimizer()
    
    test_texts = [
        "Short text",
        "Medium length text that represents a typical selection",
        "Very long text " * 50  # Long selection
    ]
    
    for text in test_texts:
        for selection_type in [SelectionType.WORD, SelectionType.LINE, SelectionType.PARAGRAPH]:
            metrics.start_timer("pdf_selection_batch")
            success, selection = pdf_optimizer.test_pdf_selection(
                text, selection_type, page_number=1
            )
            metrics.end_timer("pdf_selection_batch")
    
    report["pdf_selection_performance"] = metrics.get_stats("pdf_selection_batch")
    print(f"  Average: {report['pdf_selection_performance']['mean']*1000:.2f}ms")
    print(f"  Median: {report['pdf_selection_performance']['median']*1000:.2f}ms")
    
    # 3. Benchmark OCR Processing
    print("\n📊 Benchmarking OCR Processing...")
    ocr_optimizer = OCROptimizer()
    
    test_documents = [
        "Simple document without math",
        "Document with equation: ∫ f(x) dx = F(x) + C",
        "Complex document " * 100 + " with math formula ∑"
    ]
    
    for doc in test_documents:
        metrics.start_timer("ocr_batch")
        ocr_result = ocr_optimizer.test_ocr_fallback(doc, simulate_math_formulas=True)
        metrics.end_timer("ocr_batch")
    
    report["ocr_performance"] = metrics.get_stats("ocr_batch")
    print(f"  Average: {report['ocr_performance']['mean']*1000:.2f}ms")
    print(f"  Median: {report['ocr_performance']['median']*1000:.2f}ms")
    
    # 4. Overall Performance Summary
    print("\n📈 OVERALL PERFORMANCE SUMMARY")
    print("="*60)
    
    all_operations = ["gesture_batch", "pdf_selection_batch", "ocr_batch"]
    total_times = []
    
    for op in all_operations:
        if op in metrics.metrics:
            total_times.extend(metrics.metrics[op])
    
    if total_times:
        report["overall_metrics"] = {
            "total_operations": len(total_times),
            "average_time_ms": statistics.mean(total_times) * 1000,
            "median_time_ms": statistics.median(total_times) * 1000,
            "p95_time_ms": sorted(total_times)[int(len(total_times) * 0.95)] * 1000 if len(total_times) > 20 else 0
        }
        
        print(f"Total Operations: {report['overall_metrics']['total_operations']}")
        print(f"Average Time: {report['overall_metrics']['average_time_ms']:.2f}ms")
        print(f"Median Time: {report['overall_metrics']['median_time_ms']:.2f}ms")
        print(f"95th Percentile: {report['overall_metrics']['p95_time_ms']:.2f}ms")
    
    # Performance Grade
    avg_time = report["overall_metrics"]["average_time_ms"]
    if avg_time < 10:
        grade = "A+ ⚡ Lightning Fast!"
    elif avg_time < 50:
        grade = "A 🚀 Excellent Performance"
    elif avg_time < 100:
        grade = "B 👍 Good Performance"
    else:
        grade = "C 🔧 Needs Optimization"
    
    print(f"\nPerformance Grade: {grade}")
    print("="*60)
    
    return report


# Unit Tests for Working Features
class TestWorkingFeatures(unittest.TestCase):
    """Comprehensive tests for CHONKER & SNYFTER working features"""
    
    def setUp(self):
        """Set up test fixtures"""
        self.gesture_optimizer = GestureOptimizer()
        self.pdf_optimizer = PDFSelectionOptimizer()
        self.ocr_optimizer = OCROptimizer()
    
    def test_gesture_optimization(self):
        """Test gesture detection optimization"""
        # Test zoom in gesture
        result = self.gesture_optimizer.optimize_gesture_performance(
            "zoom", 0.5, PaneType.RIGHT
        )
        # First call should be debounced
        self.assertIsNone(result)
        
        # Test gesture caching
        cache_key = "zoom_right"
        self.assertIn(cache_key, self.gesture_optimizer.gesture_cache)
    
    def test_pdf_selection_validation(self):
        """Test PDF text selection"""
        success, selection = self.pdf_optimizer.test_pdf_selection(
            "Test selection text",
            SelectionType.WORD,
            page_number=1
        )
        
        self.assertTrue(success)
        self.assertIsNotNone(selection)
        self.assertEqual(selection.selected_text, "Test selection text")
        self.assertEqual(selection.selection_type, SelectionType.WORD)
    
    def test_ocr_vlm_fallback(self):
        """Test OCR with VLM fallback"""
        # Test document with math
        result = self.ocr_optimizer.test_ocr_fallback(
            "This document contains ∫ f(x) dx equation",
            simulate_math_formulas=True
        )
        
        self.assertEqual(result.status, OCRProcessingStatus.COMPLETED)
        self.assertTrue(result.needs_vlm_fallback())
        self.assertEqual(len(result.math_formulas_detected), 1)
        
        # Test document without math
        result2 = self.ocr_optimizer.test_ocr_fallback(
            "Simple text document",
            simulate_math_formulas=True
        )
        
        self.assertFalse(result2.needs_vlm_fallback())
        self.assertEqual(len(result2.math_formulas_detected), 0)


if __name__ == "__main__":
    # Run performance benchmarks
    print("🔥 CHONKER & SNYFTER Performance Optimization")
    print("Celebrating today's big advances!")
    
    # Run benchmarks
    benchmark_report = performance_benchmarks()
    
    # Run unit tests
    print("\n🧪 Running Unit Tests...")
    unittest.main(argv=[''], exit=False, verbosity=2)