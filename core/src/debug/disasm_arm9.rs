use crate::{Core, Engine};

mod branch;
mod data;
mod mem;
mod misc;

static COND_INSTR_LUT: [fn(u32) -> String; 4096] = {
    use arm_decode::*;

    include!("../../gen/arm9_cond_lut.txt")
};

static UNCOND_INSTR_LUT: [fn(u32) -> String; 4096] = include!("../../gen/arm9_uncond_lut.txt");

#[inline]
fn preg(reg: u32) -> String {
    debug_assert!(reg < 0x10);
    match reg & 0xF {
        v @ 0..=12 => format!("r{v}"),
        13 => "sp".to_string(),
        14 => "lr".to_string(),
        15 => "pc".to_string(),
        _ => unreachable!(),
    }
}

#[inline]
fn pcond(nibble: u32) -> &'static str {
    debug_assert!(nibble < 0x10);
    match nibble & 0xF {
        0x0 => "eq",
        0x1 => "ne",
        0x2 => "cs",
        0x3 => "cc",
        0x4 => "mi",
        0x5 => "pl",
        0x6 => "vs",
        0x7 => "vc",
        0x8 => "hi",
        0x9 => "ls",
        0xA => "ge",
        0xB => "lt",
        0xC => "gt",
        0xD => "le",
        0xE => "",
        _ => unreachable!(),
    }
}

#[inline]
fn cond_extract(instr: u32) -> &'static str {
    pcond(instr >> 28)
}

pub fn disassemble(instr: u32) -> String {
    let index = ((instr >> 4) & 0xF) | ((instr >> 16) & 0xFF0);
    if (instr >> 28) == 0xF {
        UNCOND_INSTR_LUT[index as usize](instr)
    } else {
        COND_INSTR_LUT[index as usize](instr)
    }
}

pub fn disassemble_at_adr<E: Engine>(core: &mut Core<E>, adr: u32) -> (u32, String) {
    let read = crate::cpu::arm9::bus::read32_dbg(core, adr);
    (read, disassemble(read))
}
