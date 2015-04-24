use mem::Memory;
use cpus::mos6502::exec;
use cpus::mos6502::{Mos6502,Operand};

pub fn exec<M>(cpu: &mut Mos6502<M>, op: Operand) -> Result<(), exec::Error> where M: Memory {
    let addr = try!(op.get_addr(cpu));

    cpu.pc.set(addr as u64);
    Ok(())
}

#[cfg(test)]
mod test {
    use byteorder::LittleEndian;

    use mem::{self,Memory,MemoryExt};
    use cpus::mos6502::exec::jmp;
    use cpus::mos6502::{Mos6502,Operand};

    #[test]
    pub fn jmp_sets_pc_to_address_if_absolute_argument() {
        let mem = mem::Virtual::new();
        let mut cpu = Mos6502::new(mem);

        jmp::exec(&mut cpu, Operand::Absolute(0xBEEF)).unwrap();

        assert_eq!(0xBEEF, cpu.pc.get());
    }

    #[test]
    pub fn jmp_sets_pc_to_value_at_address_if_indirect_argument() {
        let mut vm = mem::Virtual::new();
        let mut mem = mem::Fixed::new(10);
        mem.set_u16::<LittleEndian>(5, 0xBEEF).unwrap();
        vm.attach(0, Box::new(mem)).unwrap();
        let mut cpu = Mos6502::new(vm);

        jmp::exec(&mut cpu, Operand::Indirect(5)).unwrap();

        assert_eq!(0xBEEF, cpu.pc.get());
    }
}
