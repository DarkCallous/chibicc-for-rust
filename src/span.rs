#[derive(Clone, Debug)]
pub struct Span<T> {
    pub item: T,
    pub pos: usize,   
    pub len: usize, 
}

impl<T> Span<T>{
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