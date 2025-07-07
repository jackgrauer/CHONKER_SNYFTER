// Integration test for HTML rendering

use app_lib::html_renderer::HtmlRenderer;
use serde_json::json;

#[test]
fn test_html_renderer_with_structured_tables() {
    // Simulate structured table data
    let test_data = json!({
        "tables_found": 2,
        "chunks_extracted": 3,
        "formulas_detected": 0,
        "pages_processed": 1,
        "processing_time_ms": 1250,
        "tool_used": "🐹 CHONKER Real - docling",
        "tables": [
            {
                "headers": ["Name", "Age", "City"],
                "rows": [
                    ["John", "25", "New York"],
                    ["Jane", "30", "London"],
                    ["Bob", "35", "Paris"]
                ],
                "metadata": "page_1_table_1"
            },
            {
                "headers": ["Product", "Price", "Stock"],
                "rows": [
                    ["Laptop", "$999", "50"],
                    ["Mouse", "$25", "200"],
                    ["Keyboard", "$75", "150"]
                ],
                "metadata": "page_1_table_2"
            }
        ]
    });

    // Create HTML renderer and generate formatted output
    let renderer = HtmlRenderer::new();
    let html_output = renderer.render_processing_results(&test_data);

    // Basic validation
    assert!(html_output.contains("<style>"));
    assert!(html_output.contains("chonker-table"));
    assert!(html_output.contains("John"));
    assert!(html_output.contains("Laptop"));
    assert!(html_output.contains("🐹 Table 1"));
    assert!(html_output.contains("🐹 Table 2"));

    // Save for manual inspection
    std::fs::write("test_output.html", &html_output).expect("Failed to write test output");
    
    println!("✅ HTML rendering test passed!");
    println!("📄 Output saved to test_output.html");
    println!("🔍 HTML length: {} characters", html_output.len());
}
