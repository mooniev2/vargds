use super::{cond_extract, preg};

fn blx<const IMM: bool>(instr: u32) -> String {
    format!(
        "blx {}",
        if IMM {
            let offset = ((((instr as i32 & 0xFFFFFF) << 8) >> 8) * 4).wrapping_add(8);
            format!("#0x{offset:X}")
        } else {
            preg(instr & 0xF)
        }
    )
}

pub fn blx_imm(instr: u32) -> String {
    blx::<true>(instr)
}

pub fn blx_reg(instr: u32) -> String {
    blx::<false>(instr)
}

pub fn bx(instr: u32) -> String {
    let rm = preg(instr & 0xF);
    let cond = cond_extract(instr);
    format!("bx{cond} {rm}")
}

pub fn b<const ARG: arm_decode::B>(instr: u32) -> String {
    let offset = ((((instr as i32 & 0xFFFFFF) << 8) >> 8) * 4).wrapping_add(8);
    let cond = cond_extract(instr);
    format!("b{cond}{} #0x{offset:X}", if ARG.link { "l" } else { "" })
}
