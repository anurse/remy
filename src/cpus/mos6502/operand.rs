use std::{error,fmt};
use byteorder::LittleEndian;

use mem;
use mem::{Memory,MemoryExt};

use cpus::mos6502::cpu;
use cpus::mos6502::Mos6502;

/// Represents an operand that can be provided to an instruction
#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum Operand {
    /// Indicates an operand provided as an inline 8-bit unsigned integer
    Immediate(u8),
    /// Indicates an operand stored in the Accumulator (A register) 
    Accumulator,
    /// Indicates an operand stored at the provided memory address
    ///
    /// If the provided address is `m`, this operand is defined as `*m`
    Absolute(u16),
    /// Indicates an operand stored at the provided index from the current value of the provided
    /// register
    ///
    /// If the provided address is `m`, this operand is defined as `*(m+x)` or `*(m+y)` depending
    /// on the register specified
    Indexed(u16, cpu::RegisterName),
    /// Indicates an operand stored at an address stored in the provided address
    ///
    /// If the provided address is `m`, this operand is defined as `**m`
    Indirect(u16),
    /// Indicates an operand stored at an address stored in the provided address (indexed by the
    /// `X` register)
    ///
    /// If the provided address is `m`, this operand is defined as `**(m+x)`
    PreIndexedIndirect(u8),
    /// Indicates an operand stored at an address (indexed by the `Y` register) stored in the provided address
    ///
    /// If the provided address is `x`, this operand is defined as `*(*m+y)`
    PostIndexedIndirect(u8),

    // Only used in very specific instructions, always unwrapped directly rather than using the
    // member functions defined below

    /// Indicates an operand provided as an inline 8-bit signed offset to apply to the program counter
    Offset(i8),
    /// Indicates an operand provided as an inline 16-bit unsigned integer
    TwoByteImmediate(u16)
}

impl Operand {
    /// Retrieves the operand value
    ///
    /// # Arguments
    ///
    /// * `cpu` - The cpu from which to get the operand value
    pub fn get_u8<M>(&self, cpu: &Mos6502, mem: &M) -> Result<u8, Error> where M: mem::Memory {
        Ok(match self {
            &Operand::Immediate(n)      => n,
            &Operand::Accumulator       => cpu.registers.a,
            _                           => try!(mem.get_u8(try!(self.get_addr(cpu, mem)) as u64))
        })
    }

    /// Sets the value of the operand on the specified cpu
    ///
    /// # Arguments
    ///
    /// * `cpu` - The cpu on which to set the operand value
    /// * `val` - The value to set the operand to
    pub fn set_u8<M>(&self, cpu: &mut Mos6502, mem: &mut M, val: u8) -> Result<(), Error> where M: mem::Memory {
        match self {
            &Operand::Absolute(addr)     => Ok(try!(mem.set_u8(addr as u64, val))),
            &Operand::Indexed(addr, r)   => {
                let rv = r.get(cpu) as u64;
                Ok(try!(mem.set_u8(addr as u64 + rv, val)))
            }
            &Operand::Accumulator        => { cpu.registers.a = val; Ok(()) },
            _                            => Err(Error::ReadOnlyOperand)
        }
    }

    /// Retrieves the address of the operand on the specified cpu
    ///
    /// # Arguments
    ///
    /// * `cpu` - The cpu on which to get the operand value
    pub fn get_addr<M>(&self, cpu: &Mos6502, mem: &M) -> Result<u16, Error> where M: mem::Memory {
        Ok(match self {
            &Operand::Absolute(addr)             => addr,
            &Operand::Indirect(addr)             => try!(mem.get_u16::<LittleEndian>(addr as u64)),
            &Operand::Indexed(addr, r)           => addr + r.get(cpu) as u16,
            &Operand::PreIndexedIndirect(addr)   => try!(mem.get_u16::<LittleEndian>(addr as u64 + cpu.registers.x as u64)),
            &Operand::PostIndexedIndirect(addr)  => try!(mem.get_u16::<LittleEndian>(addr as u64)) + cpu.registers.y as u16,
            _                                   => return Err(Error::NonAddressOperand)
        })
    }
}

impl fmt::Display for Operand {
    /// Returns a string representing the instruction
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &Operand::Immediate(val)             => write!(formatter, "#${:02X}", val),
            &Operand::Accumulator                => formatter.write_str("A"),
            &Operand::Absolute(val)              =>
                if val <= 0x00FF {
                    write!(formatter, "${:02X}", val)
                } else {
                    write!(formatter, "${:04X}", val)
                },
            &Operand::Indexed(val, reg)          =>
                if val <= 0x00FF {
                    write!(formatter, "${:02X},{}", val, reg)
                } else {
                    write!(formatter, "${:04X},{}", val, reg)
                },
            &Operand::Indirect(val)              => write!(formatter, "${:04X}", val),
            &Operand::PreIndexedIndirect(val)    => write!(formatter, "(${:02X},X)", val),
            &Operand::PostIndexedIndirect(val)   => write!(formatter, "(${:02X}),Y", val)
        }
    }
}

/// Represents an error that occurred which accessing an `Operand`
#[derive(Clone,Debug,Eq,PartialEq)]
pub enum Error {
    /// Indicates an error occurred reading or writing memory
    ErrorAccessingMemory(mem::Error),
    /// Indicates that a request was made to write to a read-only operand such as
    /// `Operand::Immediate`
    ReadOnlyOperand,
    /// Indicates that a request was made to take the address of a non-addressable operand
    /// such as `Operand::Immediate`
    NonAddressOperand
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::ErrorAccessingMemory(_) => "error accessing memory",
            &Error::ReadOnlyOperand         => "attempted to write to a read-only operand",
            &Error::NonAddressOperand       => "attempted to take the address of an operand with no address"
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            &Error::ErrorAccessingMemory(ref err) => Some(err),
            _                                     => None
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if let &Error::ErrorAccessingMemory(ref err) = self {
            write!(fmt, "error accessing memory: {}", err)
        } else {
            error::Error::description(self).fmt(fmt)
        }
    }
}

impl From<mem::Error> for Error {
    fn from(err: mem::Error) -> Error {
        Error::ErrorAccessingMemory(err)
    }
}


#[cfg(test)]
mod test {
    mod operand {
        use mem;
        use mem::MemoryExt;
        use cpus::mos6502::{cpu,Mos6502,Operand};
        use byteorder::LittleEndian;

        #[test]
        pub fn to_string_test() {
            assert_eq!("#$AB", Operand::Immediate(0xAB).to_string());
            assert_eq!("A", Operand::Accumulator.to_string());
            assert_eq!("$ABCD", Operand::Absolute(0xABCD).to_string());
            assert_eq!("$0BCD", Operand::Absolute(0x0BCD).to_string());
            assert_eq!("$AB", Operand::Absolute(0x00AB).to_string());
            assert_eq!("$ABCD,X", Operand::Indexed(0xABCD, cpu::RegisterName::X).to_string());
            assert_eq!("$ABCD,Y", Operand::Indexed(0xABCD, cpu::RegisterName::Y).to_string());
            assert_eq!("$0BCD,X", Operand::Indexed(0xBCD, cpu::RegisterName::X).to_string());
            assert_eq!("$0BCD,Y", Operand::Indexed(0xBCD, cpu::RegisterName::Y).to_string());
            assert_eq!("$AB,X", Operand::Indexed(0x00AB, cpu::RegisterName::X).to_string());
            assert_eq!("$AB,Y", Operand::Indexed(0x00AB, cpu::RegisterName::Y).to_string());
            assert_eq!("$ABCD", Operand::Indirect(0xABCD).to_string());
            assert_eq!("($AB,X)", Operand::PreIndexedIndirect(0xAB).to_string());
            assert_eq!("($AB),Y", Operand::PostIndexedIndirect(0xAB).to_string());
        }

        #[test]
        pub fn set_absolute_puts_value_in_memory_location() {
            let mut mem = mem::Fixed::new(10);
            let mut cpu = Mos6502::new();
            assert!(Operand::Absolute(2).set_u8(&mut cpu, &mut mem, 24).is_ok());
            let val = mem.get_u8(2).unwrap();
            assert_eq!(val, 24);
        }

        #[test]
        pub fn set_indexed_x_puts_value_in_memory_location() {
            let mut mem = mem::Fixed::new(10);
            let mut cpu = Mos6502::new();
            cpu.registers.x = 1;
            assert!(Operand::Indexed(2, cpu::RegisterName::X).set_u8(&mut cpu, &mut mem, 24).is_ok());
            let val = mem.get_u8(3).unwrap();
            assert_eq!(val, 24);
        }

        #[test]
        pub fn set_indexed_y_puts_value_in_memory_location() {
            let mut mem = mem::Fixed::new(10);
            let mut cpu = Mos6502::new();
            cpu.registers.y = 1;
            assert!(Operand::Indexed(2, cpu::RegisterName::Y).set_u8(&mut cpu, &mut mem, 24).is_ok());
            let val = mem.get_u8(3).unwrap();
            assert_eq!(val, 24);
        }

        #[test]
        pub fn set_accumulator_puts_value_in_accumulator() {
            let mut mem = mem::Fixed::new(10);
            let mut cpu = Mos6502::new();
            cpu.registers.a = 24;
            Operand::Accumulator.set_u8(&mut cpu, &mut mem, 42).unwrap();
            assert_eq!(cpu.registers.a, 42);
        }

        #[test]
        pub fn get_accumulator_returns_value_from_accumulator() {
            let mut cpu = Mos6502::new();
            cpu.registers.a = 42;
            assert_eq!(Ok(42), Operand::Accumulator.get_u8(&mut cpu, &mem::Empty));
        }

        #[test]
        pub fn get_immediate_returns_immediate_value() {
            let cpu = Mos6502::new();
            let val = Operand::Immediate(42).get_u8(&cpu, &mem::Empty).unwrap();
            assert_eq!(val, 42);
        }

        #[test]
        pub fn get_absolute_returns_value_from_memory_address() {
            let mut mem = mem::Fixed::new(10);
            let cpu = Mos6502::new();
            assert!(mem.set_u8(4, 42).is_ok());
            let val = Operand::Absolute(4).get_u8(&cpu, &mem).unwrap();
            assert_eq!(val, 42);
        }

        #[test]
        pub fn get_indexed_x_adds_x_to_address() {
            let mut mem = mem::Fixed::new(10);
            let mut cpu = Mos6502::new();
            assert!(mem.set_u8(4, 42).is_ok());
            cpu.registers.x = 2;
            let val = Operand::Indexed(2, cpu::RegisterName::X).get_u8(&cpu, &mem).unwrap();
            assert_eq!(val, 42);
        }

        #[test]
        pub fn get_indexed_y_adds_y_to_address() {
            let mut mem = mem::Fixed::new(10);
            let mut cpu = Mos6502::new();
            assert!(mem.set_u8(4, 42).is_ok());
            cpu.registers.y = 2;
            let val = Operand::Indexed(2, cpu::RegisterName::Y).get_u8(&cpu, &mem).unwrap();
            assert_eq!(val, 42);
        }

        #[test]
        pub fn get_preindexed_indirect_works() {
            let mut mem = mem::Fixed::new(10);
            let mut cpu = Mos6502::new();
            assert!(mem.set_u8(8, 42).is_ok()); // Value
            assert!(mem.set_u16::<LittleEndian>(6, 8).is_ok()); // Indirect Memory Address
            cpu.registers.x = 2;
            let val = Operand::PreIndexedIndirect(4).get_u8(&cpu, &mem).unwrap();
            assert_eq!(val, 42);
        }

        #[test]
        pub fn get_postindexed_indirect_works() {
            let mut mem = mem::Fixed::new(10);
            let mut cpu = Mos6502::new();
            assert!(mem.set_u8(8, 42).is_ok()); // Value
            assert!(mem.set_u16::<LittleEndian>(2, 6).is_ok()); // Indirect Memory Address
            cpu.registers.y = 2;
            let val = Operand::PostIndexedIndirect(2).get_u8(&cpu, &mem).unwrap();
            assert_eq!(val, 42);
        }
    }
}
