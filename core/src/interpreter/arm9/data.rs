use crate::{Core, Interpreter};
use arm_decode::*;

pub fn dp<const ARG: Dp>(core: &mut Core<Interpreter>, instr: u32) {
    #[inline(always)]
    fn alu_adc(
        core: &mut Core<Interpreter>,
        update_flags: bool,
        oper0: u32,
        oper1: u32,
        carry_in: bool,
    ) -> u32 {
        let val = oper0 as u64 + oper1 as u64 + carry_in as u64;
        let carry = val >> 32 != 0;
        let val = val as u32;
        let signed_carry = (oper0 >> 31) & 0b1 != (val >> 31) & 0b1;
        if update_flags {
            let cpsr = &mut core.arm9.cpsr;
            cpsr.n_set((val as i32).is_negative());
            cpsr.z_set(val == 0);
            cpsr.c_set(carry);
            cpsr.v_set(signed_carry);
        }
        val
    }

    #[inline(always)]
    fn alu_sbc(
        core: &mut Core<Interpreter>,
        update_flags: bool,
        oper0: u32,
        oper1: u32,
        carry_in: Option<bool>,
    ) -> u32 {
        alu_adc(
            core,
            update_flags,
            oper0,
            (!oper1).wrapping_add(1),
            !if let Some(c) = carry_in { c } else { false },
        )
    }

    #[inline(always)]
    fn alu_op(core: &mut Core<Interpreter>, update_flags: bool, oper_c: bool, val: u32) {
        if update_flags {
            let cpsr = &mut core.arm9.cpsr;
            cpsr.n_set((val as i32).is_negative());
            cpsr.z_set(val == 0);
            cpsr.c_set(oper_c);
        }
    }
    let (oper, oper_c) = match ARG.oper {
        arm_decode::DpOperTy::Imm => {
            let imm = instr & 0xFF;
            let rotate = (instr >> 8) & 0xF;
            (imm.rotate_right(rotate * 2), false)
        }
        arm_decode::DpOperTy::Shft { is_reg, ty } => {
            let rm = core.arm9.gpr(instr as usize & 0xF);
            let shift = if is_reg {
                let rs = (instr >> 8) & 0xF;
                core.arm9.gpr(rs as usize)
            } else {
                (instr >> 7) & 0x1F
            };
            match ty {
                arm_decode::ShiftTy::Lsl => u32::overflowing_shl(rm, shift),
                arm_decode::ShiftTy::Lsr => u32::overflowing_shr(rm, shift),
                arm_decode::ShiftTy::Asr => (((rm as i32) >> shift) as u32, false),
                arm_decode::ShiftTy::Ror => (rm.rotate_right(shift), false),
            }
        }
    };

    let rdi = (instr >> 12) as usize & 0xF;
    let rn = core.arm9.gpr(((instr as usize) >> 16) & 0xF);
    let update_flags = ARG.flags;
    match ARG.opc {
        DpOpcTy::And => {
            let val = rn & oper;
            alu_op(core, update_flags, oper_c, val);
            core.arm9.gpr_set(rdi, val);
        }
        DpOpcTy::Eor => {
            let val = rn ^ oper;
            alu_op(core, update_flags, oper_c, val);
            core.arm9.gpr_set(rdi, val);
        }
        DpOpcTy::Sub => {
            let val = alu_sbc(core, update_flags, rn, oper, None);
            core.arm9.gpr_set(rdi, val);
        }
        DpOpcTy::Rsb => {
            let val = alu_sbc(core, update_flags, oper, rn, None);
            core.arm9.gpr_set(rdi, val);
        }
        DpOpcTy::Add => {
            let val = alu_adc(core, update_flags, rn, oper, false);
            core.arm9.gpr_set(rdi, val)
        }
        DpOpcTy::Adc => {
            let val = alu_adc(core, update_flags, rn, oper, core.arm9.cpsr.c());
            core.arm9.gpr_set(rdi, val)
        }
        DpOpcTy::Sbc => {
            let val = alu_sbc(core, update_flags, rn, oper, Some(core.arm9.cpsr.c()));
            core.arm9.gpr_set(rdi, val);
        }
        DpOpcTy::Rsc => {
            let val = alu_sbc(core, update_flags, oper, rn, Some(core.arm9.cpsr.c()));
            core.arm9.gpr_set(rdi, val);
        }
        DpOpcTy::Tst => {
            let val = rn & oper;
            alu_op(core, update_flags, oper_c, val);
        }
        DpOpcTy::Teq => {
            let val = rn ^ oper;
            alu_op(core, update_flags, oper_c, val);
        }
        DpOpcTy::Cmp => {
            let _ = alu_sbc(core, true, rn, oper, None);
        }
        DpOpcTy::Cmn => {
            let _ = alu_adc(core, true, rn, oper, false);
        }
        DpOpcTy::Orr => {
            let val = rn | oper;
            alu_op(core, update_flags, oper_c, val);
            core.arm9.gpr_set(rdi, val)
        }
        DpOpcTy::Mov => {
            let val = oper;
            alu_op(core, update_flags, oper_c, val);
            core.arm9.gpr_set(rdi, val);
        }
        DpOpcTy::Bic => {
            let val = rn & !(oper);
            alu_op(core, update_flags, oper_c, val);
            core.arm9.gpr_set(rdi, val);
        }
        DpOpcTy::Mvn => {
            let val = !oper;
            alu_op(core, update_flags, oper_c, val);
            core.arm9.gpr_set(rdi, val);
        }
    }
}

pub fn clz(core: &mut Core<Interpreter>, instr: u32) {
    let rm = core.arm9.gpr(instr as usize & 0xF);
    let leading_zeros = rm.leading_zeros();
    let rdi = (instr >> 12) & 0xF;
    core.arm9.gpr_set(rdi as usize, leading_zeros);
}

pub fn msr<const ARG: Msr>(core: &mut Core<Interpreter>, instr: u32) {
    unimplemented!()
}

pub fn mrs<const ARG: Mrs>(core: &mut Core<Interpreter>, instr: u32) {
    unimplemented!()
}

pub fn mul<const ARG: Mul>(core: &mut Core<Interpreter>, instr: u32) {
    let rdi = (instr >> 16) & 0xF;
    let rn = (instr >> 12) & 0xF;
    let rs = (instr >> 8) & 0xF;
    let rm = instr & 0xF;
    let rdihi = rdi as usize;
    let rdilo = rn as usize;
    let rdlo = core.arm9.gpr(rdilo);
    let rdhi = core.arm9.gpr(rdihi);
    let val = match ARG.ty {
        MulTy::Mla => rm.wrapping_mul(rs).wrapping_add(rn),
        MulTy::Mul => rm.wrapping_mul(rs),
        MulTy::Smlal => {
            let product = (rm as i32 as i64) * (rs as i32 as i64);
            let hi = ((product >> 32) as u32).wrapping_add(rdhi);
            let lo = (product as u32).wrapping_add(rdlo);
            core.arm9.gpr_set(rdilo, lo);
            hi
        }
        MulTy::Smull => {
            let product = (rm as i32 as i64) * (rs as i32 as i64);
            let hi = (product >> 32) as u32;
            let lo = product as u32;
            core.arm9.gpr_set(rdilo, lo);
            hi
        }
        MulTy::Umlal => {
            let product = (rm as u64) * (rs as u64);
            let lo = (product as u32).wrapping_add(rdlo);
            let hi = ((product >> 32) as u32).wrapping_add(rdhi);
            core.arm9.gpr_set(rdilo, lo);
            hi
        }
        MulTy::Umull => {
            let product = (rm as u64) * (rs as u64);
            let lo = product as u32;
            let hi = (product >> 32) as u32;
            core.arm9.gpr_set(rdilo, lo);
            hi
        }
    };
    if ARG.flags {
        let cpsr = &mut core.arm9.cpsr;
        cpsr.n_set((val as i32).is_negative());
        cpsr.z_set(val == 0);
    }
    core.arm9.gpr_set(rdi as usize, val);
}

pub fn swp<const SWP: Swp>(_core: &mut Core<Interpreter>, instr: u32) {
    unimplemented!()
}

pub fn qarith<const QARITH: QArith>(_core: &mut Core<Interpreter>, instr: u32) {
    unimplemented!()
}

pub fn dsp_mul<const DSPMUL: DspMul>(_core: &mut Core<Interpreter>, instr: u32) {
    unimplemented!()
}

pub fn cp_dp(_core: &mut Core<Interpreter>, _instr: u32) {
    unimplemented!()
}

pub fn cp_mov<const CP_MOV: CpMov>(_core: &mut Core<Interpreter>, _instr: u32) {
    unimplemented!()
}
