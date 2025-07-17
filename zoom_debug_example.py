"""
Example usage of the OpenHands zoom debugging functions.
This demonstrates how to use the debugging functions in practice.
"""

from zoom_debug_functions import (
    debug_gesture_flow,
    test_zoom_methods,
    validate_html_zoom,
    fix_zoom_conflicts,
    run_comprehensive_zoom_debug
)

def demonstrate_individual_functions():
    """Demonstrate each debugging function individually"""
    
    print("=" * 60)
    print("INDIVIDUAL FUNCTION DEMONSTRATIONS")
    print("=" * 60)
    
    # 1. Debug gesture flow
    print("\n1. GESTURE FLOW ANALYSIS")
    print("-" * 30)
    gesture_results = debug_gesture_flow()
    print(f"Status: {gesture_results['status']}")
    print(f"Issues detected: {len(gesture_results['issues_detected'])}")
    print(f"Recommendations: {len(gesture_results['recommendations'])}")
    
    # 2. Test zoom methods
    print("\n2. ZOOM METHODS TESTING")
    print("-" * 30)
    method_results = test_zoom_methods()
    print(f"Status: {method_results['status']}")
    print("Method effectiveness:")
    for method, details in method_results['method_tests'].items():
        print(f"  {method}: {details['effectiveness']}")
    
    # 3. Validate HTML zoom
    print("\n3. HTML ZOOM VALIDATION")
    print("-" * 30)
    html_results = validate_html_zoom()
    print(f"Status: {html_results['status']}")
    print("HTML zoom test results:")
    for test_name, test_result in html_results['html_zoom_tests'].items():
        print(f"  {test_name}: {test_result['visual_impact']}")
    
    # 4. Fix zoom conflicts
    print("\n4. ZOOM CONFLICT ANALYSIS")
    print("-" * 30)
    conflict_results = fix_zoom_conflicts()
    print(f"Status: {conflict_results['status']}")
    print(f"Conflicts detected: {len(conflict_results['conflicts_detected'])}")
    print(f"Fixes available: {len(conflict_results['fixes_applied'])}")

def demonstrate_comprehensive_debug():
    """Demonstrate the comprehensive debugging function"""
    
    print("\n" + "=" * 60)
    print("COMPREHENSIVE DEBUG DEMONSTRATION")
    print("=" * 60)
    
    # Run comprehensive debug
    comprehensive_results = run_comprehensive_zoom_debug()
    
    print(f"\nDebug Session ID: {comprehensive_results['debug_session_id']}")
    print(f"Overall Status: {comprehensive_results['status']}")
    
    # Summary
    summary = comprehensive_results['summary']
    print(f"\nSUMMARY:")
    print(f"  Functions run: {summary['successful_functions']}/{summary['total_debug_functions']}")
    print(f"  Total issues: {summary['total_issues_detected']}")
    print(f"  Critical issues: {summary['critical_issues_count']}")
    print(f"  Overall status: {summary['overall_status']}")
    
    # Critical issues
    if comprehensive_results['critical_issues']:
        print(f"\nCRITICAL ISSUES:")
        for issue in comprehensive_results['critical_issues']:
            print(f"  • {issue['issue']} (Line {issue['line_number']})")
            print(f"    {issue['description']}")
    
    # Action plan
    print(f"\nACTION PLAN:")
    for phase in comprehensive_results['action_plan']:
        print(f"\n{phase['phase'].upper()}:")
        for action in phase['actions']:
            print(f"  • {action}")

def demonstrate_with_mock_app():
    """Demonstrate how functions would work with a real app instance"""
    
    print("\n" + "=" * 60)
    print("MOCK APP INSTANCE DEMONSTRATION")
    print("=" * 60)
    
    # Mock app instance
    class MockApp:
        def __init__(self):
            self.faithful_output = MockTextEdit()
            self.text_zoom = 12
            self.pdf_zoom = 1.0
            self.active_pane = 'right'
        
        def eventFilter(self, obj, event):
            return False
    
    class MockTextEdit:
        def __init__(self):
            self._font_size = 12
        
        def font(self):
            return MockFont(self._font_size)
        
        def setFont(self, font):
            self._font_size = font.pointSize()
        
        def zoomIn(self, factor=1):
            pass  # Mock zoom in
        
        def zoomOut(self, factor=1):
            pass  # Mock zoom out
        
        def toHtml(self):
            return "<p>Mock HTML content</p>"
        
        def setHtml(self, html):
            pass  # Mock set HTML
    
    class MockFont:
        def __init__(self, size):
            self._size = size
        
        def pointSize(self):
            return self._size
        
        def setPointSize(self, size):
            self._size = size
    
    # Create mock app
    mock_app = MockApp()
    
    # Test with mock app
    print("\nTesting with mock app instance:")
    html_results = validate_html_zoom(app_instance=mock_app)
    print(f"HTML validation status: {html_results['status']}")
    print(f"Widget zoom visual impact: {html_results['html_zoom_tests']['widget_zoom']['visual_impact']}")
    print(f"Font zoom visual impact: {html_results['html_zoom_tests']['font_zoom']['visual_impact']}")

def main():
    """Main demonstration function"""
    
    print("OPENHANDS ZOOM DEBUG FUNCTIONS DEMONSTRATION")
    print("This shows how to use the debugging functions in practice.")
    
    # Run demonstrations
    demonstrate_individual_functions()
    demonstrate_comprehensive_debug()
    demonstrate_with_mock_app()
    
    print("\n" + "=" * 60)
    print("DEMONSTRATION COMPLETE")
    print("=" * 60)
    print("\nKey takeaways:")
    print("1. Functions work in both mock and real app modes")
    print("2. Comprehensive debug provides full analysis")
    print("3. Individual functions allow targeted debugging")
    print("4. Results include actionable recommendations")
    print("5. Integration with Pydantic models provides validation")

if __name__ == "__main__":
    main()