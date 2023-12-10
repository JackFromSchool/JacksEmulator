use crate::register::Registers;
use crate::mmu::MMU;
use crate::dissasembler::{ OpCode, RegisterData, Flags, Register };
use crate::util::{ BitOperations, le_combine };

pub struct Cpu {
    pub registers: Registers,
    pub mmu: MMU,
}

impl Cpu {

    pub fn from_rom(rom: Vec<u8>) -> Self {
        let mut cpu = Self {
            registers: Registers::default(),
            mmu: MMU::new(rom),
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

    pub fn execute(&mut self, code: OpCode) -> u32 {
        let reference = self.registers.clone();

        match code.instruction {
            _ => ()
        };

        code.cycles.into()
    }

    pub fn load(&mut self, r1: RegisterData, r2: RegisterData, _flags: Flags) {
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

    fn push(&mut self, r1: RegisterData, _flags: Flags) {
        let val = match r1.register {
            Register::AF => self.registers.get_af(),
            Register::BC => self.registers.get_bc(),
            Register::DE => self.registers.get_de(),
            Register::HL => self.registers.get_hl(),
            _ => unreachable!("No other registers pushed to stack")
        };

        let (ms, ls) = val.split();
        let sp = &mut self.registers.sp;
        *sp -= 1;
        self.mmu.write_8(*sp, ms);
        *sp -= 1;
        self.mmu.write_8(*sp, ls);
    }

    fn pop(&mut self, r1: RegisterData, _flags: Flags) {
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
    
}
