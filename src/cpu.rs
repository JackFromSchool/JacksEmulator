use crate::dissasembler::{
    Condition, Dissasembler, Flags, Instruction, Register, RegisterData, Take,
};
use crate::interupts::Interupt;
use crate::mmu::MMU;
use crate::register::Registers;
use crate::util::{le_combine, BitOperations};

use std::fs::File;
use std::fs::OpenOptions;
use std::io::LineWriter;
use std::io::Write;

pub struct Cpu {
    pub registers: Registers,
    pub mmu: MMU,
    pub halted: bool,
    pub debug_file: LineWriter<File>,
}

impl Cpu {
    pub fn from_rom(rom: Vec<u8>) -> Self {
        File::create("log.txt").unwrap();
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .open("log.txt")
            .unwrap();

        let debug_file = LineWriter::new(file);

        let mut cpu = Self {
            registers: Registers::default(),
            mmu: MMU::new(rom),
            halted: false,
            debug_file,
        };

        // Cpu defaults
        cpu.registers.pc = 0x0;
        cpu.registers.set_af(0x01B0);
        cpu.registers.set_bc(0x0013);
        cpu.registers.set_de(0x00D8);
        cpu.registers.set_hl(0x014D);
        cpu.registers.sp = 0xFFFE;

        cpu
    }

    pub fn tick(&mut self, d: &Dissasembler) -> (u8, bool) {
        if self.halted {
            if self.mmu.interupt.has_interupts() {
                self.halted = false;
                return (4, true);
            }
            return (4, false);
        }
        
        let pc = self.registers.pc;
        println!("{}", pc);

        /*
        let str = format!("A:{:02x} F:{:02x} B:{:02x} C:{:02x} D:{:02x} E:{:02x} H:{:02x} L:{:02x} SP:{:04x} PC:{:04x} PCMEM:{:02x},{:02x},{:02x},{:02x}\n",
                          self.registers.a,
                          self.registers.f,
                          self.registers.b,
                          self.registers.c,
                          self.registers.d,
                          self.registers.e,
                          self.registers.h,
                          self.registers.l,
                          self.registers.sp,
                          self.registers.pc,
                          self.mmu.read_8(pc),
                          self.mmu.read_8(pc.wrapping_add(1)),
                          self.mmu.read_8(pc.wrapping_add(2)),
                          self.mmu.read_8(pc.wrapping_add(3))
                          ).to_uppercase();

        self.debug_file.write_all(str.as_bytes()).unwrap();
        */
        //println!("{}", self.registers.pc);
        let mut code = &d.unprefixed[&self.mmu.read_8(self.registers.pc)];
        self.increment(RegisterData::from_reg(Register::PC));

        if matches!(code.instruction, Instruction::PREFIX) {
            code = &d.prefixed[&self.mmu.read_8(self.registers.pc)];
            self.increment(RegisterData::from_reg(Register::PC));
        }

        let mut instruction = code.instruction;

        //println!("Running: {}", instruction);
        
        match code.extra_data {
            Take::None => (),
            Take::Eight => {
                if code.code == 0xE0 || code.code == 0xF8 {
                    instruction = instruction.insert_r1(self.get_8());
                } else {
                    instruction = instruction.insert_r2(self.get_8());
                }
            }
            Take::Sixteen => {
                if code.code == 0x08 || code.code == 0xEA {
                    instruction = instruction.insert_r1(self.get_16())
                } else {
                    instruction = instruction.insert_r2(self.get_16())
                }
            }
        }

        match instruction {
            Instruction::NOP => (),
            Instruction::STOP => std::process::exit(0),
            Instruction::HALT => self.halt_cpu(),
            Instruction::EI => self.enable_interupts(),
            Instruction::DI => self.disable_interupts(),
            Instruction::JR(c, by) => self.jump_relative(c, by),
            Instruction::LD(r1, r2) => self.load(r1, r2),
            Instruction::LDH(r1, r2) => self.load_high(r1, r2),
            Instruction::LDASP(r) => self.load_hl_sp(r),
            Instruction::LDINC(r1, r2) => self.load_incrememnt(r1, r2),
            Instruction::LDDEC(r1, r2) => self.load_decrement(r1, r2),
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
            Instruction::ADD(r1, r2) => self.add(r1, r2, code.flags),
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

        (code.cycles.into(), false)
    }

    pub fn get_8(&mut self) -> RegisterData {
        let data = self.mmu.read_8(self.registers.pc);
        self.increment(RegisterData::from_reg(Register::PC));
        RegisterData::from_reg(Register::Const8(data))
    }

    pub fn get_16(&mut self) -> RegisterData {
        let data = self.mmu.read_16(self.registers.pc);
        self.increment(RegisterData::from_reg(Register::PC));
        self.increment(RegisterData::from_reg(Register::PC));
        RegisterData::from_reg(Register::Const16(data))
    }

    pub fn load_incrememnt(&mut self, _r1: RegisterData, r2: RegisterData) {
        if r2.register.is_16() {
            let val = self.mmu.read_8(self.registers.get_hl());
            self.registers
                .set_hl(self.registers.get_hl().wrapping_add(1));

            self.registers.a = val;
        } else {
            let val = self.registers.a;

            self.mmu.write_8(self.registers.get_hl(), val);
            self.registers
                .set_hl(self.registers.get_hl().wrapping_add(1));
        }
    }

    pub fn load_decrement(&mut self, _r1: RegisterData, r2: RegisterData) {
        if r2.register.is_16() {
            let val = self.mmu.read_8(self.registers.get_hl());
            self.registers
                .set_hl(self.registers.get_hl().wrapping_sub(1));

            self.registers.a = val;
        } else {
            let val = self.registers.a;

            self.mmu.write_8(self.registers.get_hl(), val);
            self.registers
                .set_hl(self.registers.get_hl().wrapping_sub(1));
        }
    }

    pub fn load_high(&mut self, r1: RegisterData, r2: RegisterData) {
        if r2.register == Register::A {
            let val = self.registers.a;

            match r1.register {
                Register::C => self.mmu.write_8(self.registers.c as u16 + 0xFF00, val),
                Register::Const8(i) => self.mmu.write_8(i as u16 + 0xFF00, val),
                _ => unreachable!(),
            }
        } else {
            let val = match r2.register {
                Register::C => self.mmu.read_8((self.registers.c as u16) + 0xFF00),
                Register::Const8(i) => self.mmu.read_8(0xFF00 + (i as u16)),
                _ => unreachable!(),
            };

            self.registers.a = val;
        }
    }

    pub fn load_hl_sp(&mut self, r: RegisterData) {
        let val = match r.register {
            Register::Const8(x) => x as i8 as i16,
            _ => unreachable!(),
        };

        self.registers
            .set_hl(self.mmu.read_16(self.registers.sp.wrapping_add_signed(val)));

        let reg = &mut self.registers;
        reg.unset_z();
        reg.unset_n();
        reg.set_h();
        reg.set_c();
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
                _ => unreachable!("u16 values are handled here not a {}", r2.register),
            };

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
                    _ => unreachable!("All u16 values are handled here not a {}", r1.register),
                };

                self.mmu.write_16(index, val);
            } else {
                if matches!(r1.register, Register::A) {
                    self.registers.a = self.mmu.read_8(val);
                    return;
                }

                match r1.register {
                    Register::AF => self.registers.set_af(val),
                    Register::BC => self.registers.set_bc(val),
                    Register::DE => self.registers.set_de(val),
                    Register::HL => self.registers.set_hl(val),
                    Register::SP => self.registers.sp = val,
                    Register::PC => self.registers.pc = val,
                    Register::Const16(x) => self.mmu.write_16(x, val),
                    _ => unreachable!("All u16 values are handled here not a {}", r1.register),
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
                    let index = match r2.register {
                        Register::AF => self.registers.get_af(),
                        Register::BC => self.registers.get_bc(),
                        Register::DE => self.registers.get_de(),
                        Register::HL => self.registers.get_hl(),
                        Register::SP => self.registers.sp,
                        Register::PC => self.registers.pc,
                        Register::Const16(x) => x,
                        _ => unreachable!("Handled in outer match: {}", r2.register),
                    };

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
                    let mut index = match r1.register {
                        Register::AF => self.registers.get_af(),
                        Register::BC => self.registers.get_bc(),
                        Register::DE => self.registers.get_de(),
                        Register::HL => self.registers.get_hl(),
                        Register::SP => self.registers.sp,
                        Register::PC => self.registers.pc,
                        Register::Const16(x) => x,
                        _ => unreachable!("Handled in outer match: {}", r1.register),
                    };

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
            _ => unreachable!("No other registers pushed to stack"),
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
        *sp += 1;
        let ms = self.mmu.read_8(*sp);
        *sp += 1;

        let val = le_combine(ls, ms);
        match r1.register {
            Register::AF => self
                .registers
                .set_af((val & 0xFFF0) | self.registers.get_af() & 0x000F),
            Register::BC => self.registers.set_bc(val),
            Register::DE => self.registers.set_de(val),
            Register::HL => self.registers.set_hl(val),
            Register::PC => self.registers.pc = val,
            _ => unreachable!("No other registers pushed to stack"),
        };
    }

    fn add(&mut self, r1: RegisterData, r2: RegisterData, flags: Flags) {
        if r1.register == Register::SP {
            let og = self.registers.sp;

            let add = match r2.register {
                Register::Const8(x) => x as i8 as i16,
                _ => unreachable!(),
            };

            let sum = og.wrapping_add_signed(add);
            let reg = &mut self.registers;

            reg.unset_z();
            reg.unset_n();

            // Flags may be incorrect but they are usesless so who cares
            let cast = og as i16;
            if add < 0 {
                if (cast & 0xFF) < (-add & 0xFF) {
                    reg.set_c();
                } else {
                    reg.unset_c();
                }

                if (cast & 0xF) < (-add & 0xF) {
                    reg.set_h();
                } else {
                    reg.unset_h();
                }
            } else {
                if (cast & 0xFF) + (add & 0xFF) > 0xFF {
                    reg.set_c();
                } else {
                    reg.unset_c();
                }

                if (cast & 0xF) + (add & 0xF) > 0xF {
                    reg.set_h();
                } else {
                    reg.unset_h();
                }
            }

            reg.sp = sum;
        } else if r1.register.is_16() {
            let og = self.registers.get_hl();

            let add = match r2.register {
                Register::BC => self.registers.get_bc(),
                Register::DE => self.registers.get_de(),
                Register::HL => self.registers.get_hl(),
                Register::SP => self.registers.sp,
                _ => unreachable!(),
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
                _ => unreachable!(),
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
            _ => unreachable!(),
        };

        let a = self.registers.a;
        let reg = &mut self.registers;

        let (sum, overflowed) = a.overflowing_add(add);
        let (sum2, overflowed2) = sum.overflowing_add(if reg.get_c() { 1 } else { 0 });
        
        if (a & 0xF) + (add & 0xF) + if reg.get_c() { 1 } else { 0 } > 0xF {
            reg.set_h();
        } else {
            reg.unset_h();
        }

        if overflowed || overflowed2 {
            reg.set_c();
        } else {
            reg.unset_c();
        }

        if sum2 == 0 {
            reg.set_z();
        } else {
            reg.unset_z();
        }

        reg.unset_n();

        reg.a = sum2;
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
            _ => unreachable!(),
        };

        let a = self.registers.a;
        let reg = &mut self.registers;

        let result = a.wrapping_sub(sub);

        if a < sub {
            reg.set_c();
        } else {
            reg.unset_c();
        }

        if (a & 0xF) < (sub & 0xF) {
            reg.set_h();
        } else {
            reg.unset_h();
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
            _ => unreachable!(),
        };

        let c = if self.registers.get_c() { 1 } else { 0 };
        let a = self.registers.a;
        let reg = &mut self.registers;

        let result = a.wrapping_sub(sub).wrapping_sub(c);
        
        if (a & 0xF) < (sub & 0xF) + c {
            reg.set_h();
        } else {
            reg.unset_h();
        }

        if (a as u16) < ((sub as u16) + (c as u16)) {
            reg.set_c();
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
            _ => unreachable!(),
        };

        let a = self.registers.a;
        let reg = &mut self.registers;

        let result = a.wrapping_sub(sub);

        if a < sub {
            reg.set_c();
        } else {
            reg.unset_c();
        }

        if (a & 0xF) < (sub & 0xF) {
            reg.set_h();
        } else {
            reg.unset_h();
        }

        if result == 0 {
            reg.set_z();
        } else {
            reg.unset_z();
        }

        reg.set_n();
    }

    pub fn increment(&mut self, r: RegisterData) {
        if r.register.is_16() && !r.pointer {
            let reg = &mut self.registers;
            match r.register {
                Register::BC => reg.set_bc(reg.get_bc().wrapping_add(1)),
                Register::DE => reg.set_de(reg.get_de().wrapping_add(1)),
                Register::HL => reg.set_hl(reg.get_hl().wrapping_add(1)),
                Register::SP => reg.sp = reg.sp.wrapping_add(1),
                Register::PC => reg.pc = reg.pc.wrapping_add(1),
                _ => unreachable!(),
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
                _ => unreachable!(),
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
                _ => unreachable!(),
            };
        }
    }

    pub fn decrement(&mut self, r: RegisterData) {
        if r.register.is_16() && !r.pointer {
            let reg = &mut self.registers;
            match r.register {
                Register::BC => reg.set_bc(reg.get_bc().wrapping_sub(1)),
                Register::DE => reg.set_de(reg.get_de().wrapping_sub(1)),
                Register::HL => reg.set_hl(reg.get_hl().wrapping_sub(1)),
                Register::SP => reg.sp = reg.sp.wrapping_sub(1),
                _ => unreachable!(),
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
                _ => unreachable!(),
            };

            let reg = &mut self.registers;

            if og.wrapping_sub(1) == 0 {
                reg.set_z();
            } else {
                reg.unset_z();
            }

            if (og & 0xF) == 0 {
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
                _ => unreachable!(),
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
            Register::Const8(x) => x,
            _ => unreachable!(),
        };

        let a = self.registers.a;
        let reg = &mut self.registers;

        if (a & og) == 0 {
            reg.set_z();
        } else {
            reg.unset_z();
        }

        reg.set_h();
        reg.unset_n();
        reg.unset_c();

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
            Register::Const8(x) => x,
            _ => unreachable!(),
        };

        let a = self.registers.a;
        let reg = &mut self.registers;

        reg.unset_c();
        reg.unset_n();
        reg.unset_h();

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
            Register::Const8(x) => x,
            _ => unreachable!(),
        };

        let a = self.registers.a;
        let reg = &mut self.registers;

        reg.unset_c();
        reg.unset_n();
        reg.unset_h();

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

        reg.a = (reg.a << 1) | if reg.get_c() { 1 } else { 0 };
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
            _ => unreachable!(),
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
            _ => unreachable!(),
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
            _ => unreachable!(),
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
            _ => unreachable!(),
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
            _ => unreachable!(),
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
            _ => unreachable!(),
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
            _ => unreachable!(),
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
            _ => unreachable!(),
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
        let mut value = self.registers.a;
        let mut adjust = 0;

        if self.registers.get_h() || (!self.registers.get_n() && (value & 0xF) > 9) {
            adjust |= 0x6;
        }

        if self.registers.get_c() || (!self.registers.get_n() && (value > 0x99)) {
            adjust |= 0x60;
            self.registers.set_c();
        }

        value = value.wrapping_add_signed(if self.registers.get_n() {
            -adjust
        } else {
            adjust
        });

        if value == 0 {
            self.registers.set_z();
        } else {
            self.registers.unset_z();
        }

        self.registers.unset_h();

        self.registers.a = value;
    }

    pub fn shift_left_arithmetic(&mut self, r: RegisterData) {
        let og = match r.register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
            Register::HL => self.mmu.read_8(self.registers.get_hl()),
            _ => unreachable!(),
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
            _ => unreachable!(),
        }
    }

    pub fn shift_right_arithmetic(&mut self, r: RegisterData) {
        let og = match r.register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
            Register::HL => self.mmu.read_8(self.registers.get_hl()),
            _ => unreachable!(),
        };

        let reg = &mut self.registers;

        if og & 0x01 == 0x01 {
            reg.set_c();
        } else {
            reg.unset_c();
        }

        let result = (og >> 1) | (og & 0x80);

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
            _ => unreachable!(),
        }
    }

    pub fn shift_right_logical(&mut self, r: RegisterData) {
        let og = match r.register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
            Register::HL => self.mmu.read_8(self.registers.get_hl()),
            _ => unreachable!(),
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
            _ => unreachable!(),
        }
    }

    pub fn swap(&mut self, r: RegisterData) {
        let og = match r.register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
            Register::HL => self.mmu.read_8(self.registers.get_hl()),
            _ => unreachable!(),
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
            _ => unreachable!(),
        }
    }

    pub fn test_bit(&mut self, bit: u8, r: RegisterData) {
        let val = match r.register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
            Register::HL => self.mmu.read_8(self.registers.get_hl()),
            _ => unreachable!(),
        };

        let reg = &mut self.registers;
        let binary: u8 = 2u8.pow(bit as u32);

        //println!("binary: {:b}, val: {:b}", binary, val);

        reg.unset_n();
        reg.set_h();

        if (val & binary) == binary {
            reg.unset_z();
        } else {
            reg.set_z();
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
            _ => unreachable!(),
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
            _ => unreachable!(),
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
            _ => unreachable!(),
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
            _ => unreachable!(),
        }
    }

    pub fn enable_interupts(&mut self) {
        self.mmu.enable_interupts();
    }

    pub fn disable_interupts(&mut self) {
        self.mmu.disble_interupts();
    }

    pub fn service_interupts(&mut self, interupt: Interupt) -> u8 {
        if interupt == Interupt::None || interupt == Interupt::Serial {
            return 0;
        }

        let goto = match interupt {
            Interupt::VBlank => 0x40,
            Interupt::LCD => 0x48,
            Interupt::Timer => 0x50,
            Interupt::Joypad => 0x60,
            _ => unreachable!(),
        };

        self.push(RegisterData::from_reg(Register::PC));
        self.registers.pc = goto;
        
        self.disable_interupts();
        20
    }

    pub fn halt_cpu(&mut self) {
        self.halted = true;
    }

    pub fn jump(&mut self, c: Condition, r: RegisterData) {
        let to = match r.register {
            Register::HL => self.registers.get_hl(),
            Register::Const16(x) => x,
            _ => unreachable!("{}", r.register),
        };

        match c {
            Condition::Always => {
                self.registers.pc = to;
            }
            Condition::Z if self.registers.get_z() => {
                self.registers.pc = to;
            }
            Condition::NZ if !self.registers.get_z() => {
                self.registers.pc = to;
            }
            Condition::C if self.registers.get_c() => {
                self.registers.pc = to;
            }
            Condition::NC if !self.registers.get_c() => {
                self.registers.pc = to;
            }
            _ => (),
        };
    }

    pub fn jump_relative(&mut self, c: Condition, by: RegisterData) {
        let val = match by.register {
            Register::Const8(x) => x,
            _ => unreachable!(),
        };
        let to = ((self.registers.pc as u32 as i32) + (val as i8 as i32)) as u16;

        match c {
            Condition::Always => {
                self.registers.pc = to;
            }
            Condition::Z if self.registers.get_z() => {
                self.registers.pc = to;
            }
            Condition::NZ if !self.registers.get_z() => {
                self.registers.pc = to;
            }
            Condition::C if self.registers.get_c() => {
                self.registers.pc = to;
            }
            Condition::NC if !self.registers.get_c() => {
                self.registers.pc = to;
            }
            _ => (),
        }
    }
    

    pub fn call(&mut self, c: Condition, r: RegisterData) {
        if let Register::Const16(to) = r.register {
            match c {
                Condition::Always => {
                    self.push(RegisterData::from_reg(Register::PC));
                    self.registers.pc = to;
                }
                Condition::Z if self.registers.get_z() => {
                    self.push(RegisterData::from_reg(Register::PC));
                    self.registers.pc = to;
                }
                Condition::NZ if !self.registers.get_z() => {
                    self.push(RegisterData::from_reg(Register::PC));
                    self.registers.pc = to;
                }
                Condition::C if self.registers.get_c() => {
                    self.push(RegisterData::from_reg(Register::PC));
                    self.registers.pc = to;
                }
                Condition::NC if !self.registers.get_c() => {
                    self.push(RegisterData::from_reg(Register::PC));
                    self.registers.pc = to;
                }
                _ => (),
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
            Condition::Z if self.registers.get_z() => {
                self.pop(RegisterData::from_reg(Register::PC));
            }
            Condition::NZ if !self.registers.get_z() => {
                self.pop(RegisterData::from_reg(Register::PC));
            }
            Condition::C if self.registers.get_c() => {
                self.pop(RegisterData::from_reg(Register::PC));
            }
            Condition::NC if !self.registers.get_c() => {
                self.pop(RegisterData::from_reg(Register::PC));
            }
            _ => (),
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
