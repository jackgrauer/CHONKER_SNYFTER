use std::collections::HashMap;

pub struct SmartTableDetector {
    confidence_threshold: f32,
}

impl SmartTableDetector {
    pub fn new() -> Self {
        Self {
            confidence_threshold: 0.7,
        }
    }
    
    pub fn detect_tables_smart(&self, content: &str) -> Vec<TableRegion> {
        let lines: Vec<&str> = content.lines().collect();
        let mut regions = Vec::new();
        let mut current_region: Option<TableRegionBuilder> = None;
        
        for (i, line) in lines.iter().enumerate() {
            let confidence = self.calculate_table_confidence(line, i, &lines);
            
            if confidence > self.confidence_threshold {
                if current_region.is_none() {
                    current_region = Some(TableRegionBuilder::new(i));
                }
                
                if let Some(ref mut region) = current_region {
                    region.add_line(i, line, confidence);
                }
            } else if let Some(region) = current_region.take() {
                if let Some(table) = region.build() {
                    regions.push(table);
                }
            }
        }
        
        // Don't forget last region
        if let Some(region) = current_region {
            if let Some(table) = region.build() {
                regions.push(table);
            }
        }
        
        regions
    }
    
    fn calculate_table_confidence(&self, line: &str, index: usize, all_lines: &[&str]) -> f32 {
        let mut score = 0.0;
        let mut factors = 0;
        
        // Factor 1: Presence of delimiters
        let delimiter_score = self.score_delimiters(line);
        score += delimiter_score * 0.3;
        factors += 1;
        
        // Factor 2: Structural consistency with nearby lines
        let consistency_score = self.score_consistency(line, index, all_lines);
        score += consistency_score * 0.4;
        factors += 1;
        
        // Factor 3: Content pattern (looks like data)
        let content_score = self.score_content_pattern(line);
        score += content_score * 0.3;
        factors += 1;
        
        score / factors as f32
    }
    
    fn score_delimiters(&self, line: &str) -> f32 {
        let pipe_count = line.chars().filter(|&c| c == '|').count();
        let tab_count = line.chars().filter(|&c| c == '\t').count();
        let multi_space_count = self.count_multi_spaces(line);
        
        // Score based on delimiter presence
        let delimiter_total = pipe_count + tab_count + multi_space_count;
        
        if delimiter_total == 0 {
            0.0
        } else if delimiter_total < 2 {
            0.3
        } else if delimiter_total < 4 {
            0.7
        } else {
            1.0
        }
    }
    
    fn count_multi_spaces(&self, line: &str) -> usize {
        let mut count = 0;
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            if chars[i] == ' ' {
                let mut space_run = 1;
                while i + space_run < chars.len() && chars[i + space_run] == ' ' {
                    space_run += 1;
                }
                
                if space_run >= 3 {  // 3+ spaces might be column delimiter
                    count += 1;
                }
                
                i += space_run;
            } else {
                i += 1;
            }
        }
        
        count
    }
    
    fn score_consistency(&self, line: &str, index: usize, all_lines: &[&str]) -> f32 {
        let window = 2;  // Look at 2 lines above and below
        let mut similar_lines = 0;
        let mut total_compared = 0;
        
        for offset in -window..=window {
            if offset == 0 { continue; }
            
            let check_index = index as i32 + offset;
            if check_index >= 0 && (check_index as usize) < all_lines.len() {
                total_compared += 1;
                
                let other_line = all_lines[check_index as usize];
                if self.structurally_similar(line, other_line) {
                    similar_lines += 1;
                }
            }
        }
        
        if total_compared == 0 {
            0.5  // Neutral score if no comparison possible
        } else {
            similar_lines as f32 / total_compared as f32
        }
    }
    
    fn structurally_similar(&self, line1: &str, line2: &str) -> bool {
        // Extract "segments" (parts between delimiters)
        let segments1 = self.extract_segments(line1);
        let segments2 = self.extract_segments(line2);
        
        // Similar number of segments
        if (segments1.len() as i32 - segments2.len() as i32).abs() > 1 {
            return false;
        }
        
        // Similar segment positions
        let positions1 = self.get_segment_positions(&segments1);
        let positions2 = self.get_segment_positions(&segments2);
        
        self.positions_similar(&positions1, &positions2)
    }
    
    fn extract_segments(&self, line: &str) -> Vec<String> {
        // Split by various delimiters
        let mut segments = Vec::new();
        let mut current_segment = String::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            match chars[i] {
                '|' | '\t' => {
                    if !current_segment.trim().is_empty() {
                        segments.push(current_segment.trim().to_string());
                    }
                    current_segment.clear();
                    i += 1;
                }
                ' ' if i + 2 < chars.len() && chars[i+1] == ' ' && chars[i+2] == ' ' => {
                    // Multiple spaces as delimiter
                    if !current_segment.trim().is_empty() {
                        segments.push(current_segment.trim().to_string());
                    }
                    current_segment.clear();
                    i += 3;
                }
                _ => {
                    current_segment.push(chars[i]);
                    i += 1;
                }
            }
        }
        
        if !current_segment.trim().is_empty() {
            segments.push(current_segment.trim().to_string());
        }
        
        segments
    }
    
    fn get_segment_positions(&self, segments: &[String]) -> Vec<usize> {
        let mut positions = Vec::new();
        let mut current_pos = 0;
        
        for segment in segments {
            positions.push(current_pos);
            current_pos += segment.len() + 3;  // Rough estimate
        }
        
        positions
    }
    
    fn positions_similar(&self, pos1: &[usize], pos2: &[usize]) -> bool {
        if pos1.len() != pos2.len() {
            return false;
        }
        
        let max_deviation = 5;
        for (p1, p2) in pos1.iter().zip(pos2.iter()) {
            if (*p1 as i32 - *p2 as i32).abs() > max_deviation {
                return false;
            }
        }
        
        true
    }
    
    fn score_content_pattern(&self, line: &str) -> f32 {
        // Look for patterns that suggest tabular data
        let segments = self.extract_segments(line);
        
        if segments.is_empty() {
            return 0.0;
        }
        
        let mut pattern_score = 0.0;
        
        // Short segments (typical of table cells)
        let avg_length = segments.iter().map(|s| s.len()).sum::<usize>() / segments.len();
        if avg_length < 20 {
            pattern_score += 0.3;
        }
        
        // Numeric content
        let numeric_segments = segments.iter()
            .filter(|s| self.looks_numeric(s))
            .count();
        pattern_score += (numeric_segments as f32 / segments.len() as f32) * 0.3;
        
        // Consistent segment types
        if self.segments_have_consistent_types(&segments) {
            pattern_score += 0.4;
        }
        
        pattern_score.min(1.0)
    }
    
    fn looks_numeric(&self, segment: &str) -> bool {
        segment.chars().any(|c| c.is_numeric()) &&
        segment.chars().filter(|c| c.is_numeric()).count() > segment.len() / 3
    }
    
    fn segments_have_consistent_types(&self, segments: &[String]) -> bool {
        // This is a simplified check - you could make it more sophisticated
        if segments.len() < 2 {
            return true;
        }
        
        // Check if segments in same positions across rows tend to have similar types
        true  // Simplified for this example
    }
}

pub struct TableRegion {
    pub start_line: usize,
    pub end_line: usize,
    pub lines: Vec<String>,
    pub confidence: f32,
}

struct TableRegionBuilder {
    start_line: usize,
    lines: Vec<(String, f32)>,
}

impl TableRegionBuilder {
    fn new(start_line: usize) -> Self {
        Self {
            start_line,
            lines: Vec::new(),
        }
    }
    
    fn add_line(&mut self, _line_num: usize, line: &str, confidence: f32) {
        self.lines.push((line.to_string(), confidence));
    }
    
    fn build(self) -> Option<TableRegion> {
        if self.lines.is_empty() {
            return None;
        }
        
        let avg_confidence = self.lines.iter()
            .map(|(_, conf)| conf)
            .sum::<f32>() / self.lines.len() as f32;
        
        Some(TableRegion {
            start_line: self.start_line,
            end_line: self.start_line + self.lines.len() - 1,
            lines: self.lines.into_iter().map(|(line, _)| line).collect(),
            confidence: avg_confidence,
        })
    }
}
