#[derive(Debug, Clone, Default)]
pub struct LibraryOptions {
    is_static: bool,
}

impl LibraryOptions {
    pub fn is_static(&self) -> bool {
        self.is_static
    }

    pub fn be_static(&mut self, is_static: bool) {
        self.is_static = is_static
    }
}
