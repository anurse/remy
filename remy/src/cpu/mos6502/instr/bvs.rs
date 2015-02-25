use mem::Memory;
use cpu::mos6502::{ExecError,Mos6502,Flags};

pub fn exec<M>(cpu: &mut Mos6502<M>, offset: i8) -> Result<(), ExecError> where M: Memory {
    if cpu.flags.intersects(Flags::OVERFLOW()) {
        cpu.pc.advance(offset as isize)
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use mem::VirtualMemory;
	use cpu::mos6502::instr::bvs;
	use cpu::mos6502::{Mos6502,Flags};

    #[test]
    pub fn bvs_advances_pc_by_specified_amount_if_overflow_flag_set() {
        let mut cpu = init_cpu();
        cpu.flags.set(Flags::OVERFLOW());
        bvs::exec(&mut cpu, 1).unwrap();
        assert_eq!(cpu.pc.get(), 0xABCE);
    }

    #[test]
    pub fn bvs_does_not_modify_pc_if_overflow_flag_clear() {
        let mut cpu = init_cpu();
        bvs::exec(&mut cpu, 1).unwrap();
        assert_eq!(cpu.pc.get(), 0xABCD);
    }

    fn init_cpu() -> Mos6502<VirtualMemory<'static>> {
        let vm = VirtualMemory::new();
        let mut cpu = Mos6502::new(vm);

        cpu.pc.set(0xABCD);

        cpu
    }
}
