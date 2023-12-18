#[derive(Debug)]
struct SparseSet {
    dense: Vec<usize>,
    sparse: Vec<usize>,
}

// TODO: Undelete
impl SparseSet {
    fn new(size: usize) -> Self {
        SparseSet {
            dense: Vec::with_capacity(size),
            sparse: vec![0; size],
        }
    }

    fn insert(&mut self, element: usize) {
        assert!(element < self.sparse.len());
        assert!(!self.contains(element));
        self.dense.push(element);
        self.sparse[element] = self.dense.len() - 1;
    }

    fn remove(&mut self, element: usize) {
        assert!(element < self.sparse.len());
        assert!(self.contains(element));

        let index = self.sparse[element];
        let last_index = self.dense.len() - 1;
        let last_element = self.dense[last_index];

        self.dense.swap(index, last_index);
        self.sparse[last_element] = index;
        self.dense.pop();
    }

    fn clear(&mut self) {
        self.dense.clear();
    }

    fn contains(&self, index: usize) -> bool {
        self.sparse[index] < self.dense.len() && self.dense[self.sparse[index]] == index
    }

    fn len(&self) -> usize {
        self.dense.len()
    }

    fn iter(&self) -> std::slice::Iter<usize> {
        self.dense.iter()
    }

    fn consistency_check(&self) {
        assert!(self.iter().all(|&i| self.contains(i)));
    }
}

impl IntoIterator for SparseSet {
    type Item = usize;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.dense.into_iter()
    }
}

fn main() {
    let mut ss = SparseSet::new(10);
    println!("contains 3: {}", ss.contains(3));
    for i in [2, 3, 5, 7] {
        ss.insert(i);
    }
    for i in ss.iter() {
        println!("{}", i);
    }
    println!("contains 3: {}", ss.contains(3));
    ss.remove(3);
    ss.consistency_check();
    println!("contains 3: {}", ss.contains(3));
    for i in ss.iter() {
        println!("{}", i);
    }
}
