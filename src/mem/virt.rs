use mem;

use std::{error,fmt};

struct Segment<'a> {
    base : u64,
    memory : Box<mem::Memory+'a>
}

impl<'a> Segment<'a> {
    fn new(base: u64, memory: Box<mem::Memory+'a>) -> Segment<'a> {
        Segment {
            base: base,
            memory: memory
        }
    }

    fn has_addr(&self, addr: u64) -> bool {
        addr >= self.base && addr < (self.base + self.memory.len())
    }
}

impl<'a> fmt::Debug for Segment<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_fmt(format_args!("${:04X} - ${:04X}", self.base, self.base + self.memory.len() - 1))
    }
}

/// Represents an error that can occur during a virtual memory management operation
#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum Error {
    /// Indicates that a memory overlaps with another memory in the virtual memory
    MemoryOverlap
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::MemoryOverlap => "attempted to attach a memory in a location that would overlap with another memory"
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;
        self.description().fmt(fmt)
    }
}

/// Provides an implementation of `mem::Memory` over a list of memories by performing
/// the memory operation on the memory that is mapped at the specified base address
///
/// Warning: Memories may NOT overlap
pub struct Virtual<'a> {
    segments : Vec<Segment<'a>>
}

impl<'a> Virtual<'a> {
    /// Constructs a new Virtual Memory with no member segments
    pub fn new() -> Virtual<'a> {
        Virtual {
            segments: Vec::new()
        }
    }

    /// Attaches another memory to the virtual memory
    ///
    /// # Arguments
    /// * `base` - The address to use as the base for the specified memory
    /// * `mem` - The memory to attach.
    pub fn attach(&mut self, base: u64, mem: Box<mem::Memory+'a>) -> Result<(), Error> {
        // Find the appropriate place to attach the memory
        let new_segment = Segment::new(base, mem);
        let pos = self.segments.iter()
            .position(|l| l.base > new_segment.base);

        let insert_point = match pos {
            None => self.segments.len(),
            Some(x) => x
        };

        if insert_point > 0 {
            // Check the memory on the left
            let left = &self.segments[insert_point - 1];
            if left.base + left.memory.len() - 1 >= base {
                return Err(Error::MemoryOverlap)
            }
        }

        if insert_point < self.segments.len() {
            // Check the memory on the right
            let right = &self.segments[insert_point];
            if base + new_segment.memory.len() - 1 >= right.base {
                return Err(Error::MemoryOverlap)
            }
        }

        self.segments.insert(insert_point, new_segment);
        Ok(())
    }

    fn find(&self, addr: u64) -> Option<&Segment<'a>> {
        self.segments.iter().find(|l| l.has_addr(addr))
    }

    fn find_mut(&mut self, addr: u64) -> Option<&mut Segment<'a>> {
        self.segments.iter_mut().find(|l| l.has_addr(addr))
    }
}

impl<'a> fmt::Debug for Virtual<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.segments.iter().fold(&mut fmt.debug_list(), |b, e| b.entry(e)).finish()
    }
}

impl<'a> mem::Memory for Virtual<'a> {
    fn len(&self) -> u64 {
        unimplemented!()
    }

    fn get_u8(&self, addr: u64) -> mem::Result<u8> {
        // Find the memory at the current address
        match self.find(addr) {
            Some(segment) => {
                let eaddr = addr - segment.base;
                segment.memory.get_u8(eaddr)
            },
            None => Err(mem::Error::with_detail(
                mem::ErrorKind::OutOfBounds,
                "Unable to locate a suitable memory segment",
                format!("at address: 0x{:X}", addr)))
        }
    }

    fn set_u8(&mut self, addr: u64, val: u8) -> mem::Result<()> {
        // Find the memory at the current address
        match self.find_mut(addr) {
            Some(segment) => {
                let eaddr = addr - segment.base;
                segment.memory.set_u8(eaddr, val)
            },
            None => Err(mem::Error::with_detail(
                mem::ErrorKind::OutOfBounds,
                "Unable to locate a suitable memory segment",
                format!("at address: 0x{:X}", addr)))
        }
    }
}

#[cfg(test)]
mod test {
    use mem;
    use mem::Memory;

    #[test]
    pub fn attach_with_no_items() {
        let mem = mem::Fixed::new(10);
        let mut vm = mem::Virtual::new();
        vm.attach(1000, Box::new(mem)).unwrap();
        assert_eq!(vm.segments.len(), 1);
        assert_eq!(vm.segments[0].base, 1000);
    }

    #[test]
    pub fn attach_at_end() {
        let mem1 = mem::Fixed::new(10);
        let mem2 = mem::Fixed::new(10);
        let mut vm = mem::Virtual::new();
        vm.attach(1000, Box::new(mem1)).unwrap();
        vm.attach(1010, Box::new(mem2)).unwrap();
        assert_eq!(vm.segments.len(), 2);
        assert_eq!(vm.segments[0].base, 1000);
        assert_eq!(vm.segments[1].base, 1010);
    }

    #[test]
    pub fn attach_at_end_with_overlap() {
        let mem1 = mem::Fixed::new(10);
        let mem2 = mem::Fixed::new(10);
        let mut vm = mem::Virtual::new();
        vm.attach(1000, Box::new(mem1)).unwrap();
        assert_eq!(
            vm.attach(1005, Box::new(mem2)),
            Err(mem::virt::Error::MemoryOverlap));
    }

    #[test]
    pub fn attach_at_beginning() {
        let mem1 = mem::Fixed::new(10);
        let mem2 = mem::Fixed::new(10);
        let mut vm = mem::Virtual::new();
        vm.attach(1010, Box::new(mem1)).unwrap();
        vm.attach(1000, Box::new(mem2)).unwrap();
        assert_eq!(vm.segments.len(), 2);
        assert_eq!(vm.segments[0].base, 1000);
        assert_eq!(vm.segments[1].base, 1010);
    }

    #[test]
    pub fn attach_at_beginning_with_overlap() {
        let mem1 = mem::Fixed::new(10);
        let mem2 = mem::Fixed::new(10);
        let mut vm = mem::Virtual::new();
        vm.attach(0x1005, Box::new(mem1)).unwrap();
        assert_eq!(
            vm.attach(0x1000, Box::new(mem2)),
            Err(mem::virt::Error::MemoryOverlap));
    }

    #[test]
    pub fn attach_in_middle() {
        let mem1 = mem::Fixed::new(10);
        let mem2 = mem::Fixed::new(10);
        let mem3 = mem::Fixed::new(10);
        let mut vm = mem::Virtual::new();
        vm.attach(1000, Box::new(mem1)).unwrap();
        vm.attach(1020, Box::new(mem2)).unwrap();
        vm.attach(1010, Box::new(mem3)).unwrap();
        assert_eq!(vm.segments.len(), 3);
        assert_eq!(vm.segments[0].base, 1000);
        assert_eq!(vm.segments[1].base, 1010);
        assert_eq!(vm.segments[2].base, 1020);
    }

    #[test]
    pub fn attach_in_middle_with_overlap() {
        let mem1 = mem::Fixed::new(10);
        let mem2 = mem::Fixed::new(10);
        let mem3 = mem::Fixed::new(10);
        let mut vm = mem::Virtual::new();
        vm.attach(1000, Box::new(mem1)).unwrap();
        vm.attach(1010, Box::new(mem2)).unwrap();
        assert_eq!(
            vm.attach(1005, Box::new(mem3)),
            Err(mem::virt::Error::MemoryOverlap));
    }

    #[test]
    pub fn get_from_single_memory() {
        let mut mem1 = mem::Fixed::new(10);
        let mut mem2 = mem::Fixed::new(10);
        let mut vm = mem::Virtual::new();

        mem1.set(0, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).unwrap();
        mem2.set(0, &[11, 12, 13, 14, 15, 16, 17, 18, 19, 20]).unwrap();

        vm.attach(1000, Box::new(mem1)).unwrap();
        vm.attach(1010, Box::new(mem2)).unwrap();

        let mut buf = [0, 0, 0, 0];
        vm.get(1006, &mut buf).unwrap();
        assert_eq!([7, 8, 9, 10], buf);
    }

    #[test]
    pub fn get_spanning_memories() {
        let mut mem1 = mem::Fixed::new(10);
        let mut mem2 = mem::Fixed::new(10);
        let mut vm = mem::Virtual::new();

        mem1.set(0, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).unwrap();
        mem2.set(0, &[11, 12, 13, 14, 15, 16, 17, 18, 19, 20]).unwrap();

        vm.attach(1000, Box::new(mem1)).unwrap();
        vm.attach(1010, Box::new(mem2)).unwrap();

        let mut buf = [0, 0, 0, 0];
        vm.get(1008, &mut buf).unwrap();
        assert_eq!([9, 10, 11, 12], buf);
    }

    #[test]
    pub fn set_to_single_memory() {
        let mut mem1 = mem::Fixed::new(10);
        let mut mem2 = mem::Fixed::new(10);
        let mut vm = mem::Virtual::new();

        mem1.set(0, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).unwrap();
        mem2.set(0, &[11, 12, 13, 14, 15, 16, 17, 18, 19, 20]).unwrap();

        vm.attach(1000, Box::new(mem1)).unwrap();
        vm.attach(1010, Box::new(mem2)).unwrap();

        vm.set(1006, &[0xDE, 0xAD, 0xBE, 0xEF]).unwrap();

        let mut buf = [0, 0, 0, 0];
        vm.segments[0].memory.get(6, &mut buf).unwrap();
        assert_eq!([0xDE, 0xAD, 0xBE, 0xEF], buf);
    }

    #[test]
    pub fn set_spanning_memories() {
        let mut mem1 = mem::Fixed::new(10);
        let mut mem2 = mem::Fixed::new(10);
        let mut vm = mem::Virtual::new();

        mem1.set(0, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).unwrap();
        mem2.set(0, &[11, 12, 13, 14, 15, 16, 17, 18, 19, 20]).unwrap();

        vm.attach(1000, Box::new(mem1)).unwrap();
        vm.attach(1010, Box::new(mem2)).unwrap();

        vm.set(1008, &[0xDE, 0xAD, 0xBE, 0xEF]).unwrap();

        let mut buf = [0, 0, 0, 0];
        vm.segments[0].memory.get(8, &mut buf[0..2]).unwrap();
        vm.segments[1].memory.get(0, &mut buf[2..4]).unwrap();
        assert_eq!([0xDE, 0xAD, 0xBE, 0xEF], buf);
    }
}
