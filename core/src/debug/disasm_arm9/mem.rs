use super::{cond_extract, pcond, preg};
use arm_decode::*;

#[inline(always)]
fn adr_m_3(instr: u32, rn: u32, add_ofs: bool, imm: bool, adr_ty: AdrModeTy) -> String {
    let rn = preg(rn & 0xF);
    let mut show_oper = true;
    let oper = format!(
        "{}{}",
        if add_ofs { "" } else { "-" },
        if imm {
            let imm = (instr & 0xF) | ((instr >> 4) & 0xF0);
            show_oper = imm > 0;
            format!("0x{imm:02X}")
        } else {
            preg(instr & 0xF)
        }
    );
    if show_oper {
        match adr_ty {
            AdrModeTy::Post => format!("[{rn}], {oper}"),
            AdrModeTy::Pre => todo!("[{rn}, {oper}]!"),
            AdrModeTy::Offset => format!("[{rn}, {oper}]"),
        }
    } else {
        format!("[{rn}]")
    }
}

pub fn transf<const ARG: Transf>(instr: u32) -> String {
    let cond = cond_extract(instr);
    let rd = preg((instr >> 12) & 0xF);
    let rn = (instr >> 16) & 0xF;
    let (operand, transform) = {
        let sign = if ARG.add_ofs { "" } else { "-" };
        let operand = match ARG.oper {
            TransfOperTy::Imm => {
                let imm = instr & 0xFFF;
                format!(", {sign}#0x{imm:X}")
            }
            TransfOperTy::Reg { shift } => {
                let rm = preg(instr & 0xF);
                let shift_imm = (instr >> 7) & 0x1F;
                match shift {
                    ShiftTy::Lsl => {
                        if shift_imm == 0 {
                            "".to_string()
                        } else {
                            format!(", {sign}{rm}, lsl {shift_imm:X}")
                        }
                    }
                    ShiftTy::Lsr => format!(", {sign}{rm}, lsr {shift_imm:X}"),
                    ShiftTy::Asr => format!(", {sign}{rm}, asr {shift_imm:X}"),
                    ShiftTy::Ror => format!(", {sign}{rm}, ror {shift_imm:X}"),
                }
            }
        };
        let base = preg(rn & 0xF);
        match ARG.adr_ty {
            AdrModeTy::Post => (format!("[{base}]{operand}"), (instr >> 21) & 0b1 != 0),
            AdrModeTy::Pre => (format!("[{base}{operand}]!"), false),
            AdrModeTy::Offset => (format!("[{base}{operand}]"), false),
        }
    };
    format!(
        "{}{}{cond}{} {rd}, {operand}",
        if ARG.load { "ldr" } else { "str" },
        match ARG.ty {
            TransfTy::Byte => "b",
            TransfTy::Word => "",
        },
        if transform { "t" } else { "" }
    )
}

pub fn transf_misc<const ARG: TransfMisc>(instr: u32) -> String {
    let rd = preg((instr >> 12) & 0xF);
    let rn = (instr >> 16) & 0xF;
    let oper = adr_m_3(instr, rn, ARG.add_ofs, ARG.imm, ARG.adr_ty);
    let cond = cond_extract(instr);
    let (pre, post) = match ARG.ty {
        MiscTransfTy::SH => ("ldr", "sh"),
        MiscTransfTy::H { load } => (if load { "ldr" } else { "str" }, "h"),
        MiscTransfTy::SB => ("ldr", "sb"),
    };
    format!("{pre}{cond}{post} {rd}, {oper}")
}

pub fn transf_mult<const ARG: TransfMult>(instr: u32) -> String {
    let rn = preg((instr >> 16) & 0xF);
    let register_list = {
        let mut len = 0;
        let mut start = 0;
        let mut print = String::new();
        let add_reg_single = #[inline(always)]
        |index: u32, str: &mut String| {
            let reg = index & 0xF;
            str.push_str(&format!("r{reg},"));
        };
        let add_reg_multiple = #[inline(always)]
        |start: u32, len: u32, str: &mut String| {
            let reg_lower = start & 0xF;
            let reg_upper = (start + len) & 0xF;
            str.push_str(&format!("r{reg_lower} - r{reg_upper},"));
        };
        for i in 0..16 {
            let active = instr & (1 << i) != 0;
            if active {
                len += 1;
            } else if len == 0 {
                len = 0;
                start = i + 1;
            } else if len == 1 {
                add_reg_single(start, &mut print);
                start = i;
                len = 0;
            } else {
                add_reg_multiple(start, len + 1, &mut print);
                start = i;
                len = 0;
            }
        }
        if len == 1 {
            add_reg_single(start, &mut print);
        } else if len > 1 {
            add_reg_multiple(start, len + 1, &mut print);
        }
        print.pop();
        format!("{{{print}}}")
    };
    let cond = cond_extract(instr);
    format!(
        "{}{cond}{} {rn}{}, {register_list}",
        if ARG.load { "ldm" } else { "stm" },
        match ARG.adr_ty {
            AdrMode4::IA => "ia",
            AdrMode4::IB => "ib",
            AdrMode4::DA => "da",
            AdrMode4::DB => "db",
        },
        if ARG.base_update { "!" } else { "" },
    )
}

pub fn transf_double<const ARG: TransfDouble>(instr: u32) -> String {
    let start = (instr >> 12) & 0xE; // ignore lower bit.
    let rn = (instr >> 16) & 0xF;
    let lo = preg(start);
    let hi = preg(start + 1);
    let oper = adr_m_3(instr, rn, ARG.add_ofs, ARG.imm, ARG.adr_ty);
    let cond = cond_extract(instr);
    format!(
        "{}{cond}d, {{{lo},{hi}}}, {oper}",
        if ARG.store { "str" } else { "ldr" }
    )
}

pub fn cp_transf<const ARG: CpTransf>(instr: u32) -> String {
    let rn = preg((instr >> 16) & 0xF);
    let crd = (instr >> 12) & 0xF;
    let proc = (instr >> 8) & 0xF;
    let imm = instr & 0xFF;
    let option = imm;
    let offset = imm << 2;
    let offset = if ARG.add_ofs {
        format!("0x{offset:X}")
    } else {
        format!("-0x{offset:X}")
    };
    let l = if ARG.n { "l" } else { "" };
    let cond_value = (instr >> 28) & 0xF;
    let if_cond = cond_value < 0xF;
    let cond = pcond(cond_value);
    format!(
        "{}{l} p{proc}, c{crd}, {}",
        if ARG.load {
            if if_cond {
                format!("ldc{cond}")
            } else {
                "ldc2".to_string()
            }
        } else if if_cond {
            format!("stc2{cond}")
        } else {
            "stc2".to_string()
        },
        match ARG.adr_ty {
            AdrMode5::Post => format!("[{rn}], {offset}"),
            AdrMode5::Unindexed => format!("[{rn}], 0x{option:08X}"),
            AdrMode5::Pre => format!("[{rn}, {offset}]!"),
            AdrMode5::Offset => format!("[{rn}, {offset}]"),
        }
    )
}
