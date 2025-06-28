#!/bin/bash

echo "🐹 CHONKER Mouse Capture Test"
echo "============================="
echo
echo "This will help you test if mouse capture is working properly."
echo
echo "1. Run the debug app first:"
echo "   cargo run --bin debug_mouse"
echo
echo "2. Test these scenarios:"
echo "   - Click and drag in the LEFT pane only"
echo "   - Click and drag in the RIGHT pane only" 
echo "   - Try to drag from LEFT to RIGHT pane"
echo
echo "3. If mouse capture is working:"
echo "   ✅ You should see mouse events logged in the right pane"
echo "   ✅ Selection should NOT bleed between panes"
echo "   ✅ Terminal's native selection should be DISABLED"
echo
echo "4. If mouse capture is NOT working:"
echo "   ❌ No mouse events appear in the log"
echo "   ❌ Terminal still does native text selection"
echo "   ❌ Selection bleeds across pane boundaries"
echo
echo "5. Then test the main app:"
echo "   cargo run --bin chonker-tui"
echo
echo "6. In the main app:"
echo "   - Press 'm' to toggle between selection modes"
echo "   - Look for debug output in the terminal"
echo "   - Test selection in different modes"
echo
echo "Common Issues & Solutions:"
echo "========================="
echo
echo "Issue: No mouse events detected"
echo "Solution: Your terminal might not support mouse capture"
echo "Test with: echo -e \"\\e[?1000h\" then click around"
echo
echo "Issue: Selection still bleeds across panes"
echo "Solution: Mouse capture might not be fully working"
echo "Check: Look for 'DEBUG: Mouse capture ENABLED' messages"
echo
echo "Issue: Terminal dimension detection is wrong"
echo "Solution: Check the debug output for actual vs expected coordinates"
echo
echo "Press Enter to start the debug test..."
read

echo "Starting debug mouse test..."
cargo run --bin debug_mouse
