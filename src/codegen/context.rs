// pub struct ProgContext {

// }

// impl ProgContext {
//     pub fn new() -> ProgContext {
//         ProgContext {  }
//     }
// }

pub struct FnContext {
    pub name: String,
    pub label_cnt: usize,
}

impl FnContext {
    pub fn new(name: String) -> FnContext {
        FnContext { name, label_cnt: 0 }
    }

    pub fn apply(&mut self) -> usize {
        let result = self.label_cnt;
        self.label_cnt += 1;
        result
    }
}
