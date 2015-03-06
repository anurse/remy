use mem::Memory;
use cpu::mos6502::{ExecError,Operand,Mos6502,Flags};

pub fn exec<M>(cpu: &mut Mos6502<M>, op: Operand) -> Result<(), ExecError> where M: Memory {
    let n = try!(op.get_u8(cpu)) as isize;
    let a = cpu.registers.a as isize;
    let c = if cpu.flags.carry() { 1 } else { 0 };

    let t = 
        if cpu.bcd_enabled && cpu.flags.intersects(Flags::BCD()) {
            let v = bcd_to_uint(n) + bcd_to_uint(a) + c;
            cpu.flags.set_if(Flags::CARRY(), v > 99);
            uint_to_bcd(v)
        } else {
            let v = n + a + c;
            cpu.flags.set_if(Flags::CARRY(), v > 255);
            v
        };

    cpu.flags.set_if(Flags::OVERFLOW(), (a & 0x80) != (t & 0x80));
	cpu.registers.a = (t & 0xFF) as u8;
	cpu.flags.set_sign_and_zero(cpu.registers.a);
	Ok(())
}

fn bcd_to_uint(bcd: isize) -> isize {
    (((bcd & 0xF0) >> 4) * 10) + (bcd & 0x0F)
}

fn uint_to_bcd(int: isize) -> isize {
    let v = if int > 99 {
        int - 100
    } else {
        int
    };
    if v > 99 {
        panic!("bcd overflow!");
    }
    let h = (v / 10) as u8;
    let l = (v % 10) as u8;
    
    ((h << 4) | l) as isize
}

#[cfg(test)]
mod test {
    use mem::VirtualMemory;
	use cpu::mos6502::instr::adc;
	use cpu::mos6502::{Mos6502,Operand,Flags};

	#[test]
	pub fn adc_adds_regularly_when_carry_not_set() {
		let mut cpu = init_cpu();
		adc::exec(&mut cpu, Operand::Immediate(1)).unwrap();
		assert_eq!(cpu.registers.a, 43);
	}

	#[test]
	pub fn adc_adds_carry_value_when_carry_flag_is_set() {
		let mut cpu = init_cpu();
		cpu.flags.set(Flags::CARRY()); // Set CARRY()
		adc::exec(&mut cpu, Operand::Immediate(1)).unwrap();
		assert_eq!(cpu.registers.a, 44);
	}

	#[test]
	pub fn adc_sets_flags_when_overflow() {
		let mut cpu = init_cpu();
		adc::exec(&mut cpu, Operand::Immediate(255)).unwrap();
		assert_eq!(cpu.registers.a, 41);
		assert_eq!(cpu.flags, Flags::CARRY() | Flags::RESERVED());
	}

    #[test]
    pub fn adc_does_regular_addition_when_bcd_disabled_even_when_bcd_flag_set() {
		let vm = VirtualMemory::new();
		let mut cpu = Mos6502::without_bcd(vm);
        cpu.flags.set(Flags::BCD());
        cpu.registers.a = 0xAB;
        adc::exec(&mut cpu, Operand::Immediate(0xCD)).unwrap();
        assert_eq!(0xAB + 0xCD, cpu.registers.a);
    }

    #[test]
    pub fn adc_adds_bcd_when_bcd_flag_set() {
        let mut cpu = init_cpu();
        cpu.flags.set(Flags::BCD());
        cpu.registers.a = 0x25;
        adc::exec(&mut cpu, Operand::Immediate(0x24)).unwrap();
        assert_eq!(0x49, cpu.registers.a); // 49 in bcd
    }

    #[test]
    pub fn adc_sets_carry_when_bcd_addition_overflows() {
        let mut cpu = init_cpu();
        cpu.flags.set(Flags::BCD());
        cpu.registers.a = 0x90;
        adc::exec(&mut cpu, Operand::Immediate(0x12)).unwrap();
        assert_eq!(0x02, cpu.registers.a);
        assert!(cpu.flags.intersects(Flags::CARRY()));
    }

	fn init_cpu() -> Mos6502<VirtualMemory<'static>> {
		let vm = VirtualMemory::new();
		let mut cpu = Mos6502::new(vm);
		cpu.registers.a = 42;
		cpu
	}
}
