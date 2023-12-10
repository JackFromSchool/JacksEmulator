use crate::register::Registers;
use crate::mmu::MMU;
use crate::dissasembler::{ OpCode, RegisterData, Flags, Register, Condition, Instruction, Dissasembler };
use crate::util::{ BitOperations, le_combine };

pub struct Cpu {
    pub registers: Registers,
    pub mmu: MMU,
    pub halted: bool,
}

impl Cpu {

    pub fn from_rom(rom: Vec<u8>) -> Self {
        let mut cpu = Self {
            registers: Registers::default(),
            mmu: MMU::new(rom),
            halted: false,
        };

        // Cpu defaults
        cpu.registers.pc = 0x100;
        cpu.registers.set_af(0x01B0);
        cpu.registers.set_bc(0x0013);
        cpu.registers.set_de(0x00D8);
        cpu.registers.set_hl(0x014D);
        cpu.registers.sp = 0xFFFE;

        cpu
    }


    pub fn tick(&mut self, d: &Dissasembler) -> u32 {
        let code = &d.unprefixed[&self.mmu.read_8(self.registers.pc)];
        
        match code.instruction {
            Instruction::NOP => (),
            Instruction::STOP => std::process::exit(0),
            Instruction::HALT => self.halt_cpu(),
            Instruction::EI => self.enable_interupts(),
            Instruction::DI => self.disable_interupts(),
            Instruction::JR(c, by) => self.jump_relative(c, by),
            Instruction::LD(r1, r2) => self.load(r1, r2),
            Instruction::LDH(_, _) => unimplemented!(),
            Instruction::LDASP => unimplemented!(),
            Instruction::INC(r) => self.increment(r),
            Instruction::DEC(r) => self.decrement(r),
            Instruction::RLCA => self.rotate_left_copy_a(),
            Instruction::RLA => self.rotate_left_a(),
            Instruction::RRCA => self.rotate_right_copy_a(),
            Instruction::RRA => self.rotate_right_a(),
            Instruction::DAA => self.decimal_adjust_accumulator(),
            Instruction::SCF => self.set_carry_flag(),
            Instruction::CPL => self.complement_accumulator(),
            Instruction::CCF => self.complement_carry_flag(),
            Instruction::ADD(_, _) => unimplemented!(),
            Instruction::SUB(r) => self.subtract(r),
            Instruction::ADC(r) => self.add_carry(r),
            Instruction::SBC(r) => self.subtract_carry(r),
            Instruction::AND(r) => self.and(r),
            Instruction::XOR(r) => self.xor(r),
            Instruction::OR(r) => self.or(r),
            Instruction::CP(r) => self.compare(r),
            Instruction::RET(c) => self.ret(c),
            Instruction::CALL(c, r) => self.call(c, r),
            Instruction::POP(r) => self.pop(r),
            Instruction::PUSH(r) => self.push(r),
            Instruction::JP(c, r) => self.jump(c, r),
            Instruction::RETI => self.return_interrupt(),
            Instruction::RST(v) => self.restart(v),
            Instruction::RLC(r) => self.rotate_left_copy(r),
            Instruction::RRC(r) => self.rotate_right_copy(r),
            Instruction::RL(r) => self.rotate_left(r),
            Instruction::RR(r) => self.rotate_right(r),
            Instruction::SLA(r) => self.shift_left_arithmetic(r),
            Instruction::SRA(r) => self.shift_right_arithmetic(r),
            Instruction::SWAP(r) => self.swap(r),
            Instruction::SRL(r) => self.shift_right_logical(r),
            Instruction::BIT(b, r) => self.test_bit(b, r),
            Instruction::RES(b, r) => self.reset_bit(b, r),
            Instruction::SET(b, r) => self.set_bit(b, r),
            Instruction::PREFIX => (),
        };
        
        code.cycles.into()
    }

    pub fn load(&mut self, r1: RegisterData, r2: RegisterData) {
        if r2.register.is_16() && !r2.pointer {
            let mut val = match r2.register {
                Register::AF => self.registers.get_af(),
                Register::BC => self.registers.get_bc(),
                Register::DE => self.registers.get_de(),
                Register::HL => self.registers.get_hl(),
                Register::SP => self.registers.sp,
                Register::PC => self.registers.pc,
                Register::Const16(x) => x,
                _ => unreachable!("u16 values are handled here not a {}", r2.register)
            };

            if let Some(func) = r2.operation {
                val = func(val);
            }

            if r2.pointer {
                val = self.mmu.read_16(val);
            }

            if r1.pointer {
                let mut index = match r1.register {
                    Register::AF => self.registers.get_af(),
                    Register::BC => self.registers.get_bc(),
                    Register::DE => self.registers.get_de(),
                    Register::HL => self.registers.get_hl(),
                    Register::SP => self.registers.sp,
                    Register::PC => self.registers.pc,
                    Register::Const16(x) => x,
                    _ => unreachable!("All u16 values are handled here not a {}", r1.register)
                };

                if let Some(func) = r1.operation {
                    index = func(index);
                }

                self.mmu.write_16(index, val);
            } else {
                match r1.register {
                    Register::AF => self.registers.set_af(val),
                    Register::BC => self.registers.set_bc(val),
                    Register::DE => self.registers.set_de(val),
                    Register::HL => self.registers.set_hl(val),
                    Register::SP => self.registers.sp = val,
                    Register::PC => self.registers.pc = val,
                    _ => unreachable!("All u16 values are handled here not a {}", r1.register)
                };
            }

        } else {
            let val = match r2.register {
                Register::A => self.registers.a,
                Register::F => self.registers.f,
                Register::B => self.registers.b,
                Register::C => self.registers.c,
                Register::D => self.registers.d,
                Register::E => self.registers.e,
                Register::H => self.registers.h,
                Register::L => self.registers.l,
                Register::Const8(x) => x,
                _ => {
                    let mut index = match r2.register {
                        Register::AF => self.registers.get_af(),
                        Register::BC => self.registers.get_bc(),
                        Register::DE => self.registers.get_de(),
                        Register::HL => self.registers.get_hl(),
                        Register::SP => self.registers.sp,
                        Register::PC => self.registers.pc,
                        Register::Const16(x) => x,
                        _ => unreachable!("Handled in outer match")
                    };

                    if let Some(func) = r2.operation  {
                        index = func(index);
                    }

                    self.mmu.read_8(index)
                }
            };

            match r1.register {
                Register::A => self.registers.a = val,
                Register::F => self.registers.f = val,
                Register::B => self.registers.b = val,
                Register::C => self.registers.c = val,
                Register::D => self.registers.d = val,
                Register::E => self.registers.e = val,
                Register::H => self.registers.h = val,
                Register::L => self.registers.l = val,
                _ => {
                    let mut index = match r2.register {
                        Register::AF => self.registers.get_af(),
                        Register::BC => self.registers.get_bc(),
                        Register::DE => self.registers.get_de(),
                        Register::HL => self.registers.get_hl(),
                        Register::SP => self.registers.sp,
                        Register::PC => self.registers.pc,
                        Register::Const16(x) => x,
                        _ => unreachable!("Handled in outer match")
                    };

                    if let Some(func) = r1.operation  {
                        index = func(index);
                    }

                    self.mmu.write_8(index, val);
                }
            }
        }
    }

    fn push(&mut self, r1: RegisterData) {
        let val = match r1.register {
            Register::AF => self.registers.get_af(),
            Register::BC => self.registers.get_bc(),
            Register::DE => self.registers.get_de(),
            Register::HL => self.registers.get_hl(),
            Register::PC => self.registers.pc,
            _ => unreachable!("No other registers pushed to stack")
        };

        let (ms, ls) = val.split();
        let sp = &mut self.registers.sp;
        *sp -= 1;
        self.mmu.write_8(*sp, ms);
        *sp -= 1;
        self.mmu.write_8(*sp, ls);
    }

    fn pop(&mut self, r1: RegisterData) {
        let sp = &mut self.registers.sp;

        let ls = self.mmu.read_8(*sp);
        *sp -= 1;
        let ms = self.mmu.read_8(*sp);
        *sp -= 1;

        let val = le_combine(ls, ms);
        match r1.register {
            Register::AF => self.registers.set_af(val),
            Register::BC => self.registers.set_bc(val),
            Register::DE => self.registers.set_de(val),
            Register::HL => self.registers.set_hl(val),
            Register::PC => self.registers.pc = val,
            _ => unreachable!("No other registers pushed to stack")
        };
    }

    fn add(&mut self, r1: RegisterData, r2: RegisterData, flags: Flags) {
        if r1.register == Register::SP {
            let og = self.registers.sp;
            
            let add = match r2.register {
                Register::Const8(x) => x as i16,
                _ => unreachable!()
            };

            let (sum, overflow) = og.overflowing_add_signed(add);
            let reg = &mut self.registers;
            
            // TODO: Figure out other flags
            reg.unset_z();
            reg.unset_n();

            reg.sp = sum;
            
        } else if r1.register.is_16() {
            let og = self.registers.get_hl();
            
            let add = match r2.register {
                Register::BC => self.registers.get_bc(),
                Register::DE => self.registers.get_de(),
                Register::HL => self.registers.get_hl(),
                Register::SP => self.registers.sp,
                _ => unreachable!()
            };

            let (sum, overflowed) = og.overflowing_add(add);
            
            let reg = &mut self.registers;
            if overflowed {
                reg.set_c()
            } else {
                reg.unset_c()
            }

            if (og & 0xFFF) + (add & 0xFFF) > 0xFFF {
                reg.set_h()
            } else {
                reg.unset_h()
            }

            reg.unset_n();
            reg.set_hl(sum);
            
        } else {
            let og = self.registers.a;

            let add = match r2.register {
                Register::A => self.registers.a,
                Register::B => self.registers.b,
                Register::C => self.registers.c,
                Register::D => self.registers.d,
                Register::E => self.registers.e,
                Register::H => self.registers.h,
                Register::L => self.registers.l,
                Register::HL => self.mmu.read_8(self.registers.get_hl()),
                Register::Const8(x) => x,
                _ => unreachable!()
            };

            let (sum, overflowed) = og.overflowing_add(add);
            let reg = &mut self.registers;
            
            if overflowed {
                reg.set_c()
            } else {
                reg.unset_c()
            }

            if (og & 0xF) + (add & 0xF) > 0xF {
                reg.set_h()
            } else {
                reg.unset_h()
            }

            if sum == 0 {
                reg.set_z()
            } else {
                reg.unset_z()
            }

            reg.unset_n();

            reg.a = sum;
        }
    }

    pub fn add_carry(&mut self, r: RegisterData) {
        let add = match r.register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
            Register::HL => self.mmu.read_8(self.registers.get_hl()),
            Register::Const8(x) => x,
            _ => unreachable!()
        };
        
        let a = self.registers.a;
        let reg = &mut self.registers;

        let (sum, overflowed) = a.overflowing_add(add + if reg.get_c() { 1 } else { 0 });

        if overflowed {
            reg.set_c();
        } else {
            reg.unset_c();
        }

        if (a & 0xF) + (add & 0xF) > 0xF {
            reg.set_h();
        } else {
            reg.unset_h();
        }

        if sum == 0 {
            reg.set_z();
        } else {
            reg.unset_z();
        }

        reg.a = sum;
        
    }

    pub fn subtract(&mut self, r: RegisterData) {
        let sub = match r.register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
            Register::HL => self.mmu.read_8(self.registers.get_hl()),
            Register::Const8(x) => x,
            _ => unreachable!()
        };

        let a = self.registers.a;
        let reg = &mut self.registers;

        let result = a.wrapping_sub(sub);

        if a < sub {
            reg.set_c();
        } else {
            reg.unset_c();
        }

        if (a & 0xF) < (sub &0xF) {
            reg.set_h();
        } else {
            reg.unset_c();
        }

        if result == 0 {
            reg.set_z();
        } else {
            reg.unset_z();
        }

        reg.set_n();

        reg.a = result;
    } 

    pub fn subtract_carry(&mut self, r: RegisterData) {
        let sub = match r.register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
            Register::HL => self.mmu.read_8(self.registers.get_hl()),
            Register::Const8(x) => x,
            _ => unreachable!()
        };
        
        let c = if self.registers.get_c() { 1 } else { 0 };
        let a = self.registers.a;
        let reg = &mut self.registers;

        let result = a.wrapping_sub(sub).wrapping_sub(c);

        if (a as u16) < ((sub as u16) + (c as u16)) {
            reg.set_c();
        } else {
            reg.unset_c();
        }

        if (a & 0xF) < ((sub &0xF) + c) {
            reg.set_h();
        } else {
            reg.unset_c();
        }

        if result == 0 {
            reg.set_z();
        } else {
            reg.unset_z();
        }

        reg.set_n();

        reg.a = result;
    }

    pub fn compare(&mut self, r: RegisterData) {
        let sub = match r.register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
            Register::HL => self.mmu.read_8(self.registers.get_hl()),
            Register::Const8(x) => x,
            _ => unreachable!()
        };

        let a = self.registers.a;
        let reg = &mut self.registers;

        let result = a.wrapping_sub(sub);

        if a < sub {
            reg.set_c();
        } else {
            reg.unset_c();
        }

        if (a & 0xF) < (sub &0xF) {
            reg.set_h();
        } else {
            reg.unset_c();
        }

        if result == 0 {
            reg.set_z();
        } else {
            reg.unset_z();
        }

        reg.set_n();
    }

    pub fn increment(&mut self, r: RegisterData) {
        if r.register.is_16() {
            let reg = &mut self.registers;
            match r.register {
                Register::BC => reg.set_bc(reg.get_bc().wrapping_add(1)),
                Register::DE => reg.set_de(reg.get_de().wrapping_add(1)),
                Register::HL => reg.set_hl(reg.get_hl().wrapping_add(1)),
                Register::SP => reg.sp = reg.sp.wrapping_add(1),
                _ => unreachable!()
            };
        } else {
            let og = match r.register {
                Register::A => self.registers.a,
                Register::B => self.registers.b,
                Register::C => self.registers.c,
                Register::D => self.registers.d,
                Register::E => self.registers.e,
                Register::H => self.registers.h,
                Register::L => self.registers.l,
                Register::HL => self.mmu.read_8(self.registers.get_hl()),
                _ => unreachable!()
            };

            let reg = &mut self.registers;

            if og.wrapping_add(1) == 0 {
                reg.set_z();
            } else {
                reg.unset_z();
            }

            if (og & 0xF) + 1 > 0xF {
                reg.set_h();
            } else {
                reg.unset_h();
            }

            reg.unset_n();

            let result = og.wrapping_add(1);

            match r.register {
                Register::A => self.registers.a = result,
                Register::B => self.registers.b = result,
                Register::C => self.registers.c = result,
                Register::D => self.registers.d = result,
                Register::E => self.registers.e = result,
                Register::H => self.registers.h = result,
                Register::L => self.registers.l = result,
                Register::HL => self.mmu.write_8(self.registers.get_hl(), result),
                _ => unreachable!()
            };
        }
    }
    
    pub fn decrement(&mut self, r: RegisterData) {
        if r.register.is_16() {
            let reg = &mut self.registers;
            match r.register {
                Register::BC => reg.set_bc(reg.get_bc().wrapping_sub(1)),
                Register::DE => reg.set_de(reg.get_de().wrapping_sub(1)),
                Register::HL => reg.set_hl(reg.get_hl().wrapping_sub(1)),
                Register::SP => reg.sp = reg.sp.wrapping_sub(1),
                _ => unreachable!()
            };
        } else {
            let og = match r.register {
                Register::A => self.registers.a,
                Register::B => self.registers.b,
                Register::C => self.registers.c,
                Register::D => self.registers.d,
                Register::E => self.registers.e,
                Register::H => self.registers.h,
                Register::L => self.registers.l,
                Register::HL => self.mmu.read_8(self.registers.get_hl()),
                _ => unreachable!()
            };

            let reg = &mut self.registers;

            if og.wrapping_sub(1) == 0 {
                reg.set_z();
            } else {
                reg.unset_z();
            }

            if (og & 0xF) == 0  {
                reg.set_h();
            } else {
                reg.unset_h();
            }

            reg.set_n();

            let result = og.wrapping_sub(1);

            match r.register {
                Register::A => self.registers.a = result,
                Register::B => self.registers.b = result,
                Register::C => self.registers.c = result,
                Register::D => self.registers.d = result,
                Register::E => self.registers.e = result,
                Register::H => self.registers.h = result,
                Register::L => self.registers.l = result,
                Register::HL => self.mmu.write_8(self.registers.get_hl(), result),
                _ => unreachable!()
            };
        }
    }

    pub fn and(&mut self, r: RegisterData) {
        let og = match r.register {
                Register::A => self.registers.a,
                Register::B => self.registers.b,
                Register::C => self.registers.c,
                Register::D => self.registers.d,
                Register::E => self.registers.e,
                Register::H => self.registers.h,
                Register::L => self.registers.l,
                Register::HL => self.mmu.read_8(self.registers.get_hl()),
                _ => unreachable!()
        };

        let a = self.registers.a;
        let reg = &mut self.registers;

        if (a & og) == 0 {
            reg.set_z();
        } else {
            reg.unset_z();
        }

        reg.set_h();

        reg.a = a & og;
    }

    pub fn or(&mut self, r: RegisterData) {
        let og = match r.register {
                Register::A => self.registers.a,
                Register::B => self.registers.b,
                Register::C => self.registers.c,
                Register::D => self.registers.d,
                Register::E => self.registers.e,
                Register::H => self.registers.h,
                Register::L => self.registers.l,
                Register::HL => self.mmu.read_8(self.registers.get_hl()),
                _ => unreachable!()
        };

        let a = self.registers.a;
        let reg = &mut self.registers;

        if (a | og) == 0 {
            reg.set_z();
        } else {
            reg.unset_z();
        }

        reg.a = a | og;
    }

    pub fn xor(&mut self, r: RegisterData) {
        let og = match r.register {
                Register::A => self.registers.a,
                Register::B => self.registers.b,
                Register::C => self.registers.c,
                Register::D => self.registers.d,
                Register::E => self.registers.e,
                Register::H => self.registers.h,
                Register::L => self.registers.l,
                Register::HL => self.mmu.read_8(self.registers.get_hl()),
                _ => unreachable!()
        };

        let a = self.registers.a;
        let reg = &mut self.registers;

        if (a ^ og) == 0 {
            reg.set_z();
        } else {
            reg.unset_z();
        }

        reg.a = a ^ og;
    }

    pub fn rotate_left_a(&mut self) {
        let reg = &mut self.registers;

        reg.unset_z();
        reg.unset_n();
        reg.unset_h();

        let c = if reg.get_c() { 1 } else { 0 };

        if (reg.a & 0b1000_0000) == 0b1000_0000 {
            reg.set_c();
        } else {
            reg.unset_c();
        }

        reg.a = (reg.a << 1) | c;
    }

    pub fn rotate_left_copy_a(&mut self) {
        let reg = &mut self.registers;

        reg.unset_z();
        reg.unset_n();
        reg.unset_h();

        if (reg.a & 0b1000_0000) == 0b1000_0000 {
            reg.set_c();
        } else {
            reg.unset_c();
        }

        reg.a = (reg.a >> 1) | if reg.get_c() { 1 } else { 0 };
    }

    pub fn rotate_right_a(&mut self) {
        let reg = &mut self.registers;

        reg.unset_z();
        reg.unset_n();
        reg.unset_h();

        let c = if reg.get_c() { 0x80 } else { 0 };

        if (reg.a & 0b0000_0001) == 0b0000_0001 {
            reg.set_c();
        } else {
            reg.unset_c();
        }

        reg.a = (reg.a >> 1) | c;
    }

    pub fn rotate_right_copy_a(&mut self) {
        let reg = &mut self.registers;

        reg.unset_z();
        reg.unset_n();
        reg.unset_h();

        if (reg.a & 0b0000_0001) == 0b0000_0001 {
            reg.set_c();
        } else {
            reg.unset_c();
        }

        reg.a = (reg.a >> 1) | if reg.get_c() { 0x80 } else { 0 };
    }

    pub fn rotate_left(&mut self, r: RegisterData) {
        let og = match r.register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
            Register::HL => self.mmu.read_8(self.registers.get_hl()),
            _ => unreachable!()
        };

        let reg = &mut self.registers;

        let c = if reg.get_c() { 1 } else { 0 };

        reg.unset_h();
        reg.unset_n();

        if (og & 0b1000_0000) == 0b1000_0000 {
            reg.set_c();
        } else {
            reg.unset_c();
        }

        let new = (og << 1) | c;

        if new == 0 {
            reg.set_z();
        } else {
            reg.unset_z();
        }

        match r.register {
            Register::A => self.registers.a = new,
            Register::B => self.registers.b = new,
            Register::C => self.registers.c = new,
            Register::D => self.registers.d = new,
            Register::E => self.registers.e = new,
            Register::H => self.registers.h = new,
            Register::L => self.registers.l = new,
            Register::HL => self.mmu.write_8(self.registers.get_hl(), new),
            _ => unreachable!()
        }
    }

    pub fn rotate_left_copy(&mut self, r: RegisterData) {
        let og = match r.register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
            Register::HL => self.mmu.read_8(self.registers.get_hl()),
            _ => unreachable!()
        };

        let reg = &mut self.registers;

        reg.unset_h();
        reg.unset_n();

        if (og & 0b1000_0000) == 0b1000_0000 {
            reg.set_c();
        } else {
            reg.unset_c();
        }

        let new = (og << 1) | if reg.get_c() { 1 } else { 0 };

        if new == 0 {
            reg.set_z();
        } else {
            reg.unset_z();
        }

        match r.register {
            Register::A => self.registers.a = new,
            Register::B => self.registers.b = new,
            Register::C => self.registers.c = new,
            Register::D => self.registers.d = new,
            Register::E => self.registers.e = new,
            Register::H => self.registers.h = new,
            Register::L => self.registers.l = new,
            Register::HL => self.mmu.write_8(self.registers.get_hl(), new),
            _ => unreachable!()
        }
    }

    pub fn rotate_right(&mut self, r: RegisterData) {
        let og = match r.register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
            Register::HL => self.mmu.read_8(self.registers.get_hl()),
            _ => unreachable!()
        };

        let reg = &mut self.registers;

        let c = if reg.get_c() { 0x80 } else { 0 };

        reg.unset_h();
        reg.unset_n();

        if (og & 0b0000_0001) == 0b0000_0001 {
            reg.set_c();
        } else {
            reg.unset_c();
        }

        let new = (og >> 1) | c;

        if new == 0 {
            reg.set_z();
        } else {
            reg.unset_z();
        }

        match r.register {
            Register::A => self.registers.a = new,
            Register::B => self.registers.b = new,
            Register::C => self.registers.c = new,
            Register::D => self.registers.d = new,
            Register::E => self.registers.e = new,
            Register::H => self.registers.h = new,
            Register::L => self.registers.l = new,
            Register::HL => self.mmu.write_8(self.registers.get_hl(), new),
            _ => unreachable!()
        }
    }

    pub fn rotate_right_copy(&mut self, r: RegisterData) {
        let og = match r.register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
            Register::HL => self.mmu.read_8(self.registers.get_hl()),
            _ => unreachable!()
        };

        let reg = &mut self.registers;

        reg.unset_h();
        reg.unset_n();

        if (og & 0b0000_0001) == 0b0000_0001 {
            reg.set_c();
        } else {
            reg.unset_c();
        }

        let new = (og >> 1) | if reg.get_c() { 0x80 } else { 0 };

        if new == 0 {
            reg.set_z();
        } else {
            reg.unset_z();
        }

        match r.register {
            Register::A => self.registers.a = new,
            Register::B => self.registers.b = new,
            Register::C => self.registers.c = new,
            Register::D => self.registers.d = new,
            Register::E => self.registers.e = new,
            Register::H => self.registers.h = new,
            Register::L => self.registers.l = new,
            Register::HL => self.mmu.write_8(self.registers.get_hl(), new),
            _ => unreachable!()
        }
    }

    pub fn set_carry_flag(&mut self) {
        let reg = &mut self.registers;
        reg.unset_n();
        reg.unset_h();
        reg.set_c();
    }

    pub fn complement_carry_flag(&mut self) {
        let reg = &mut self.registers;
        reg.unset_n();
        reg.unset_h();
        if reg.get_c() {
            reg.unset_c();
        } else {
            reg.set_c();
        }
    }

    pub fn complement_accumulator(&mut self) {
        let reg = &mut self.registers;
        reg.a = !reg.a;
        reg.set_n();
        reg.set_h();
    }

    pub fn decimal_adjust_accumulator(&mut self) {
        let mut a = self.registers.a;
        let mut adjust = 0;
        let reg = &mut self.registers;
        
        if reg.get_n() {
            adjust |= if reg.get_c() { 0x60 } else { 0x00 };
            adjust |= if reg.get_h() { 0x06} else { 0x00 };
            a = a.wrapping_sub(adjust);
        } else {
            adjust |= if a & 0x0F > 0x09 { 0x06 } else { 0x00 };
            adjust |= if a > 099 { 0x60 } else { 0x00 };
            a = a.wrapping_add(adjust);
        }
        
        if adjust >= 0x60 {
            reg.set_c();
        } else {
            reg.unset_c();
        }

        if a == 0 {
            reg.set_z();
        } else {
            reg.unset_z();
        }

        reg.unset_h();
        
        reg.a = a;
    }

    pub fn shift_left_arithmetic(&mut self, r: RegisterData) {
        let og  = match r.register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
            Register::HL => self.mmu.read_8(self.registers.get_hl()),
            _ => unreachable!()
        };

        let reg = &mut self.registers;

        if og & 0x80 == 0x80 {
            reg.set_c();
        } else {
            reg.unset_c();
        }

        let result = og << 1;
        
        if result == 0 {
            reg.set_z();
        } else {
            reg.unset_z();
        }
        
        reg.unset_h();
        reg.unset_n();

        match r.register {
            Register::A => self.registers.a = result,
            Register::B => self.registers.b = result,
            Register::C => self.registers.c = result,
            Register::D => self.registers.d = result,
            Register::E => self.registers.e = result,
            Register::H => self.registers.h = result,
            Register::L => self.registers.l = result,
            Register::HL => self.mmu.write_8(self.registers.get_hl(), result),
            _ => unreachable!()
        }
    }

    pub fn shift_right_arithmetic(&mut self, r: RegisterData) {
        let og  = match r.register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
            Register::HL => self.mmu.read_8(self.registers.get_hl()),
            _ => unreachable!()
        };

        let reg = &mut self.registers;

        if og & 0x01 == 0x01 {
            reg.set_c();
        } else {
            reg.unset_c();
        }

        let result = (og >> 1) | ( og & 0x80);
        
        if result == 0 {
            reg.set_z();
        } else {
            reg.unset_z();
        }

        reg.unset_h();
        reg.unset_n();

        match r.register {
            Register::A => self.registers.a = result,
            Register::B => self.registers.b = result,
            Register::C => self.registers.c = result,
            Register::D => self.registers.d = result,
            Register::E => self.registers.e = result,
            Register::H => self.registers.h = result,
            Register::L => self.registers.l = result,
            Register::HL => self.mmu.write_8(self.registers.get_hl(), result),
            _ => unreachable!()
        }
    }

    pub fn shift_right_logical(&mut self, r: RegisterData) {
        let og  = match r.register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
            Register::HL => self.mmu.read_8(self.registers.get_hl()),
            _ => unreachable!()
        };

        let reg = &mut self.registers;

        if og & 0x01 == 0x01 {
            reg.set_c();
        } else {
            reg.unset_c();
        }
        
        reg.unset_h();
        reg.unset_n();

        let result = og >> 1;
        
        if result == 0 {
            reg.set_z();
        } else {
            reg.unset_z();
        }

        match r.register {
            Register::A => self.registers.a = result,
            Register::B => self.registers.b = result,
            Register::C => self.registers.c = result,
            Register::D => self.registers.d = result,
            Register::E => self.registers.e = result,
            Register::H => self.registers.h = result,
            Register::L => self.registers.l = result,
            Register::HL => self.mmu.write_8(self.registers.get_hl(), result),
            _ => unreachable!()
        }
    }

    pub fn swap(&mut self, r: RegisterData) {
        let og  = match r.register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
            Register::HL => self.mmu.read_8(self.registers.get_hl()),
            _ => unreachable!()
        };

        let result = (og >> 4) | (og << 4);

        let reg = &mut self.registers;

        reg.unset_c();
        reg.unset_h();
        reg.unset_n();

        if result == 0 {
            reg.set_z();
        } else {
            reg.unset_z();
        }

        match r.register {
            Register::A => self.registers.a = result,
            Register::B => self.registers.b = result,
            Register::C => self.registers.c = result,
            Register::D => self.registers.d = result,
            Register::E => self.registers.e = result,
            Register::H => self.registers.h = result,
            Register::L => self.registers.l = result,
            Register::HL => self.mmu.write_8(self.registers.get_hl(), result),
            _ => unreachable!()
        }
    }

    pub fn test_bit(&mut self, bit: u8, r: RegisterData) {
        let val  = match r.register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
            Register::HL => self.mmu.read_8(self.registers.get_hl()),
            _ => unreachable!()
        };

        let reg = &mut self.registers;
        let binary: u8 = 2u8.pow(bit as u32);

        reg.unset_n();
        reg.set_h();

        if (val & binary) == binary {
            reg.set_z();
        } else {
            reg.unset_z();
        }
    }

    pub fn reset_bit(&mut self, bit: u8, r: RegisterData) {
        let mut val = match r.register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
            Register::HL => self.mmu.read_8(self.registers.get_hl()),
            _ => unreachable!()
        };

        val = val & !(2u8.pow(bit as u32));

        match r.register {
            Register::A => self.registers.a = val,
            Register::B => self.registers.b = val,
            Register::C => self.registers.c = val,
            Register::D => self.registers.d = val,
            Register::E => self.registers.e = val,
            Register::H => self.registers.h = val,
            Register::L => self.registers.l = val,
            Register::HL => self.mmu.write_8(self.registers.get_hl(), val),
            _ => unreachable!()
        }
    }

    pub fn set_bit(&mut self, bit: u8, r: RegisterData) {
        let mut val = match r.register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
            Register::HL => self.mmu.read_8(self.registers.get_hl()),
            _ => unreachable!()
        };

        val = val | 2u8.pow(bit as u32);

        match r.register {
            Register::A => self.registers.a = val,
            Register::B => self.registers.b = val,
            Register::C => self.registers.c = val,
            Register::D => self.registers.d = val,
            Register::E => self.registers.e = val,
            Register::H => self.registers.h = val,
            Register::L => self.registers.l = val,
            Register::HL => self.mmu.write_8(self.registers.get_hl(), val),
            _ => unreachable!()
        }
    }

    pub fn enable_interupts(&mut self) {
        self.mmu.enable_interupts();
    }

    pub fn disable_interupts(&mut self) {
        self.mmu.disble_interupts();
    }

    pub fn halt_cpu(&mut self) {
        self.halted = true;
    }

    pub fn jump(&mut self, c: Condition, r: RegisterData) {
        let to = match r.register {
            Register::HL => self.registers.get_hl(),
            Register::Const16(x) => x,
            _ => unreachable!()
        };

        match c {
            Condition::Always => {
                self.registers.pc = to;
            },
            Condition::Z if self.registers.get_n() => {
                self.registers.pc = to;
            },
            Condition::NZ if !self.registers.get_n() => {
                self.registers.pc = to;
            },
            Condition::C if self.registers.get_c() => {
                self.registers.pc = to;
            },
            Condition::NC if !self.registers.get_c() => {
                self.registers.pc = to;
            },
            _ => ()
        };
    }

    pub fn jump_relative(&mut self, c: Condition, by: i8) {
        let to = ((self.registers.pc as u32 as i32) + (by as i32)) as u16;
        
        match c {
            Condition::Always => {
                self.registers.pc = to; 
            }
            Condition::Z if self.registers.get_n() => {
                self.registers.pc = to; 
            },
            Condition::NZ if !self.registers.get_n() => {
                self.registers.pc = to; 
            },
            Condition::C if self.registers.get_c() => {
                self.registers.pc = to; 
            },
            Condition::NC if !self.registers.get_c() => {
                self.registers.pc = to; 
            },
            _ => ()
        }
    }

    pub fn call(&mut self, c: Condition, r: RegisterData) {
        if let Register::Const16(to) = r.register {
            match c {
                Condition::Always => {
                    self.push(RegisterData::from_reg(Register::PC));
                    self.registers.pc = to; 
                }
                Condition::Z if self.registers.get_n() => {
                    self.push(RegisterData::from_reg(Register::PC));
                    self.registers.pc = to; 
                },
                Condition::NZ if !self.registers.get_n() => {
                    self.push(RegisterData::from_reg(Register::PC));
                    self.registers.pc = to; 
                },
                Condition::C if self.registers.get_c() => {
                    self.push(RegisterData::from_reg(Register::PC));
                    self.registers.pc = to; 
                },
                Condition::NC if !self.registers.get_c() => {
                    self.push(RegisterData::from_reg(Register::PC));
                    self.registers.pc = to; 
                },
                _ => ()
            }
        } else {
            unreachable!()
        }
    }

    pub fn ret(&mut self, c: Condition) {
        match c {
            Condition::Always => {
                self.pop(RegisterData::from_reg(Register::PC));
            }
            Condition::Z if self.registers.get_n() => {
                self.pop(RegisterData::from_reg(Register::PC));
            },
            Condition::NZ if !self.registers.get_n() => {
                self.pop(RegisterData::from_reg(Register::PC));
            },
            Condition::C if self.registers.get_c() => {
                self.pop(RegisterData::from_reg(Register::PC));
            },
            Condition::NC if !self.registers.get_c() => {
                self.pop(RegisterData::from_reg(Register::PC));
            },
            _ => ()
        }
    }

    pub fn return_interrupt(&mut self) {
        self.enable_interupts();
        self.pop(RegisterData::from_reg(Register::PC));
    }

    pub fn restart(&mut self, to: u16) {
        self.push(RegisterData::from_reg(Register::PC));
        self.registers.pc = to;
    }

}


