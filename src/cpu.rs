use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::fmt::{Display, Formatter};

struct Cpu {
    /// CPU registers, both as 8-bit registers and as 16-bit accessible pairs
    regs: BTreeMap<RegName, Box<dyn RegValue>>,
    /// Program counter
    pc: u16,
    /// Stack pointer
    sp: u16,
}

impl Cpu {
    fn new() -> Self {
        let mut regs: BTreeMap<RegName, Box<dyn RegValue>> = BTreeMap::new();

        // Insert the 8-bit register values
        regs.insert(RegName::B, Box::new(CpuRegister::new(RegName::B)));
        regs.insert(RegName::C, Box::new(CpuRegister::new(RegName::C)));
        regs.insert(RegName::D, Box::new(CpuRegister::new(RegName::D)));
        regs.insert(RegName::E, Box::new(CpuRegister::new(RegName::E)));
        regs.insert(RegName::H, Box::new(CpuRegister::new(RegName::H)));
        regs.insert(RegName::L, Box::new(CpuRegister::new(RegName::L)));
        regs.insert(RegName::A, Box::new(CpuRegister::new(RegName::A)));
        regs.insert(RegName::F, Box::new(CpuRegister::new(RegName::F)));

        // Insert the 16-bit register values
        regs.insert(RegName::BC, Box::new(CpuRegisterPair::new(RegName::B, RegName::C)));
        regs.insert(RegName::DE, Box::new(CpuRegisterPair::new(RegName::D, RegName::E)));
        regs.insert(RegName::HL, Box::new(CpuRegisterPair::new(RegName::H, RegName::L)));
        regs.insert(RegName::AF, Box::new(CpuRegisterPair::new(RegName::A, RegName::F)));

        Self { regs, pc: 0, sp: 0 }
    }
    
    fn set_flag(&self, flag: FlagName, value: bool) {
        let index = match flag {
            FlagName::Z => 7,
            FlagName::N => 6,
            FlagName::H => 5,
            FlagName::C => 4,
        };
        let current_value = self.regs[&RegName::F].read_u8(&self); 
        let mask = 1 << index;
        
        if value {
            self.regs[&RegName::F].write_u8(&self, current_value | mask);
        } else {
            self.regs[&RegName::F].write_u8(&self, current_value & !mask);
        }
    }
    
    fn set_flags(&self, z: bool, n: bool, h: bool, c: bool) {
        let flag_nibble = c as u8 | ((h as u8) << 1) | ((n as u8) << 2) | ((z as u8) << 3);
        let flag_value = flag_nibble << 4;
        self.regs[&RegName::F].write_u8(&self, flag_value);
    }
}

enum FlagName {
    /// Zero flag: bit is set if the result of the previous operation was zero.
    Z,
    /// Subtraction flag: bit is set if the previous operation was a subtraction.
    N,
    /// Half-carry flag: bit is set if there was a carry for the lower 4 bits of the result.
    H,
    /// Carry flag: bit is set when an 8-bit or 16-bit operation "rolls over".
    C,
}

/// Most of the CPU's registers can be accessed either as 8-bit or 16-bit instructions. For
/// example, BC is a combination of the 8-bit B and C registers.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
enum RegName {
    // 8-bit registers
    B,
    C,
    D,
    E,
    H,
    L,
    A, // Accumulator
    F, // Flags

    // 16-bit registers
    BC,
    DE,
    HL,
    AF, // Accumulator + flags
}

impl Display for RegName {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

trait RegValue {
    fn read_u8(&self, cpu: &Cpu) -> u8;
    fn read_u16(&self, cpu: &Cpu) -> u16;
    fn write_u8(&self, cpu: &Cpu,value: u8);
    fn write_u16(&self, cpu: &Cpu, value: u16);
}

struct CpuRegister {
    name: RegName,
    value: RefCell<u8>,
}

impl CpuRegister {
    fn new(name: RegName) -> Self {
        Self {
            name,
            value: RefCell::new(0),
        }
    }
}

impl RegValue for CpuRegister {
    fn read_u8(&self, _: &Cpu) -> u8 {
        *self.value.borrow()
    }

    fn read_u16(&self, _: &Cpu) -> u16 {
        panic!("Trying to read u16 from 8-bit register {}", self.name)
    }

    fn write_u8(&self, _: &Cpu, value: u8) {
        *self.value.borrow_mut() = value;
    }

    fn write_u16(&self, _: &Cpu, _: u16) {
        panic!("Trying to write u16 to 8-bit register {}", self.name)
    }
}

struct CpuRegisterPair {
    upper: RegName,
    lower: RegName,
}

impl CpuRegisterPair {
    fn new(upper: RegName, lower: RegName) -> Self {
        Self { upper, lower, }
    }
}

impl RegValue for CpuRegisterPair {
    fn read_u8(&self, _: &Cpu) -> u8 {
        panic!("Trying to read u8 from 16-bit register {}{}", self.upper, self.lower)
    }

    fn read_u16(&self, cpu: &Cpu) -> u16 {
        let upper = &*cpu.regs[&self.upper];
        let lower = &*cpu.regs[&self.upper];
        ((upper.read_u8(cpu) as u16) << 8) | (lower.read_u8(cpu) as u16)
    }

    fn write_u8(&self, _: &Cpu, _: u8) {
        panic!("Trying to write u8 to 16-bit register {}{}", self.upper, self.lower)
    }

    fn write_u16(&self, cpu: &Cpu, value: u16) {
        let upper = &*cpu.regs[&self.upper];
        let lower = &*cpu.regs[&self.upper];
        let upper_val = (value >> 8) & 0xff;
        let lower_val = value & 0xff;

        upper.write_u8(cpu, upper_val.try_into().unwrap());
        lower.write_u8(cpu, lower_val.try_into().unwrap());
    }
}