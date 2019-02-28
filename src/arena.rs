use std::mem;

pub struct Arena {
    data: Vec<Vec<u8>>,
}

impl Arena {
    pub fn with_capacity(capacity: usize) -> Arena {
        Arena {
            data: vec![Vec::with_capacity(capacity)],
        }
    }

    fn alloc_bytes(&mut self, bytes: usize, align: usize) -> *mut u8 {
        let (ptr, len, capacity) = {
            let last = &self.data.last().unwrap();
            let len = last.len();
            (last.as_ptr() as usize + len, len, last.capacity())
        };
        let offset = ((ptr + align - 1) & !(align - 1)) - ptr;

        if len + offset + bytes > capacity {
            let size = capacity.max(bytes);
            self.data.push(Vec::with_capacity(size.saturating_add(size)));
        }

        let last = &mut self.data.last_mut().unwrap();
        unsafe {
            last.set_len(len + offset + bytes);
            last.as_mut_ptr().offset((len + offset) as isize)
        }
    }

    pub fn alloc<'a, T: Copy>(&mut self, value: T) -> &'a mut T where Self: 'a {
        let ptr = self.alloc_bytes(mem::size_of::<T>(), mem::align_of::<T>());
        let result: &'a mut T = unsafe { &mut *(ptr as *mut T) };
        *result = value;
        result
    }

    pub fn alloc_slice<'a, T: Copy>(&mut self, values: &[T]) -> &'a mut [T] where Self: 'a {
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
