#![feature(fs_try_exists)]

use arm_decode::*;
use std::fs;
use std::path::Path;

fn main() {
    gen_arm9_lut();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=gen")
}

trait Emit {
    fn emit(&self) -> String;
}

macro_rules! impl_emit_primitive {
    ($($ty:ty),*) => {
        $(
            impl Emit for $ty {
                fn emit(&self) -> String {
                    format!("{self}")
                }
            }
        )*
    };
}

impl_emit_primitive!(bool);

macro_rules! impl_emit_enum {
    (
        $(
            $enum:ident{
                $(
                    $v:ident { $($field:ident),* }
                ),*
            }
        )*
    ) => {
        $(
            impl Emit for $enum {
                fn emit(&self) -> String {
                    let base = stringify!($enum);
                    match self {
                        $(
                            $enum::$v { $($field),* } => {
                                #[allow(unused_mut)]
                                let mut fields = String::new();
                                $(
                                    fields.push_str(&format!("{}: {},", stringify!($field), ($field).emit()));
                                )*
                                format!("{base}::{} {{ {fields} }}", stringify!($v))
                            },
                        )*
                    }
                }
            }
        )*
    };
}

impl_emit_enum! {
    DspMulTy {
        Smul { x },
        Smla { x },
        Smulw {},
        Smlaw {},
        Smlal { x }
    }
    DpOpcTy {
        And {},
        Eor {},
        Sub {},
        Rsb {},
        Add {},
        Adc {},
        Sbc {},
        Rsc {},
        Tst {},
        Teq {},
        Cmp {},
        Cmn {},
        Orr {},
        Mov {},
        Bic {},
        Mvn {}
    }
    DpOperTy {
        Imm {},
        Shft { is_reg, ty }
    }
    ShiftTy {
        Lsl {},
        Lsr {},
        Asr {},
        Ror {}
    }
    MulTy {
        Mla {},
        Mul {},
        Smlal {},
        Smull {},
        Umlal {},
        Umull {}
    }
    TransfTy {
        Byte {},
        Word {}
    }
    TransfOperTy {
        Imm {},
        Reg { shift }
    }
    AdrModeTy {
        Post {},
        Pre {},
        Offset {}
    }
    MiscTransfTy {
        SH  {},
        H { load },
        SB {}
    }
    AdrMode4 {
        IA {},
        IB {},
        DA {},
        DB {}
    }
    AdrMode5 {
        Post {},
        Unindexed {},
        Pre {},
        Offset {}
    }
}

macro_rules! impl_emit_struct {
    (
        $(
            $ty:ty {
                $($field:ident),* $(,)?
            }
        )*
    ) => {
        $(
            impl Emit for $ty {
                fn emit(&self) -> String {
                    let base = stringify!($ty);
                    let mut fields = String::new();
                    let Self { $($field),* } = self;
                    $(
                        fields.push_str(&format!("{}: {},", stringify!($field), ($field).emit()));
                    )*
                    format!("{base} {{ {fields} }}")
                }
            }
        )*
    };
}

impl_emit_struct! {
    Msr {
        r,imm
    }
    Mrs {
        r
    }
    B {
        link
    }
    QArith {
        sub, doubles
    }
    DspMul {
        ty, y
    }
    Dp {
        flags, opc, oper
    }
    Mul {
        flags, ty
    }
    Swp {
        byte
    }
    Transf {
        load, add_ofs, ty, oper, adr_ty
    }
    TransfMisc {
        add_ofs, imm, ty, adr_ty
    }
    TransfDouble {
        store, add_ofs, imm, adr_ty
    }
    TransfMult {
        load, base_update, adr_ty, privilige_mode
    }
    CpMov {
        arm_reg_load
    }
    CpTransf {
        load, adr_ty, add_ofs, n
    }
}

fn gen_arm9_lut() {
    macro_rules! emit {
        ($mod:pat, $ident:ident) => {
            stringify!($mod::$ident).to_string()
        };
        ($mod:pat, $ident:ident, $expr:expr) => {
            format!(
                "{}::{}::<{{ {} }}>",
                stringify!($mod),
                stringify!($ident),
                ($expr).emit()
            )
        };
    }
    let build_dir = Path::new("gen");
    let cond_lut_path = Path::join(build_dir, "arm9_cond_lut.txt");
    let uncond_lut_path = Path::join(build_dir, "arm9_uncond_lut.txt");
    if !fs::try_exists(build_dir).expect("") {
        fs::create_dir(build_dir).expect("unable to create directory")
    }

    let processor = arm_decode::Processor {};
    if !fs::try_exists(cond_lut_path.clone()).expect("") {
        let mut cond_lut_str = String::new();
        for i in 0..(1 << 12) {
            let instr = ((i & 0xF) << 4) | ((i & 0xFF0) << 16);
            cond_lut_str.push_str(&match processor.decode_cond(instr) {
                CondInstr::Msr(e) => emit!(data, msr, e),
                CondInstr::Mrs(e) => emit!(data, mrs, e),
                CondInstr::Bx => emit!(branch, bx),
                CondInstr::BlxReg => emit!(branch, blx_reg),
                CondInstr::B(e) => emit!(branch, b, e),
                CondInstr::Clz => emit!(data, clz),
                CondInstr::QArith(e) => emit!(data, qarith, e),
                CondInstr::DspMul(e) => emit!(data, dsp_mul, e),
                CondInstr::Bkpt => emit!(misc, bkpt),
                CondInstr::Dp(e) => emit!(data, dp, e),
                CondInstr::Mul(e) => emit!(data, mul, e),
                CondInstr::Swp(e) => emit!(data, swp, e),
                CondInstr::Transf(e) => emit!(mem, transf, e),
                CondInstr::TransfMisc(e) => emit!(mem, transf_misc, e),
                CondInstr::TransfDouble(e) => emit!(mem, transf_double, e),
                CondInstr::TransfMult(e) => emit!(mem, transf_mult, e),
                CondInstr::CpDp => emit!(data, cp_dp),
                CondInstr::CpMov(e) => emit!(data, cp_mov, e),
                CondInstr::CpTransf(e) => emit!(mem, cp_transf, e),
                CondInstr::Swi => emit!(misc, swi),
                CondInstr::Undef => emit!(misc, undef),
                CondInstr::Unpred => emit!(misc, unpred),
            });
            cond_lut_str.push(',');
        }
        fs::write(cond_lut_path, format!("[{cond_lut_str}]")).expect("failed to write LUT to file");
    }
    if !fs::try_exists(uncond_lut_path.clone()).expect("") {
        let processor = arm_decode::Processor {};
        let mut uncond_lut_str = String::new();
        for i in 0..(1 << 12) {
            let instr = ((i & 0xF) << 4) | ((i & 0xFF0) << 16);
            uncond_lut_str.push_str(&match processor.decode_uncond(instr) {
                UnCondInstr::BlxImm => emit!(branch, blx_imm),
                UnCondInstr::Undef => emit!(misc, undef),
            });
            uncond_lut_str.push(',');
        }
        fs::write(uncond_lut_path, format!("[{uncond_lut_str}]"))
            .expect("failed to write LUT to file");
    }
}
