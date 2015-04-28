use mem::Memory;
use cpus::mos6502::exec;
use cpus::mos6502::Mos6502;

pub fn exec<M>(cpu: &mut Mos6502, mem: &mut M, addr: u16) -> Result<(), exec::Error> where M: Memory {
    let pc = cpu.pc.get() - 1;
    try!(cpu.push(mem, ((pc & 0xFF00) >> 8) as u8));
    try!(cpu.push(mem, (pc & 0x00FF) as u8));
    cpu.pc.set(addr as u64);
    Ok(())
}

#[cfg(test)]
mod test {
    use mem;
    use mem::Memory;
    use cpus::mos6502::Mos6502;
    use cpus::mos6502::STACK_START;
    use cpus::mos6502::exec::jsr;

    #[test]
    pub fn jsr_sets_pc_to_address() {
        let (mut cpu, mut mem) = init_cpu();

        jsr::exec(&mut cpu, &mut mem, 0xBEEF).unwrap();

        assert_eq!(0xBEEF, cpu.pc.get());
    }

    #[test]
    pub fn jsr_pushes_old_pc_minus_one_to_stack() {
        let (mut cpu, mut mem) = init_cpu();

        jsr::exec(&mut cpu, &mut mem, 0xBEEF).unwrap();

        assert_eq!(Ok(0xCC), cpu.pull());
        assert_eq!(Ok(0xAB), cpu.pull());
    }

    fn init_cpu() -> (Mos6502,mem::Virtual<'static>) {
        let stack_memory = mem::Fixed::new(32);
        let mut vm = mem::Virtual::new();

        vm.attach(STACK_START, Box::new(stack_memory)).unwrap();

        let mut cpu = Mos6502::new();

        cpu.registers.a = 42;
        cpu.registers.sp = 16;
        cpu.pc.set(0xABCD);
        (cpu,vm)
    }
}
