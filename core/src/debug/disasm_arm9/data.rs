use super::{cond_extract, pcond, preg};
use arm_decode::*;

pub fn dp<const ARG: Dp>(instr: u32) -> String {
    let rd_i = (instr >> 12) & 0xF;
    let rn_i = (instr >> 16) & 0xF;
    let rd = preg(rd_i);
    let rn = preg(rn_i);
    let regs = if rd_i == rn_i {
        rd.to_string()
    } else {
        format!("{rd}, {rn}")
    };
    let ((operand, mov_oper), mov) = match ARG.oper {
        DpOperTy::Imm => {
            let rotate = (instr >> 8) & 0xF;
            let imm = instr & 0xFF;
            let operand = format!("#0x{:X}", imm.rotate_right(rotate * 2));
            ((operand.clone(), operand), "mov")
        }
        DpOperTy::Shft { is_reg, ty } => {
            let rm = preg(instr & 0xF);
            let shift_print = match ty {
                ShiftTy::Lsl => "lsl",
                ShiftTy::Lsr => "lsr",
                ShiftTy::Asr => "asr",
                ShiftTy::Ror => "ror",
            };
            (
                if is_reg {
                    let rs = preg((instr >> 8) & 0xF);
                    (format!("{rm}, {shift_print} {rs}"), format!("{rm}, {rs}"))
                } else {
                    let imm = (instr >> 7) & 0x1F;
                    (
                        if imm != 0 {
                            format!("{rm}, {shift_print} #0x{imm:X}")
                        } else {
                            rm.clone()
                        },
                        if imm != 0 {
                            format!("{rm}, #0x{imm:X}")
                        } else {
                            rm
                        },
                    )
                },
                shift_print,
            )
        }
    };
    let s = if ARG.flags { "s" } else { "" };
    let cond = cond_extract(instr);
    match ARG.opc {
        DpOpcTy::And => format!("and{cond}{s} {regs}, {operand}"),
        DpOpcTy::Eor => format!("eor{cond}{s} {regs}, {operand}"),
        DpOpcTy::Sub => format!("sub{cond}{s} {regs}, {operand}"),
        DpOpcTy::Rsb => format!("rsb{cond}{s} {regs}, {operand}"),
        DpOpcTy::Add => format!("add{cond}{s} {regs}, {operand}"),
        DpOpcTy::Adc => format!("adc{cond}{s} {regs}, {operand}"),
        DpOpcTy::Sbc => format!("sbc{cond}{s} {regs}, {operand}"),
        DpOpcTy::Rsc => format!("rsc{cond}{s} {regs}, {operand}"),
        DpOpcTy::Tst => format!("tst{cond} {regs}, {operand}"),
        DpOpcTy::Teq => format!("teq{cond} {regs}, {operand}"),
        DpOpcTy::Cmp => format!("cmp{cond} {regs}, {operand}"),
        DpOpcTy::Cmn => format!("cmn{cond} {regs}, {operand}"),
        DpOpcTy::Orr => format!("orr{cond}{s} {regs}, {operand}"),
        DpOpcTy::Bic => format!("bic{cond}{s} {regs}, {operand}"),
        DpOpcTy::Mvn => format!("mvn{cond}{s} {regs}, {operand}"),
        DpOpcTy::Mov => format!("{mov}{cond}{s} {rd}, {mov_oper}"),
    }
}

pub fn clz(instr: u32) -> String {
    let rd = preg((instr >> 12) & 0xF);
    let rm = preg(instr & 0xF);
    let cond = cond_extract(instr);
    format!("clz{cond} {rd}, {rm}")
}

pub fn msr<const ARG: Msr>(instr: u32) -> String {
    let psr = if ARG.r { "spsr" } else { "cpsr" };
    let arg = if ARG.imm {
        let imm = instr & 0xF;
        let rot = (instr >> 8) & 0xF;
        let val = imm.rotate_right(rot * 2);
        format!("0x{val:X}")
    } else {
        preg(instr & 0xF)
    };
    let cond = cond_extract(instr);
    format!("msr{cond} {psr}_, {arg}")
}

pub fn mrs<const ARG: Mrs>(instr: u32) -> String {
    let rd = preg((instr >> 12) & 0xF);
    let cond = cond_extract(instr);
    format!("mrs{cond} {rd}, {}", if ARG.r { "spsr" } else { "cpsr" })
}

pub fn mul<const ARG: Mul>(instr: u32) -> String {
    let rm = preg((instr >> 8) & 0xF);
    let rs = preg((instr >> 8) & 0xF);
    let rd = preg((instr >> 16) & 0xF);
    let rn = preg((instr >> 12) & 0xF);
    let rdhi = preg((instr >> 16) & 0xF);
    let rdlo = preg((instr >> 12) & 0xF);
    let flags = if ARG.flags { "s" } else { "" };
    let cond = cond_extract(instr);
    match ARG.ty {
        MulTy::Mla => format!("mla{cond}{flags} {rd}, {rm}, {rs}, {rn}"),
        MulTy::Mul => format!("mul{cond}{flags} {rd}, {rm}, {rs}"),
        MulTy::Smlal => format!("smlal{cond}{flags} {rdlo}, {rdhi}, {rm}, {rs}"),
        MulTy::Smull => format!("smull{cond}{flags} {rdlo}, {rdhi}, {rm}, {rs}"),
        MulTy::Umlal => format!("umlal{cond}{flags} {rdlo}, {rdhi}, {rm}, {rs}"),
        MulTy::Umull => format!("umull{cond}{flags} {rdlo}, {rdhi}, {rm}, {rs}"),
    }
}

pub fn qarith<const ARG: QArith>(instr: u32) -> String {
    let opc_m = (ARG.sub, ARG.doubles);
    let opc = match opc_m {
        (true, true) => "qdsub",
        (true, false) => "qsub",
        (false, true) => "qdadd",
        (false, false) => "qadd",
    };
    let rd_index = (instr >> 12) & 0xF;
    let rm_index = instr & 0xF;
    let rd = preg(rd_index);
    let rm = preg(rm_index);
    let rn = preg((instr >> 16) & 0xF);
    let cond = cond_extract(instr);
    format!(
        "{opc}{cond} {rd}, {}",
        if rd_index != rm_index {
            format!("{rm}, {rn}")
        } else {
            rn
        }
    )
}

pub fn dsp_mul<const ARG: DspMul>(instr: u32) -> String {
    let arg_ty = #[inline(always)]
    |b: bool| {
        if b {
            "T"
        } else {
            "B"
        }
    };
    let y = arg_ty(ARG.y);
    let cond = cond_extract(instr);
    let rd = preg((instr >> 16) & 0xF);
    let rn = preg((instr >> 12) & 0xF);
    let rs = preg((instr >> 8) & 0xF);
    let rm = preg(instr & 0xF);
    match ARG.ty {
        DspMulTy::Smul { x } => format!("smul{}{y}{cond} {rd}, {rm}, {rs}", arg_ty(x)),
        DspMulTy::Smla { x } => format!("smla{}{y}{cond} {rd}, {rm}, {rs}, {rn}", arg_ty(x)),
        DspMulTy::Smulw => format!("smulw{y}{cond} {rd}, {rm}, {rs}"),
        DspMulTy::Smlaw => format!("smlaw{y}{cond} {rd}, {rm}, {rs}, {rn}"),
        DspMulTy::Smlal { x } => format!("smlal{}{y}{cond} {rn}, {rd}, {rm}, {rs}", arg_ty(x)),
    }
}

pub fn cp_dp(instr: u32) -> String {
    let cond_val = (instr >> 28) & 0xF;
    let cond = pcond(cond_val & 0xF);
    let crn = (instr >> 16) & 0xF;
    let crd = (instr >> 12) & 0xF;
    let crm = instr & 0xF;
    let opcode_1 = (instr >> 20) & 0xF;
    let opcode_2 = (instr >> 5) & 0b111;
    let proc = (instr >> 8) & 0xF;
    if cond_val == 0xF {
        format!("cdp2 p{proc}, {opcode_1}, c{crd}, c{crn}, c{crm}, {opcode_2}")
    } else {
        format!("cdp{cond} p{proc}, {opcode_1}, c{crd}, c{crn}, c{crm}, {opcode_2}")
    }
}

pub fn cp_mov<const ARG: CpMov>(instr: u32) -> String {
    let crn = (instr >> 16) & 0xF;
    let crm = instr & 0xF;
    let rd = preg((instr >> 12) & 0xF);
    let opcode_1 = (instr >> 21) & 0b111;
    let opcode_2 = (instr >> 5) & 0b111;
    let proc = (instr >> 8) & 0xF;
    let cond_val = (instr >> 28) & 0xF;
    let if_cond = cond_val < 0xF;
    let cond = pcond(cond_val & 0xF);
    format!(
        "{} p{proc}, {opcode_1}, {rd}, c{crn}, c{crm}, {opcode_2}",
        if ARG.arm_reg_load {
            if if_cond {
                format!("mrc{cond}")
            } else {
                "mrc2".to_string()
            }
        } else if if_cond {
            format!("mcr{cond}")
        } else {
            "mcr2".to_string()
        },
    )
}
