#[repr(C)]
pub struct SmallArray<const N: usize, T> {
    pub data: [T; N],
    pub count: usize,
}

impl<const N: usize, T> Default for SmallArray<N, T> {
    fn default() -> Self {
        Self {
            data: std::array::from_fn(|_| unsafe { std::mem::zeroed() }),
            count: 0,
        }
    }
}

impl<const N: usize, T> SmallArray<N, T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, elem: T) {
        assert!(self.count < N);
        self.data[self.count] = elem;
        self.count += 1;
    }

    pub fn pop(&mut self) -> T {
        assert!(self.count > 0);
        let elem = std::mem::replace(&mut self.data[self.count], unsafe { std::mem::zeroed() });
        self.count -= 1;
        elem
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.count = 0;
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.count
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

use std::ops::Index;

impl<const N: usize, T> Index<usize> for SmallArray<N, T> {
    type Output = T;

    fn index(&self, i: usize) -> &Self::Output {
        &self.data[i]
    }
}

use std::ops::IndexMut;

impl<const N: usize, T> IndexMut<usize> for SmallArray<N, T> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.data[i]
    }
}

use std::ops::Deref;

impl<const N: usize, T> Deref for SmallArray<N, T> {
    type Target = [T; N];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

use std::ops::DerefMut;

impl<const N: usize, T> DerefMut for SmallArray<N, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
