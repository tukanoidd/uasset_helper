use std::path::Path;

pub struct SplitVecContainer<L, R> {
    pub left: Vec<L>,
    pub right: Vec<R>,
}

impl<L, R> SplitVecContainer<L, R> {
    pub fn new(left: Vec<L>, right: Vec<R>) -> Self {
        Self { left, right }
    }
}

impl<L, R> Default for SplitVecContainer<L, R> {
    fn default() -> Self {
        Self {
            left: vec![],
            right: vec![],
        }
    }
}

impl<L, R> SplitVecContainer<L, R> {
    #[inline]
    pub fn push_left(&mut self, left: L) {
        self.left.push(left);
    }

    #[inline]
    pub fn push_right(&mut self, right: R) {
        self.right.push(right);
    }
}

impl<L, R> From<SplitVecContainer<L, R>> for (Vec<L>, Vec<R>) {
    fn from(container: SplitVecContainer<L, R>) -> Self {
        (container.left, container.right)
    }
}

#[derive(Debug, Clone)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    #[inline]
    pub fn toggle(&mut self) {
        *self = self.toggled();
    }

    pub fn toggled(&self) -> Self {
        match self {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        }
    }

    #[allow(dead_code)]
    pub fn is_ascending(&self) -> bool {
        matches!(self, Self::Ascending)
    }

    #[allow(dead_code)]
    pub fn is_descending(&self) -> bool {
        matches!(self, Self::Descending)
    }
}

pub fn path_to_str(path: impl AsRef<Path>) -> String {
    path.as_ref().to_str().unwrap().to_string()
}
