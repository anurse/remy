use std::error;

use mem;

use cpu::mos6502;
use cpu::mos6502::{Mos6502,Operand,OperandError};

mod adc;
mod and;
mod asl;
mod bcc;

#[derive(Copy,Debug,Eq,PartialEq)]
pub enum Instruction {
	ADC(Operand),
	AND(Operand),
	ASL(Operand),
	BCC(i8),
	BCS(i8),
	BEQ(i8),
	BIT(Operand),
	BMI(i8),
	BNE(i8),
	BPL(i8),
	BRK,
	BVC(i8),
	BVS(i8),
	CLC,
	CLD,
	CLI,
	CLV,
	CMP(Operand),
	CPX(Operand),
	CPY(Operand),
	DEC(Operand),
	DEX,
	DEY,
	EOR(Operand),
	INC(Operand),
	INX,
	INY,
	JMP(Operand),
	JSR(Operand),
	LDA(Operand),
	LDX(Operand),
	LDY(Operand),
	LSR(Operand),
	NOP,
	ORA(Operand),
	PHA,
	PHP,
	PLA,
	PLP,
	ROL(Operand),
	ROR(Operand),
	RTI,
	RTS,
	SBC(Operand),
	SEC,
	SED,
	SEI,
	STA(Operand),
	STX(Operand),
	STY(Operand),
	TAX,
	TAY,
	TSX,
	TXA,
	TXS,
	TYA,
}

#[derive(Clone,Debug,Eq,PartialEq)]
pub enum ExecError {
	ErrorRetrievingOperand(OperandError),
	ErrorReadingMemory(mem::MemoryError),
	UnknownInstruction
}

impl error::FromError<OperandError> for ExecError {
	fn from_error(err: OperandError) -> ExecError {
		ExecError::ErrorRetrievingOperand(err)
	}
}

impl error::FromError<mem::MemoryError> for ExecError {
	fn from_error(err: mem::MemoryError) -> ExecError {
		ExecError::ErrorReadingMemory(err)
	}
}

impl Instruction {
	pub fn exec<M>(self, cpu: &mut Mos6502<M>) -> Result<(), ExecError> where M: mem::Memory {
		match self {
			Instruction::ADC(op) => adc::exec(cpu, op),
			Instruction::AND(op) => and::exec(cpu, op),
			Instruction::ASL(op) => asl::exec(cpu, op), 
			Instruction::BCC(offset) => bcc::exec(cpu, offset),
			Instruction::BCS(offset) => {
				if cpu.registers.has_flags(mos6502::Flags::CARRY()) {
					cpu.pc.advance(offset as isize)
				}
				Ok(())
			}
			Instruction::BEQ(offset) => {
				if cpu.registers.has_flags(mos6502::Flags::ZERO()) {
					cpu.pc.advance(offset as isize)
				}
				Ok(())
			}
			Instruction::BIT(op) => {
				let m = try!(op.get_u8(cpu));
				let t = cpu.registers.a & m;

				if m & 0x80 != 0 {
					cpu.registers.set_flags(mos6502::Flags::SIGN());
				} else {
					cpu.registers.clear_flags(mos6502::Flags::SIGN());
				}

				if m & 0x40 != 0 {
					cpu.registers.set_flags(mos6502::Flags::OVERFLOW());
				} else {
					cpu.registers.clear_flags(mos6502::Flags::OVERFLOW());
				}

				if t == 0 {
					cpu.registers.set_flags(mos6502::Flags::ZERO());
				} else {
					cpu.registers.clear_flags(mos6502::Flags::ZERO());
				}

				Ok(())
			}
			Instruction::BMI(offset) => {
				if cpu.registers.has_flags(mos6502::Flags::SIGN()) {
					cpu.pc.advance(offset as isize)
				}
				Ok(())
			}
			Instruction::BNE(offset) => {
				if !cpu.registers.has_flags(mos6502::Flags::ZERO()) {
					cpu.pc.advance(offset as isize)
				}
				Ok(())
			}
			Instruction::BPL(offset) => {
				if !cpu.registers.has_flags(mos6502::Flags::SIGN()) {
					cpu.pc.advance(offset as isize)
				}
				Ok(())
			}
			Instruction::BRK => {
				cpu.pc.advance(1);
				let pc = cpu.pc.get();
				try!(cpu.push(((pc & 0xFF00) >> 8) as u8));
				try!(cpu.push((pc & 0x00FF) as u8));

				let new_flags = cpu.registers.get_flags() | mos6502::Flags::BREAK();
				try!(cpu.push(new_flags.bits()));

				cpu.pc.set(try!(cpu.mem.get_le_u16(0xFFFE)) as usize);
				Ok(())
			}
			Instruction::BVC(offset) => {
				if !cpu.registers.has_flags(mos6502::Flags::OVERFLOW()) {
					cpu.pc.advance(offset as isize)
				}
				Ok(())
			}
			Instruction::BVS(offset) => {
				if cpu.registers.has_flags(mos6502::Flags::OVERFLOW()) {
					cpu.pc.advance(offset as isize)
				}
				Ok(())
			}
			Instruction::CLC => {
				cpu.registers.clear_flags(mos6502::Flags::CARRY());
				Ok(())
			}
			Instruction::CLD => {
				cpu.registers.clear_flags(mos6502::Flags::BCD());
				Ok(())
			}
			Instruction::CLI => {
				cpu.registers.clear_flags(mos6502::Flags::INTERRUPT());
				Ok(())
			}
			Instruction::CLV => {
				cpu.registers.clear_flags(mos6502::Flags::OVERFLOW());
				Ok(())
			}
			Instruction::CMP(op) => {
				let val = try!(op.get_u8(cpu));
				let t = (cpu.registers.a as isize) - (val as isize);

				cpu.registers.clear_flags(
					mos6502::Flags::SIGN() |
					mos6502::Flags::CARRY() |
					mos6502::Flags::ZERO());

				if t < 0 {
					cpu.registers.set_flags(mos6502::Flags::SIGN());
				} else if t >= 0 {
					cpu.registers.set_flags(mos6502::Flags::CARRY());
					if t == 0 {
						cpu.registers.set_flags(mos6502::Flags::ZERO());
					}
				}
				Ok(())
			}
			_ => Err(ExecError::UnknownInstruction)
		}
	}
}

#[cfg(test)]
mod test {
	mod mos6502_instruction {
		use mem;
		use mem::Memory;
		use cpu::mos6502;
		use cpu::mos6502::{Instruction,Operand,Mos6502};
		use cpu::mos6502::cpu::STACK_START;

		#[test]
		pub fn bcs_does_not_modify_pc_if_carry_flag_clear() {
			let mut cpu = init_cpu();
			Instruction::BCS(1).exec(&mut cpu).unwrap();
			assert_eq!(cpu.pc.get(), 0xABCD);
		}

		#[test]
		pub fn bcs_advances_pc_by_specified_amount_if_carry_flag_set() {
			let mut cpu = init_cpu();
			cpu.registers.set_flags(mos6502::Flags::CARRY());
			Instruction::BCS(1).exec(&mut cpu).unwrap();
			assert_eq!(cpu.pc.get(), 0xABCE);
		}

		#[test]
		pub fn beq_advances_pc_by_specified_amount_if_zero_flag_set() {
			let mut cpu = init_cpu();
			cpu.registers.set_flags(mos6502::Flags::ZERO());
			Instruction::BEQ(1).exec(&mut cpu).unwrap();
			assert_eq!(cpu.pc.get(), 0xABCE);
		}

		#[test]
		pub fn beq_does_not_modify_pc_if_zero_flag_clear() {
			let mut cpu = init_cpu();
			Instruction::BEQ(1).exec(&mut cpu).unwrap();
			assert_eq!(cpu.pc.get(), 0xABCD);
		}

		#[test]
		pub fn bit_sets_sign_bit_if_bit_7_of_operand_is_set() {
			let mut cpu = init_cpu();
			cpu.registers.a = 0xFF;
			Instruction::BIT(Operand::Immediate(0x80)).exec(&mut cpu).unwrap();
			assert_eq!(cpu.registers.get_flags(), mos6502::Flags::SIGN() | mos6502::Flags::RESERVED());
		}

		#[test]
		pub fn bit_clears_sign_bit_if_bit_7_of_operand_is_not_set() {
			let mut cpu = init_cpu();
			cpu.registers.a = 0xFF;
			cpu.registers.set_flags(mos6502::Flags::SIGN() | mos6502::Flags::RESERVED());
			Instruction::BIT(Operand::Immediate(0x01)).exec(&mut cpu).unwrap();
			assert_eq!(cpu.registers.get_flags(), mos6502::Flags::RESERVED());
		}

		#[test]
		pub fn bit_sets_overflow_bit_if_bit_6_of_operand_is_set() {
			let mut cpu = init_cpu();
			cpu.registers.a = 0xFF;
			Instruction::BIT(Operand::Immediate(0x40)).exec(&mut cpu).unwrap();
			assert_eq!(cpu.registers.get_flags(), mos6502::Flags::OVERFLOW() | mos6502::Flags::RESERVED());
		}

		#[test]
		pub fn bit_clears_overflow_bit_if_bit_6_of_operand_is_not_set() {
			let mut cpu = init_cpu();
			cpu.registers.a = 0xFF;
			cpu.registers.set_flags(mos6502::Flags::OVERFLOW() | mos6502::Flags::RESERVED());
			Instruction::BIT(Operand::Immediate(0x01)).exec(&mut cpu).unwrap();
			assert_eq!(cpu.registers.get_flags(), mos6502::Flags::RESERVED());
		}

		#[test]
		pub fn bit_sets_zero_flag_if_result_of_masking_operand_with_a_is_zero() {
			let mut cpu = init_cpu();
			cpu.registers.a = 0x02;
			Instruction::BIT(Operand::Immediate(0x01)).exec(&mut cpu).unwrap();
			assert_eq!(cpu.registers.get_flags(), mos6502::Flags::ZERO() | mos6502::Flags::RESERVED());
		}

		#[test]
		pub fn bit_clears_zero_flag_if_result_of_masking_operand_with_a_is_nonzero() {
			let mut cpu = init_cpu();
			cpu.registers.a = 0x02;
			cpu.registers.set_flags(mos6502::Flags::ZERO() | mos6502::Flags::RESERVED());
			Instruction::BIT(Operand::Immediate(0x03)).exec(&mut cpu).unwrap();
			assert_eq!(cpu.registers.get_flags(), mos6502::Flags::RESERVED());
		}

		#[test]
		pub fn bmi_advances_pc_by_specified_amount_if_sign_flag_set() {
			let mut cpu = init_cpu();
			cpu.registers.set_flags(mos6502::Flags::SIGN());
			Instruction::BMI(1).exec(&mut cpu).unwrap();
			assert_eq!(cpu.pc.get(), 0xABCE);
		}

		#[test]
		pub fn bmi_does_not_modify_pc_if_sign_flag_clear() {
			let mut cpu = init_cpu();
			Instruction::BMI(1).exec(&mut cpu).unwrap();
			assert_eq!(cpu.pc.get(), 0xABCD);
		}

		#[test]
		pub fn bne_advances_pc_by_specified_amount_if_zero_flag_clear() {
			let mut cpu = init_cpu();
			Instruction::BNE(1).exec(&mut cpu).unwrap();
			assert_eq!(cpu.pc.get(), 0xABCE);
		}

		#[test]
		pub fn bne_does_not_modify_pc_if_zero_flag_set() {
			let mut cpu = init_cpu();
			cpu.registers.set_flags(mos6502::Flags::ZERO());
			Instruction::BNE(1).exec(&mut cpu).unwrap();
			assert_eq!(cpu.pc.get(), 0xABCD);
		}

		#[test]
		pub fn bpl_advances_pc_by_specified_amount_if_sign_flag_clear() {
			let mut cpu = init_cpu();
			Instruction::BPL(1).exec(&mut cpu).unwrap();
			assert_eq!(cpu.pc.get(), 0xABCE);
		}

		#[test]
		pub fn bpl_does_not_modify_pc_if_sign_flag_set() {
			let mut cpu = init_cpu();
			cpu.registers.set_flags(mos6502::Flags::SIGN());
			Instruction::BPL(1).exec(&mut cpu).unwrap();
			assert_eq!(cpu.pc.get(), 0xABCD);
		}

		#[test]
		pub fn brk_increments_and_pushes_pc_on_to_stack() {
			let mut cpu = init_cpu();
			Instruction::BRK.exec(&mut cpu).unwrap();

			assert_eq!(Ok(0xAB), cpu.mem.get_u8(STACK_START + 16));
			assert_eq!(Ok(0xCE), cpu.mem.get_u8(STACK_START + 15));
		}

		#[test]
		pub fn brk_sets_break_flag_and_pushes_flags_on_to_stack() {
			let mut cpu = init_cpu();
			let flags = mos6502::Flags::SIGN() | mos6502::Flags::OVERFLOW() | mos6502::Flags::RESERVED();
			cpu.registers.set_flags(flags);
			Instruction::BRK.exec(&mut cpu).unwrap();

			assert_eq!(Ok((flags | mos6502::Flags::BREAK()).bits()), cpu.mem.get_u8(STACK_START + 14));
		}

		#[test]
		pub fn brk_does_not_set_break_flag_on_current_flags() {
			let mut cpu = init_cpu();
			let flags = mos6502::Flags::SIGN() | mos6502::Flags::OVERFLOW() | mos6502::Flags::RESERVED();
			cpu.registers.set_flags(flags);
			Instruction::BRK.exec(&mut cpu).unwrap();

			assert_eq!(flags, cpu.registers.get_flags());
		}

		#[test]
		pub fn brk_sets_pc_to_address_at_vector() {
			let mut cpu = init_cpu();
			Instruction::BRK.exec(&mut cpu).unwrap();

			assert_eq!(0xBEEF, cpu.pc.get());
		}

		#[test]
		pub fn bvc_advances_pc_by_specified_amount_if_overflow_flag_clear() {
			let mut cpu = init_cpu();
			Instruction::BVC(1).exec(&mut cpu).unwrap();
			assert_eq!(cpu.pc.get(), 0xABCE);
		}

		#[test]
		pub fn bvc_does_not_modify_pc_if_overflow_flag_set() {
			let mut cpu = init_cpu();
			cpu.registers.set_flags(mos6502::Flags::OVERFLOW());
			Instruction::BVC(1).exec(&mut cpu).unwrap();
			assert_eq!(cpu.pc.get(), 0xABCD);
		}

		#[test]
		pub fn bvs_advances_pc_by_specified_amount_if_overflow_flag_set() {
			let mut cpu = init_cpu();
			cpu.registers.set_flags(mos6502::Flags::OVERFLOW());
			Instruction::BVS(1).exec(&mut cpu).unwrap();
			assert_eq!(cpu.pc.get(), 0xABCE);
		}

		#[test]
		pub fn bvc_does_not_modify_pc_if_overflow_flag_clear() {
			let mut cpu = init_cpu();
			Instruction::BVS(1).exec(&mut cpu).unwrap();
			assert_eq!(cpu.pc.get(), 0xABCD);
		}

		#[test]
		pub fn clc_clears_carry_flag() {
			let mut cpu = init_cpu();
			cpu.registers.set_flags(mos6502::Flags::CARRY());
			Instruction::CLC.exec(&mut cpu).unwrap();
			assert!(!cpu.registers.has_flags(mos6502::Flags::CARRY()));
		}

		#[test]
		pub fn cld_clears_bcd_flag() {
			let mut cpu = init_cpu();
			cpu.registers.set_flags(mos6502::Flags::BCD());
			Instruction::CLD.exec(&mut cpu).unwrap();
			assert!(!cpu.registers.has_flags(mos6502::Flags::BCD()));
		}

		#[test]
		pub fn cli_clears_interrupt_flag() {
			let mut cpu = init_cpu();
			cpu.registers.set_flags(mos6502::Flags::INTERRUPT());
			Instruction::CLI.exec(&mut cpu).unwrap();
			assert!(!cpu.registers.has_flags(mos6502::Flags::INTERRUPT()));
		}

		#[test]
		pub fn clv_clears_overflow_flag() {
			let mut cpu = init_cpu();
			cpu.registers.set_flags(mos6502::Flags::OVERFLOW());
			Instruction::CLV.exec(&mut cpu).unwrap();
			assert!(!cpu.registers.has_flags(mos6502::Flags::OVERFLOW()));
		}

		#[test]
		pub fn cmp_sets_sign_bit_if_operand_greater_than_a() {
			let mut cpu = init_cpu();
			Instruction::CMP(Operand::Immediate(43)).exec(&mut cpu).unwrap();
			assert!(cpu.registers.has_flags(mos6502::Flags::SIGN()));
		}

		#[test]
		pub fn cmp_clears_sign_bit_if_operand_less_than_a() {
			let mut cpu = init_cpu();
			cpu.registers.set_flags(mos6502::Flags::SIGN());
			Instruction::CMP(Operand::Immediate(41)).exec(&mut cpu).unwrap();
			assert!(!cpu.registers.has_flags(mos6502::Flags::SIGN()));
		}

		#[test]
		pub fn cmp_sets_carry_bit_if_a_greater_than_operand() {
			let mut cpu = init_cpu();
			Instruction::CMP(Operand::Immediate(41)).exec(&mut cpu).unwrap();
			assert!(cpu.registers.has_flags(mos6502::Flags::CARRY()));
		}

		#[test]
		pub fn cmp_sets_carry_bit_if_a_equal_to_operand() {
			let mut cpu = init_cpu();
			Instruction::CMP(Operand::Immediate(42)).exec(&mut cpu).unwrap();
			assert!(cpu.registers.has_flags(mos6502::Flags::CARRY()));
		}

		#[test]
		pub fn cmp_clears_carry_bit_if_a_less_than_operand() {
			let mut cpu = init_cpu();
			cpu.registers.set_flags(mos6502::Flags::CARRY());
			Instruction::CMP(Operand::Immediate(43)).exec(&mut cpu).unwrap();
			assert!(!cpu.registers.has_flags(mos6502::Flags::CARRY()));
		}

		#[test]
		pub fn cmp_sets_zero_bit_if_a_equal_to_operand() {
			let mut cpu = init_cpu();
			Instruction::CMP(Operand::Immediate(42)).exec(&mut cpu).unwrap();
			assert!(cpu.registers.has_flags(mos6502::Flags::ZERO()));
		}

		#[test]
		pub fn cmp_clears_zero_bit_if_a_less_than_operand() {
			let mut cpu = init_cpu();
			cpu.registers.set_flags(mos6502::Flags::ZERO());
			Instruction::CMP(Operand::Immediate(43)).exec(&mut cpu).unwrap();
			assert!(!cpu.registers.has_flags(mos6502::Flags::ZERO()));
		}

		#[test]
		pub fn cmp_clears_zero_bit_if_a_greater_than_operand() {
			let mut cpu = init_cpu();
			cpu.registers.set_flags(mos6502::Flags::ZERO());
			Instruction::CMP(Operand::Immediate(41)).exec(&mut cpu).unwrap();
			assert!(!cpu.registers.has_flags(mos6502::Flags::ZERO()));
		}

		fn init_cpu() -> Mos6502<mem::VirtualMemory<'static>> {
			let base_memory = mem::FixedMemory::new(32);
			let stack_memory = mem::FixedMemory::new(32);
			let vector_memory = mem::FixedMemory::new(6);
			let mut vm = mem::VirtualMemory::new();
			vm.attach(0, Box::new(base_memory)).unwrap();
			vm.attach(STACK_START, Box::new(stack_memory)).unwrap();
			vm.attach(0xFFFA, Box::new(vector_memory)).unwrap();

			let mut cpu = Mos6502::new(vm);
			cpu.registers.a = 42;
			cpu.registers.sp = 16;
			cpu.pc.set(0xABCD);
			cpu.mem.set_le_u16(0xFFFE, 0xBEEF).unwrap();

			cpu
		}
	}
}
