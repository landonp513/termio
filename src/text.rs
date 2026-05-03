pub fn char_to_byte(line: &str, char_idx: usize) -> usize {
    line.char_indices()
        .nth(char_idx)
        .map(|(byte_idx, _)| byte_idx)
        .unwrap_or(line.len())
}

pub fn byte_to_char(line: &str, byte_idx: usize) -> usize {
    line[..byte_idx].chars().count()
}

pub fn byte_offset_search(line: &str, char_start: usize, query: &str) -> Option<usize> {
    let byte_start = char_to_byte(line, char_start);
    line[byte_start..].find(query).map(|relative| relative + byte_start)
}
