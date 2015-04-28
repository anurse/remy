use mem::Memory;
use cpus::mos6502::exec;
use cpus::mos6502::{Mos6502,Flags};

pub fn exec(cpu: &mut Mos6502, flag_selector: Flags) -> Result<(), exec::Error> {
    cpu.flags.set(flag_selector);
    Ok(())
}

#[cfg(test)]
mod test {
    use cpus::mos6502::exec::set_flag;
    use cpus::mos6502::{Mos6502,Flags};

    #[test]
    pub fn set_flag_sets_flag() {
        let mut cpu = Mos6502::new();
        cpu.pc.set(0xABCD);
        cpu.flags.set(Flags::SIGN());
        set_flag::exec(&mut cpu, Flags::CARRY()).unwrap();
        assert!(cpu.flags.intersects(Flags::CARRY()));
        assert!(cpu.flags.intersects(Flags::SIGN()));
    }
}
