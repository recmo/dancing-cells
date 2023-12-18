/// Sparse Sets
/// https://dl.acm.org/doi/pdf/10.1145/176454.176484
// TODO: Generalize over index int type. Maybe add u24, u40, u48 as well.
// TODO: One of the unique features of this datastructure is that is safe to run on uninitialized memory, can we tell rust that?
#[derive(Debug)]
pub struct SparseSet {
    domain: Vec<usize>,
    inverse: Vec<usize>,
    size: usize,
}

// TODO: Undelete
impl SparseSet {
    /// Constructs a new set containing all the elements 0..size
    pub fn new(size: usize) -> Self {
        SparseSet {
            domain: (0..size).collect(),
            inverse: (0..size).collect(),
            size,
        }
    }

    pub fn contains(&self, element: usize) -> bool {
        assert!(element < self.domain.len());
        self.inverse[element] < self.size
    }

    pub fn iter(&self) -> std::slice::Iter<usize> {
        self.domain[..self.size].iter()
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn remove(&mut self, element: usize) -> bool {
        assert!(element < self.domain.len());
        let index = self.inverse[element];
        if index < self.size {
            self.size -= 1;
            self.swap(index, self.size);
            true
        } else {
            false
        }
    }

    pub fn undo(&mut self) {
        assert!(self.size < self.domain.len());
        self.size += 1;
    }

    // Swap two elements in the domain and adjust inverse.
    fn swap(&mut self, a: usize, b: usize) {
        self.domain.swap(a, b);
        self.inverse.swap(self.domain[a], self.domain[b]);
    }

    fn check_consistency(&self) {
        // `domain` and `inverse` are inverse permutations.
        for i in 0..self.size {
            assert!(self.inverse[self.domain[i]] == i);
        }
    }
}
