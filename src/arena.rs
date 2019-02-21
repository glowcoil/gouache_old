use std::mem;

pub struct Arena {
    data: Vec<Vec<u8>>,
    next_size: usize,
}

impl Arena {
    pub fn with_capacity(capacity: usize) -> Arena {
        Arena {
            data: vec![Vec::with_capacity(capacity)],
            next_size: capacity,
        }
    }

    pub fn alloc<'a, T: Copy>(&mut self, value: T) -> &'a mut T where Self: 'a {
        let bytes = mem::size_of::<T>();
        let align = mem::align_of::<T>();
        let (ptr, len, capacity) = {
            let last = &self.data.last().unwrap();
            let len = last.len();
            (last.as_ptr() as usize + len, len, last.capacity())
        };
        let offset = ((ptr + align - 1) & !(align - 1)) - ptr;

        let result_ptr = if len + offset + bytes > capacity {
            let mut exact: Vec<T> = Vec::with_capacity(1);
            let (exact_ptr, exact_len, exact_cap) = (exact.as_mut_ptr(), exact.len(), exact.capacity());
            mem::forget(exact);
            let mut exact_bytes: Vec<u8> = unsafe { Vec::from_raw_parts(exact_ptr as *mut u8, exact_len * bytes, exact_cap * bytes) };
            let result = exact_bytes.as_mut_ptr();
            self.data.push(exact_bytes);

            self.next_size = self.next_size.saturating_add(self.next_size);
            self.data.push(Vec::with_capacity(self.next_size));

            result
        } else {
            let last = &mut self.data.last_mut().unwrap();
            unsafe {
                last.set_len(len + offset + bytes);
                last.as_mut_ptr().offset((len + offset) as isize)
            }
        };

        let result: &'a mut T = unsafe { &mut *(result_ptr as *mut T) };
        *result = value;
        result
    }
}

#[test]
fn test_overflow() {
    let mut arena = Arena::with_capacity(4);

    let x: &mut u64 = arena.alloc(3);
    assert_eq!(*x, 3);
    assert_eq!(arena.data.len(), 3);
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
