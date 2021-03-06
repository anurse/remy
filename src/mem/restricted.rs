use mem;

pub struct ReadOnlyMemory<M>(M) where M: mem::Memory;

impl<M> mem::Memory for ReadOnlyMemory<M> where M: mem::Memory {
    fn len(&self) -> u64 {
        let &ReadOnlyMemory(ref m) = self;
        m.len()
    }

    fn get_u8(&self, addr: u64) -> mem::Result<u8> {
        let &ReadOnlyMemory(ref m) = self;
        m.get_u8(addr)
    }

    #[allow(unused_variables)]
    fn set_u8(&mut self, addr: u64, val: u8) -> mem::Result<()> {
        Err(mem::Error::new(mem::ErrorKind::MemoryNotWritable, "attempted to write to read-only memory"))
    }
}

pub fn read_only<M>(inner: M) -> ReadOnlyMemory<M> where M: mem::Memory {
    ReadOnlyMemory(inner)
}

pub struct WriteOnlyMemory<M>(M) where M: mem::Memory;

impl<M> mem::Memory for WriteOnlyMemory<M> where M: mem::Memory {
    fn len(&self) -> u64 {
        let &WriteOnlyMemory(ref m) = self;
        m.len()
    }

    #[allow(unused_variables)]
    fn get_u8(&self, addr: u64) -> mem::Result<u8> {
        Err(mem::Error::new(mem::ErrorKind::MemoryNotReadable, "attempted to read from write-only memory"))
    }

    fn set_u8(&mut self, addr: u64, val: u8) -> mem::Result<()> {
        let &mut WriteOnlyMemory(ref mut m) = self;
        m.set_u8(addr, val)
    }
}

pub fn write_only<M>(inner: M) -> WriteOnlyMemory<M> where M: mem::Memory {
    WriteOnlyMemory(inner)
}
