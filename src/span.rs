#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Span {
    pub pos: usize,   
    pub len: usize, 
}

impl Span{
    pub fn start(&self) -> usize{
        self.pos
    }
    pub fn end(&self) -> usize{
        self.pos + self.len
    }
    pub fn text<'a>(&'a self, source: &'a str) -> &'a str{
        &source[self.pos..self.end()]
    }
}