use std::collections::hash_map::HashMap;
use serde_json::{ Result, Value };
use crate::cpu::Cpu;


#[derive(Clone, Copy)]
/// Represents a register and whether or not it is a pointer to data instead of an actual register.
/// Includes an optional operation to be done to the register value before its use
pub struct RegisterData {
    pub register: Register,
    pub pointer: bool,
}

#[derive(enum_display::EnumDisplay, PartialEq, Eq, Clone, Copy)]
/// Each register that might be used for an instruction and constant values should they be used
/// instead
pub enum Register {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
    Const8(u8),
    Const16(u16),
    None,
}

impl Register {

    pub fn is_16(&self) -> bool {
        match self {
            Self::AF | Self::BC | Self::DE | Self::HL | Self::PC | Self::SP | Self::Const16(_) => true,
            _ => false,
        }
    }

    
}

impl RegisterData {
    pub fn empty() -> Self {
        Self {
            register: Register::None,
            pointer: false,
        }
    }
    
    pub fn from_reg(register: Register) -> Self {
        Self {
            register,
            pointer: false,
        }
    }
}

impl std::convert::From<&str> for RegisterData {
    fn from(value: &str) -> Self {

        let mut pointer = false;
        if value.contains("(") {
            pointer = true;
        }

        let register = match value
            .replace("(", "")
            .replace(")", "")
            .replace("-", "")
            .replace("+", "") 
            .as_str()
        {
            "A" => Register::A,
            "F" => Register::F,
            "B" => Register::B,
            "C" => Register::C,
            "D" => Register::D,
            "E" => Register::E,
            "H" => Register::H,
            "L" => Register::L,
            "AF" => Register::AF,
            "BC" => Register::BC,
            "DE" => Register::DE,
            "HL" => Register::HL,
            "SP" => Register::SP,
            "PC" => Register::PC,
            "d16" | "a16"=> Register::None,
            "d8" | "r8" | "a8" => Register::None,
            _ => panic!("Register couldn't be deciphered: {}", value),
        };

        Self {
            register,
            pointer,
        }
    }
}

#[derive(Clone, Copy)]
/// Conditions that must be checked for certain instructions
pub enum Condition {
    NZ,
    NC,
    Z,
    C,
    Always,
}

impl std::convert::From<&str> for Condition {
    fn from(value: &str) -> Self {
        match value {
            "NC" => Self::NC,
            "C" => Self::C,
            "NZ" => Self::NZ,
            "Z" => Self::Z,
            _ => panic!("Condition couldn't be deciphered: {}", value)
        }
    }
}

#[derive(Clone, Copy)]
/// # Flags stores each flag as an Option<(bool, bool)>
/// If the flag contains a none then it is not affected by a subsuquent call to an instruction
///
/// If the flag containts the tuple then it is as follows:
///     * tuple.0 is true if the flag is dependent on the next instruction call
///     * tuple.1 is the value the flag should be set too if tuple.0 is false and it is not dependent
pub struct Flags {
    pub z: Option<(bool, bool)>,
    pub n: Option<(bool, bool)>,
    pub hc: Option<(bool, bool)>,
    pub c: Option<(bool, bool)>,
}

impl Flags {

    /// Creates a Flags with all flags containing a None value
    pub fn all_none() -> Self {
        Self {
            z: None,
            n: None,
            hc: None,
            c: None,
        }
    }

}

#[derive(enum_display::EnumDisplay, Clone, Copy)]
/// Represents all possible instructions the gameboy can do and the data required for them to run
pub enum Instruction {
    NOP, // Done
    STOP, // Done
    HALT, // Done
    EI, // Done
    DI, // Done
    JR(Condition, RegisterData), // Done
    LD(RegisterData, RegisterData), // Done
    LDH(RegisterData, RegisterData),
    /// Load instruction for LD HL, SP+i8
    LDASP(RegisterData),
    LDINC(RegisterData, RegisterData),
    LDDEC(RegisterData, RegisterData),
    INC(RegisterData), // Done
    DEC(RegisterData), // Done
    RLCA, // Done
    RLA, // Done
    RRCA, // Done
    RRA, // Done
    DAA, // Done
    SCF, // Done
    CPL, // Done
    CCF, // Done
    ADD(RegisterData, RegisterData), // Done
    SUB(RegisterData), // Done
    ADC(RegisterData), // Done
    SBC(RegisterData), // Done
    AND(RegisterData), // Done
    XOR(RegisterData), // Done
    OR(RegisterData), // Done
    CP(RegisterData), // Done
    RET(Condition), // Done
    CALL(Condition, RegisterData),
    POP(RegisterData), // Done
    PUSH(RegisterData), // Done
    JP(Condition, RegisterData), // Done
    RETI, // Done
    RST(u16),
    RLC(RegisterData), // Done
    RRC(RegisterData), // Done
    RL(RegisterData), // Done
    RR(RegisterData), // Done
    SLA(RegisterData), // Done
    SRA(RegisterData), // Done
    SWAP(RegisterData), // Done
    SRL(RegisterData), // Done
    BIT(u8, RegisterData), // Done
    RES(u8, RegisterData), // Done
    SET(u8, RegisterData), // Done
    PREFIX,
}

impl Instruction {

    pub fn insert_r2(&mut self, new: RegisterData) -> Instruction {
        match self {
            Self::LD(r1, _r2) => Self::LD(*r1, new),
            Self::LDH(r1, _r2) => Self::LDH(*r1, new),
            Self::ADD(r1, _r2) => Self::ADD(*r1, new),
            Self::SUB(_r1) => Self::SUB(new),
            Self::ADC(_r1) => Self::ADC(new),
            Self::SBC(_r1) => Self::SBC(new),
            Self::XOR(_r1) => Self::XOR(new),
            Self::OR(_r1) => Self::OR(new),
            Self::AND(_r1) => Self::AND(new),
            Self::CP(_r1) => Self::CP(new),
            Self::JR(c, _r2) => Self::JR(*c, new),
            Self::JP(c, _r2) => Self::JP(*c, new),
            Self::CALL(c, _r2) => Self::CALL(*c, new),
            _ => unreachable!("{}", self)
        }
    }
    
    pub fn insert_r1(&mut self, new: RegisterData) -> Instruction {
        match self {
            Self::LD(_r1, r2) => Self::LD(new, *r2),
            Self::LDASP(_r1) => Self::LDASP(new),
            Self::LDH(_r1, r2) => Self::LDH(new, *r2),
            _ => unreachable!()
        }
    }
    
}

#[derive(enum_display::EnumDisplay)]
pub enum Take {
    None,
    Eight,
    Sixteen,
}

/// Represents and opcode and what must be done for it to run correctly and in time
pub struct OpCode {
    pub code: u8,
    pub instruction: Instruction,
    pub flags: Flags,
    pub cycles: u8,
    pub extra_data: Take,
}

/// Holds two HashMaps that take in opcode numeric values and return the OpCode
pub struct Dissasembler {
    pub unprefixed: HashMap<u8, OpCode>,
    pub prefixed: HashMap<u8, OpCode>,
}

impl Dissasembler {

    /// Creates a new Dissasembler object that contains HashMaps that take an opcode value and
    /// retrn its associated OpCode
    pub fn new() -> Result<Self> {
        let v: Value = serde_json::from_str(include_str!("opcodes.json"))?;

        let mut unprefixed = HashMap::new();
        let mut prefixed = HashMap::new();
        
        let mut i = 0;
        loop {
            let object = v.get(i);

            if object.is_none() {
                break;
            }

            let object = object.unwrap();

            // Creation of all flags for opcode
            let flags_array = object["flagsZNHC"].as_array().unwrap();
            let mut flags = Flags::all_none();

            for (i, flag) in flags_array.iter().enumerate() {
                match i {
                    0 => flags.z = Self::create_flag(flag.as_str().unwrap()),
                    1 => flags.n = Self::create_flag(flag.as_str().unwrap()),
                    2 => flags.hc = Self::create_flag(flag.as_str().unwrap()),
                    3 => flags.c = Self::create_flag(flag.as_str().unwrap()),
                    _ => panic!("Flag Creation Panic")
                }
            }

            // Cycles of opcode set here 
            let cycles = object["cycles"].as_u64().unwrap() as u8;
            
            // Define if extra data is needed
            let bytes = object["bytes"].as_u64().unwrap();
            let extra_data = match bytes {
                1 => Take::None,
                2 if object.get("prefix").is_none() => Take::Eight,
                2 => Take::None,
                3 => Take::Sixteen,
                _ => unreachable!()
            };

            // Instruction created here
            let instruction = match object["mnemonic"].as_str().unwrap() {
                "NOP" => Instruction::NOP,
                "STOP" => Instruction::STOP,
                "HALT" => Instruction::HALT,
                "EI" => Instruction::EI,
                "DI" => Instruction::DI,
                "RLCA" => Instruction::RLCA,
                "RLA" => Instruction::RLA,
                "RRCA" => Instruction::RRCA,
                "RRA" => Instruction::RRA,
                "DAA" => Instruction::DAA,
                "SCF" => Instruction::SCF,
                "CPL" => Instruction::CPL,
                "CCF" => Instruction::CCF,
                "INC" => Instruction::INC(object["operands"][0].as_str().unwrap().into()),
                "DEC" => Instruction::DEC(object["operands"][0].as_str().unwrap().into()),
                "LD" => {
                    if object["operands"][1].as_str().unwrap() == "SP+r8" {
                        Instruction::LDASP(RegisterData::empty())
                    } else {
                        Instruction::LD(
                            object["operands"][0].as_str().unwrap().into(),
                            object["operands"][1].as_str().unwrap().into(),
                        )
                    }
                }
                "LDINC" => Instruction::LDINC(
                    object["operands"][0].as_str().unwrap().into(),
                    object["operands"][1].as_str().unwrap().into(),
                ),
                "LDDEC" => Instruction::LDDEC(
                    object["operands"][0].as_str().unwrap().into(),
                    object["operands"][1].as_str().unwrap().into(),
                ),
                "JR" => {
                    if Self::is_const(object["operands"][0].as_str().unwrap()).0 {
                        Instruction::JR(Condition::Always, RegisterData::from_reg(Register::Const8(0)))
                    } else {
                        Instruction::JR(object["operands"][0].as_str().unwrap().into(), RegisterData::from_reg(Register::Const8(0)))
                    }
                }
                "ADD" => Instruction::ADD(
                    object["operands"][0].as_str().unwrap().into(),
                    object["operands"][1].as_str().unwrap().into(),
                ),
                "SUB" => Instruction::SUB(object["operands"][0].as_str().unwrap().into()),
                "ADC" => Instruction::ADC(object["operands"][1].as_str().unwrap().into()),
                "SBC" => Instruction::SBC(object["operands"][1].as_str().unwrap().into()),
                "AND" => Instruction::AND(object["operands"][0].as_str().unwrap().into()),
                "XOR" => Instruction::XOR(object["operands"][0].as_str().unwrap().into()),
                "OR" => Instruction::OR(object["operands"][0].as_str().unwrap().into()),
                "CP" => Instruction::CP(object["operands"][0].as_str().unwrap().into()),
                "RET" => {
                    if object["operands"].get(0).is_none() {
                        Instruction::RET(Condition::Always)
                    } else {
                        Instruction::RET(object["operands"][0].as_str().unwrap().into())
                    }
                },
                "CALL" => {
                    if Self::is_const(object["operands"][0].as_str().unwrap()).0 {
                        Instruction::CALL(Condition::Always, RegisterData::empty())
                    } else {
                        Instruction::CALL(object["operands"][0].as_str().unwrap().into(), RegisterData::empty())
                    } 
                },
                "POP" => Instruction::POP(object["operands"][0].as_str().unwrap().into()),
                "PUSH" => Instruction::PUSH(object["operands"][0].as_str().unwrap().into()),
                "JP" => {
                    if object["opcode"].as_str().unwrap() == "0xe9" {
                        Instruction::JP(Condition::Always, RegisterData::from_reg(Register::HL))
                    } else if Self::is_const(object["operands"][0].as_str().unwrap()).0 {
                        Instruction::JP(Condition::Always, RegisterData::empty())
                    } else {
                        Instruction::JP(object["operands"][0].as_str().unwrap().into(), RegisterData::empty())
                    }
                },
                "RETI" => Instruction::RETI,
                "RST" => Instruction::RST(u16::from_str_radix(&object["operands"][0].as_str().unwrap().replace("H", ""), 10).unwrap()),
                "LDH" => Instruction::LDH(
                    object["operands"][0].as_str().unwrap().into(),
                    object["operands"][1].as_str().unwrap().into(),
                ),
                "RLC" => Instruction::RLC(object["operands"][0].as_str().unwrap().into()),
                "RRC" => Instruction::RRC(object["operands"][0].as_str().unwrap().into()),
                "RL" => Instruction::RL(object["operands"][0].as_str().unwrap().into()),
                "RR" => Instruction::RR(object["operands"][0].as_str().unwrap().into()),
                "SLA" => Instruction::SLA(object["operands"][0].as_str().unwrap().into()),
                "SRA" => Instruction::SRA(object["operands"][0].as_str().unwrap().into()),
                "SWAP" => Instruction::SWAP(object["operands"][0].as_str().unwrap().into()),
                "SRL" => Instruction::SRL(object["operands"][0].as_str().unwrap().into()),
                "BIT" => Instruction::BIT(
                    object["operands"][0].as_str().unwrap().parse().unwrap(),
                    object["operands"][1].as_str().unwrap().into()
                ),
                "RES" => Instruction::RES(
                    object["operands"][0].as_str().unwrap().parse().unwrap(),
                    object["operands"][1].as_str().unwrap().into()
                ),
                "SET" => Instruction::SET(
                    object["operands"][0].as_str().unwrap().parse().unwrap(),
                    object["operands"][1].as_str().unwrap().into()
                ),
                "PREFIX" => Instruction::PREFIX,
                _ => panic!("Instruction doesn't exist: {}", object["mnemonic"].as_str().unwrap()),
            };
            
            let code = u8::from_str_radix(&object["opcode"].as_str().unwrap().replace("0x", ""), 16).unwrap();
            let opcode = OpCode {
                code,
                instruction,
                flags,
                cycles,
                extra_data,
            };
            if object.get("prefix").is_none() {
                unprefixed.insert(code, opcode);
            } else {
                prefixed.insert(code, opcode);
            }

            i += 1;
        }

        Ok(Self {
            unprefixed,
            prefixed,
        })
    }

    fn create_flag(str: &str) -> Option<(bool, bool)> {
        match  str {
            "0" => Some((false, false)),
            "1" => Some((false, true)),
            _  => Some((true, false)),
        }
    }

    fn is_const(str: &str) -> (bool, Register) {
        match str
            .replace("(", "")
            .replace(")", "")
            .as_str()
        {
            "d16" | "a16"=> (true, Register::Const16(0)),
            "d8" | "r8" | "a8" => (true, Register::Const8(0)),
            _ => (false, Register::None),
        }
    }

}
