pub struct ProgContext {
    label_cnt: usize,
}

impl ProgContext {
    pub fn new() -> ProgContext {
        ProgContext { label_cnt: 0 }
    }

    pub fn apply(&mut self) -> usize {
        let result = self.label_cnt;
        self.label_cnt += 1;
        result
    }
}
