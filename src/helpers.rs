pub(crate) fn find_block_end(text: &str) -> Option<usize> {
    let mut stack = 0;
    for (index, char) in text.char_indices() {
        if char == '(' {
            stack += 1;
        }
        if char == ')' {
            stack -= 1;
            assert!(stack >= 0);
            if stack == 0 {
                return Some(index);
            }
        }
    }
    None
}
