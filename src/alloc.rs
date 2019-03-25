use std::mem;
use std::cell::UnsafeCell;

pub struct Arena {
    data: UnsafeCell<Vec<Vec<u8>>>,
}

impl Arena {
    pub fn with_capacity(capacity: usize) -> Arena {
        Arena {
            data: UnsafeCell::new(vec![Vec::with_capacity(capacity)]),
        }
    }

    fn alloc_bytes(&self, bytes: usize, align: usize) -> *mut u8 {
        let mut data = unsafe { &mut *self.data.get() };
        let (ptr, len, capacity) = {
            let last = &data.last().unwrap();
            let len = last.len();
            (last.as_ptr() as usize + len, len, last.capacity())
        };
        let offset = ((ptr + align - 1) & !(align - 1)) - ptr;

        if len + offset + bytes > capacity {
            let size = capacity.max(bytes);
            data.push(Vec::with_capacity(size.saturating_add(size)));
        }

        let last = &mut data.last_mut().unwrap();
        unsafe {
            last.set_len(len + offset + bytes);
            last.as_mut_ptr().offset((len + offset) as isize)
        }
    }

    pub fn alloc<'a, T: Copy>(&'a self, value: T) -> &'a mut T {
        let ptr = self.alloc_bytes(mem::size_of::<T>(), mem::align_of::<T>());
        let result: &'a mut T = unsafe { &mut *(ptr as *mut T) };
        *result = value;
        result
    }

    pub fn alloc_slice<'a, T: Copy>(&'a self, values: &[T]) -> &'a mut [T] {
        let ptr = self.alloc_bytes(mem::size_of::<T>() * values.len(), mem::align_of::<T>());
        let result: &'a mut [T] = unsafe { std::slice::from_raw_parts_mut(ptr as *mut T, values.len()) };
        result.copy_from_slice(values);
        result
    }
}

#[test]
fn test_overflow() {
    let mut arena = Arena::with_capacity(4);

    let x: &mut u64 = arena.alloc(3);
    assert_eq!(*x, 3);
    assert_eq!(arena.data.len(), 2);
}

#[test]
fn test_alignment() {
    let mut arena = Arena::with_capacity(1024);

    let x: &mut u8 = arena.alloc(3);
    assert_eq!(x as *mut u8 as usize % mem::align_of::<u8>(), 0);

    let x: &mut u16 = arena.alloc(3);
    assert_eq!(x as *mut u16 as usize % mem::align_of::<u16>(), 0);

    let x: &mut u32 = arena.alloc(3);
    assert_eq!(x as *mut u32 as usize % mem::align_of::<u32>(), 0);

    let x: &mut u64 = arena.alloc(3);
    assert_eq!(x as *mut u64 as usize % mem::align_of::<u64>(), 0);
}

#[test]
fn test_slice() {
    let mut arena = Arena::with_capacity(1024);

    let xs: [u32; 16] = [0; 16];
    let ys = arena.alloc_slice(&xs);
    assert_eq!(xs, ys);
}


pub struct Slab<T> {
    next: usize,
    entries: Vec<Entry<T>>,
}

enum Entry<T> {
    Empty(usize),
    Value(T),
}

impl<T> Slab<T> {
    pub fn new() -> Slab<T> {
        Slab {
            next: 0,
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, value: T) -> usize {
        let index = self.next;
        if index == self.entries.len() {
            self.entries.push(Entry::Value(value));
            self.next = self.entries.len();
        } else {
            if let Entry::Empty(next) = self.entries[index] {
                self.next = next;
                self.entries[index] = Entry::Value(value);
            } else {
                unreachable!()
            }
        }
        index
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if let Some(entry @ Entry::Value(_)) = self.entries.get_mut(index) {
            if let Entry::Value(value) = std::mem::replace(entry, Entry::Empty(self.next)) {
                self.next = index;
                Some(value)
            } else {
                unreachable!()
            }
        } else {
            None
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        match self.entries.get(index) {
            Some(Entry::Value(value)) => Some(value),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        match self.entries.get_mut(index) {
            Some(Entry::Value(value)) => Some(value),
            _ => None,
        }
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter { entries: self.entries.iter() }
    }

    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, T> {
        IterMut { entries: self.entries.iter_mut() }
    }
}

impl<T> IntoIterator for Slab<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        IntoIter { entries: self.entries.into_iter() }
    }
}

pub struct IntoIter<T> {
    entries: std::vec::IntoIter<Entry<T>>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        while let Some(entry) = self.entries.next() {
            if let Entry::Value(value) = entry {
                return Some(value);
            }
        }
        None
    }
}

pub struct Iter<'a, T> {
    entries: std::slice::Iter<'a, Entry<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        while let Some(entry) = self.entries.next() {
            if let Entry::Value(value) = entry {
                return Some(value);
            }
        }
        None
    }
}

pub struct IterMut<'a, T> {
    entries: std::slice::IterMut<'a, Entry<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<&'a mut T> {
        while let Some(entry) = self.entries.next() {
            if let Entry::Value(value) = entry {
                return Some(value);
            }
        }
        None
    }
}
