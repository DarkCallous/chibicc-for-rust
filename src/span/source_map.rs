use std::path::PathBuf;

pub enum FileName {
    Real(PathBuf),
    Cli,
}

pub struct SourceFile {
    pub name: FileName,
    pub src: String,
    pub lines: Vec<usize>,
}

impl SourceFile {
    pub fn new(name: FileName, src: String) -> SourceFile {
        let mut file = SourceFile {
            name,
            src,
            lines: vec![0],
        };
        file.lines.extend(
            file.src
                .as_bytes()
                .iter()
                .enumerate()
                .filter_map(|(i, b)| (*b == b'\n').then_some(i + 1)),
        );
        file
    }

    pub fn lookup_line(&self, pos: usize) -> Option<usize> {
        self.lines.partition_point(|x| x < &pos).checked_sub(1)
    }

    pub fn lookup_line_column(&self, pos: usize) -> (usize, usize) {
        if let Some(line_id) = self.lookup_line(pos) {
            (line_id, pos - self.lines[line_id])
        } else {
            (0, pos)
        }
    }

    pub fn line_content(&self, line: usize) -> &str {
        let start = self.lines[line];
        let end = *self.lines.get(line + 1).unwrap_or(&self.src.len());
        &self.src[start..end]
    }
}

pub struct SourceMap {}
