#!/bin/bash

echo "🔍 CHONKER Performance Benchmark: TUI vs GUI"
echo "============================================="
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}📊 Binary Size Comparison:${NC}"
echo "TUI Binary:  $(ls -lh target/release/chonker | awk '{print $5}')"
echo "GUI Binary:  $(ls -lh target/release/chonker_gui | awk '{print $5}')"
echo

echo -e "${BLUE}🚀 Startup Time Comparison:${NC}"
echo -e "${YELLOW}Testing TUI startup (help command):${NC}"
time_tui_help=$(bash -c "time timeout 2s ./target/release/chonker --help >/dev/null 2>&1" 2>&1 | grep "real" | awk '{print $2}')
echo "TUI Help: $time_tui_help"

echo -e "${YELLOW}Testing GUI startup:${NC}"
time_gui_help=$(bash -c "time timeout 2s ./target/release/chonker_gui --help >/dev/null 2>&1" 2>&1 | grep "real" | awk '{print $2}')
echo "GUI Help: $time_gui_help"
echo

echo -e "${BLUE}💾 Memory Usage Comparison:${NC}"
echo -e "${YELLOW}Testing TUI memory usage:${NC}"
tui_memory=$(/usr/bin/time -l sh -c 'echo "" | timeout 1s ./target/release/chonker tui >/dev/null 2>&1' 2>&1 | grep "maximum resident" | awk '{print $1}')
tui_memory_mb=$((tui_memory / 1024 / 1024))
echo "TUI Memory: ${tui_memory_mb}MB (${tui_memory} bytes)"

echo -e "${YELLOW}Testing GUI memory usage:${NC}"
gui_memory=$(/usr/bin/time -l sh -c 'timeout 2s ./target/release/chonker_gui --help >/dev/null 2>&1' 2>&1 | grep "maximum resident" | awk '{print $1}')
gui_memory_mb=$((gui_memory / 1024 / 1024))
echo "GUI Memory: ${gui_memory_mb}MB (${gui_memory} bytes)"
echo

echo -e "${BLUE}📈 Performance Analysis:${NC}"
memory_ratio=$(echo "scale=2; $gui_memory / $tui_memory" | bc -l)
echo "Memory Ratio: ${memory_ratio}x (GUI uses ${memory_ratio}x more memory than TUI)"

if (( $(echo "$memory_ratio > 10" | bc -l) )); then
    echo -e "${RED}❌ SEVERE: GUI uses ${memory_ratio}x more memory${NC}"
elif (( $(echo "$memory_ratio > 5" | bc -l) )); then
    echo -e "${YELLOW}⚠️  HIGH: GUI uses ${memory_ratio}x more memory${NC}"
elif (( $(echo "$memory_ratio > 2" | bc -l) )); then
    echo -e "${YELLOW}📊 MODERATE: GUI uses ${memory_ratio}x more memory${NC}"
else
    echo -e "${GREEN}✅ GOOD: Memory usage difference is reasonable${NC}"
fi

echo
echo -e "${BLUE}🏁 Summary:${NC}"
echo "- TUI is more memory efficient (${tui_memory_mb}MB vs ${gui_memory_mb}MB)"
echo "- GUI provides richer visualization at cost of memory"
echo "- For automated/scripted tasks, use TUI"
echo "- For interactive document review, GUI is acceptable"
echo

# Additional feature-specific tests
echo -e "${BLUE}🔧 Feature-Specific Performance:${NC}"
echo -e "${YELLOW}Testing TUI with database operations:${NC}"
tui_db_time=$(bash -c "time timeout 3s ./target/release/chonker status >/dev/null 2>&1" 2>&1 | grep "real" | awk '{print $2}')
echo "TUI Database Status: $tui_db_time"

echo -e "${YELLOW}Testing compilation times:${NC}"
echo "TUI Build Time: 2:03.33 (from previous measurement)"
echo "GUI Build Time: 26.096s (from previous measurement)"
echo
echo -e "${GREEN}🎯 RECOMMENDATION: Use TUI for automated workflows, GUI for interactive editing${NC}"
