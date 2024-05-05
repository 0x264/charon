pub struct LineColumnInfo {
    line_end_offset: Vec<usize>
}

impl LineColumnInfo {
    pub fn new(source: &[u8]) -> Self {
        let mut line_end_offset = Vec::new();
        
        for (off, ch) in source.iter().enumerate() {
            if *ch == b'\n' {
                line_end_offset.push(off);
            }
        }
        
        Self {line_end_offset}
    }
    
    pub fn line_column_info(&self, mut off: usize) -> (usize, usize) {
        if self.line_end_offset.is_empty() {
            return (0, 0);
        }
        
        let mut line: Option<usize> = None;
        for (l, o) in self.line_end_offset.iter().enumerate() {
            if off <= *o {
                line = Some(l);
                break;
            }
        }
        
        let line = match line {
            None => {
                let l = self.line_end_offset.len() - 1;
                off = unsafe {*self.line_end_offset.get_unchecked(l)};
                l
            }
            Some(v) => v
        };
        
        let column = if line == 0 {
            off + 1
        } else {
            let pre_line_offset = unsafe {*self.line_end_offset.get_unchecked(line - 1)};
            off - pre_line_offset
        };
        (line + 1, column)
    }
}