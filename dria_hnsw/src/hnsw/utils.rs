use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap; //node_metadata
use std::collections::HashMap;

#[derive(PartialEq, Debug)]
pub struct Numeric(pub f32);

impl Eq for Numeric {}

impl PartialOrd for Numeric {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for Numeric {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(Ordering::Equal)
    }
}

pub trait IntoMap<Numeric> {
    fn into_map(self) -> HashMap<u32, f32>;
}

impl IntoMap<Numeric> for BinaryHeap<(Numeric, u32)> {
    fn into_map(self) -> HashMap<u32, f32> {
        self.into_iter().map(|(d, i)| (i, d.0)).collect()
    }
}

impl IntoMap<Numeric> for BinaryHeap<Reverse<(Numeric, u32)>> {
    fn into_map(self) -> HashMap<u32, f32> {
        self.into_iter().map(|Reverse((d, i))| (i, d.0)).collect()
    }
}

pub trait IntoHeap<Numeric> {
    fn into_maxheap(self) -> BinaryHeap<(Numeric, u32)>;
    fn into_minheap(self) -> BinaryHeap<Reverse<(Numeric, u32)>>;
}

impl IntoHeap<Numeric> for HashMap<u32, f32> {
    fn into_maxheap(self) -> BinaryHeap<(Numeric, u32)> {
        self.into_iter().map(|(i, d)| (Numeric(d), i)).collect()
    }

    fn into_minheap(self) -> BinaryHeap<Reverse<(Numeric, u32)>> {
        self.into_iter()
            .map(|(i, d)| Reverse((Numeric(d), i)))
            .collect()
    }
}

impl IntoHeap<Numeric> for Vec<(f32, u32)> {
    fn into_maxheap(self) -> BinaryHeap<(Numeric, u32)> {
        self.into_iter().map(|(d, i)| (Numeric(d), i)).collect()
    }

    fn into_minheap(self) -> BinaryHeap<Reverse<(Numeric, u32)>> {
        self.into_iter()
            .map(|(d, i)| Reverse((Numeric(d), i)))
            .collect()
    }
}

pub fn create_min_heap() -> BinaryHeap<Reverse<(Numeric, u32)>> {
    let q: BinaryHeap<Reverse<(Numeric, u32)>> = BinaryHeap::new();
    q
}

pub fn create_max_heap() -> BinaryHeap<(Numeric, u32)> {
    let q: BinaryHeap<(Numeric, u32)> = BinaryHeap::new();
    q
}
