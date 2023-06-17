#![no_std]

extern crate alloc;
extern crate self as arm_decode;

pub struct Processor {}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DpOpcTy {
    And,
    Eor,
    Sub,
    Rsb,
    Add,
    Adc,
    Sbc,
    Rsc,
    Tst,
    Teq,
    Cmp,
    Cmn,
    Orr,
    Mov,
    Bic,
    Mvn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DpOperTy {
    Imm,
    Shft { is_reg: bool, ty: ShiftTy },
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShiftTy {
    Lsl,
    Lsr,
    Asr,
    Ror,
}

impl ShiftTy {
    const fn from_bits(bits: u32) -> Self {
        assert!(bits < 0b100);
        use ShiftTy::*;
        match bits {
            0b00 => Lsl,
            0b01 => Lsr,
            0b10 => Asr,
            0b11 => Ror,
            _ => panic!("unrachable"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DspMulTy {
    Smul { x: bool },
    Smla { x: bool },
    Smulw,
    Smlaw,
    Smlal { x: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransfTy {
    Byte,
    Word,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransfOperTy {
    Imm,
    Reg { shift: ShiftTy },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiscTransfTy {
    /// Signed halfword.
    SH,
    /// Unsigned halfword.
    H { load: bool },
    /// Signed byte.
    SB,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiscTransfOper {
    Imm,
    Reg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MulTy {
    Mla,
    Mul,
    Smlal,
    Smull,
    Umlal,
    Umull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdrModeTy {
    Post,
    Pre,
    Offset,
}

impl AdrModeTy {
    const fn from_w_p(w: bool, p: bool) -> AdrModeTy {
        use AdrModeTy::*;
        if p {
            if w {
                Pre
            } else {
                Offset
            }
        } else {
            Post
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdrMode4 {
    /// Increment after.
    IA,
    /// Increment before.
    IB,
    /// Decrement after.
    DA,
    /// Decrement before.
    DB,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdrMode5 {
    Post,
    Unindexed,
    Pre,
    Offset,
}

macro_rules! define_uncond {
    (
        pub enum $uncond:ident {
            $(
                struct $v0:ident {
                    $(
                        $(#[doc = $doc_str:literal])?
                        $field:ident : $ty:ty
                    ),* $(,)?
                },
            )*
            $(enum $v1:ident,)*
        }
    ) => {
        pub enum $uncond{
            $($v0($v0),)*
            $($v1,)*
        }
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub struct $v0 {
                $(
                    $(#[doc = $doc_str])?
                    pub $field : $ty
                ),*
            }
        )*
    };
}

define_uncond! {
    pub enum CondInstr {
        struct Msr {
            r: bool,
            imm: bool,
        },
        struct Mrs {
            r: bool,
        },
        struct B {
            link: bool,
        },
        struct QArith {
            sub: bool,
            doubles: bool,
        },
        struct DspMul {
            ty: DspMulTy,
            y: bool,
        },
        struct Dp {
            flags: bool,
            opc: DpOpcTy,
            oper: DpOperTy,
        },
        struct Mul {
            flags: bool,
            ty: MulTy,
        },
        struct Swp {
            byte: bool,
        },
        struct Transf {
            load: bool,
            /// Add the offset (true) or subtract the offset (false).
            add_ofs: bool,
            ty: TransfTy,
            oper: TransfOperTy,
            adr_ty: AdrModeTy,
        },
        struct TransfMisc {
            /// Add the offset (true) or subtract the offset (false).
            add_ofs: bool,
            imm: bool,
            ty: MiscTransfTy,
            adr_ty: AdrModeTy,
        },
        struct TransfDouble {
            store: bool,
            /// Add the offset (true) or subtract the offset (false).
            add_ofs: bool,
            imm: bool,
            adr_ty: AdrModeTy,
        },
        struct TransfMult {
            load: bool,
            base_update: bool,
            adr_ty: AdrMode4,
            privilige_mode: bool,
        },
        struct CpMov {
            /// If the operation loads into an ARM register, or a coprocessor register.
            arm_reg_load: bool,
        },
        struct CpTransf {
            load: bool,
            adr_ty: AdrMode5,
            /// Add the offset (true) or subtract the offset (false).
            add_ofs: bool,
            /// Coprocessor dependent.
            n: bool,
        },
        enum Bx,
        enum BlxReg,
        enum Clz,
        enum Bkpt,
        enum CpDp,
        enum Swi,
        enum Undef,
        enum Unpred,
    }
}

pub enum UnCondInstr {
    BlxImm,
    Undef,
}

macro_rules! b {
    ($b:literal) => {
        (1 << $b)
    };
}

impl Processor {
    pub const fn decode_cond(&self, instr: u32) -> CondInstr {
        let instr = instr & 0x0FFFFFFF;
        // Miscellaneous instructions (3-3)
        if instr & 0x0F90_0010 == 0x0100_0000 || instr & 0x0F90_0090 == 0x0100_0010 {
            // Some miscelanous instructions must be handled before DP instructions due
            // to a 'hole' within the encoding table caused by the opcode field being equal
            // to 0b10xx while S is zero.
            let bit_7 = instr & b!(7) == 0;
            let bits = (instr >> 4) & 0b111;
            let upper = (instr >> 21) & 0b11;
            if bit_7 {
                match bits {
                    0b000 => {
                        let r = upper & b!(1) != 0;
                        // Move status register to register.
                        if upper & 0b01 == 0 {
                            CondInstr::Mrs(Mrs { r })
                        }
                        // Move register to status register.
                        else {
                            CondInstr::Msr(Msr { r, imm: false })
                        }
                    }
                    0b001 => {
                        // Branch/exchange instruction set.
                        if upper == 0b01 {
                            CondInstr::Bx
                        }
                        // Count leading zeros.
                        else if upper == 0b11 {
                            CondInstr::Clz
                        } else {
                            CondInstr::Undef
                        }
                    }
                    0b011 => {
                        // Branch and link/exchange instruction set.
                        if upper == 0b01 {
                            CondInstr::BlxReg
                        } else {
                            CondInstr::Undef
                        }
                    }
                    0b101 => {
                        // Enhanced DSP add/subtracts.
                        let sub = upper & b!(1) != 0;
                        let doubles = upper & b!(0) != 0;
                        CondInstr::QArith(QArith { sub, doubles })
                    }
                    0b111 => {
                        // Software Breakpoint.
                        if upper == 0b01 {
                            CondInstr::Bkpt
                        } else {
                            CondInstr::Undef
                        }
                    }
                    _ => CondInstr::Undef,
                }
            } else {
                // Enhanced DSP multiples.
                if bits & 0b001 == 0 {
                    let x = bits & 0b010 != 0;
                    let y = bits & 0b100 != 0;
                    CondInstr::DspMul(DspMul {
                        ty: match upper {
                            0b00 => DspMulTy::Smla { x },
                            0b01 => {
                                if x {
                                    DspMulTy::Smulw
                                } else {
                                    DspMulTy::Smlaw
                                }
                            }
                            0b10 => DspMulTy::Smlal { x },
                            0b11 => DspMulTy::Smul { x },
                            _ => panic!("unreachable"),
                        },
                        y,
                    })
                } else {
                    CondInstr::Undef
                }
            }
        }
        // Multiples, extra load/stores
        else if instr & 0x0E00_0090 == 0x0000_0090 {
            let bits = (instr >> 4) & 0b1111;
            match bits {
                0b1001 => {
                    let bits = (instr >> 20) & 0b11111;
                    if instr & b!(24) != 0 {
                        let bits = (instr >> 22) & 0b11;
                        let acc = instr & b!(21) != 0;
                        let set_flags = instr & b!(20) != 0;
                        let ty = if bits & 0b11 == 0b00 {
                            if acc {
                                MulTy::Mla
                            } else {
                                MulTy::Mul
                            }
                        } else if bits & 0b10 == 0b10 {
                            let signed = instr & b!(22) != 0;
                            if acc {
                                if signed {
                                    MulTy::Smlal
                                } else {
                                    MulTy::Umlal
                                }
                            } else if signed {
                                MulTy::Smull
                            } else {
                                MulTy::Umull
                            }
                        } else {
                            return CondInstr::Undef;
                        };
                        CondInstr::Mul(Mul {
                            flags: set_flags,
                            ty,
                        })
                    }
                    // Swap/swap byte.
                    else if bits & 0b11011 == 0b10000 {
                        CondInstr::Swp(Swp {
                            byte: instr & b!(22) != 0,
                        })
                    } else {
                        CondInstr::Undef
                    }
                }
                _ if bits & 0b1001 == 0b1001 => {
                    let b20 = instr & b!(20) != 0;
                    let add_ofs = instr & b!(23) != 0;
                    let imm = instr & b!(22) != 0;
                    let w = instr & b!(21) != 0;
                    let p = instr & b!(24) != 0;
                    if p && w {
                        return CondInstr::Unpred;
                    }
                    let addressing = AdrModeTy::from_w_p(w, p);
                    let bits = (instr >> 5) & 0b11;
                    if bits == 0b01 {
                        CondInstr::TransfMisc(TransfMisc {
                            ty: MiscTransfTy::H { load: b20 },
                            add_ofs,
                            imm,
                            adr_ty: addressing,
                        })
                    } else if bits & 0b10 == 0b10 {
                        let imm = instr & b!(22) != 0;
                        // Load signed halfword/byte.
                        if b20 {
                            let halfword = instr & b!(5) != 0;
                            CondInstr::TransfMisc(TransfMisc {
                                ty: if halfword {
                                    MiscTransfTy::SH
                                } else {
                                    MiscTransfTy::SB
                                },
                                add_ofs,
                                imm,
                                adr_ty: addressing,
                            })
                        }
                        // Load/store two words.
                        else {
                            let store = instr & b!(5) != 0;
                            CondInstr::TransfDouble(TransfDouble {
                                store,
                                imm,
                                add_ofs,
                                adr_ty: addressing,
                            })
                        }
                    } else {
                        CondInstr::Undef
                    }
                }
                _ => CondInstr::Undef,
            }
        }
        // Data processing shift or immediate
        else if instr & 0x0E00_0010 == 0x0000_0000
            || instr & 0x0E00_0090 == 0x0000_0010
            || instr & 0x0E00_0000 == 0x0200_0000
        {
            let opcode = {
                use DpOpcTy::*;
                match (instr >> 21) & 0xF {
                    0b0000 => And,
                    0b0001 => Eor,
                    0b0010 => Sub,
                    0b0011 => Rsb,
                    0b0100 => Add,
                    0b0101 => Adc,
                    0b0110 => Sbc,
                    0b0111 => Rsc,
                    0b1000 => Tst,
                    0b1001 => Teq,
                    0b1010 => Cmp,
                    0b1011 => Cmn,
                    0b1100 => Orr,
                    0b1101 => Mov,
                    0b1110 => Bic,
                    0b1111 => Mvn,
                    _ => panic!("Unreachable"),
                }
            };
            let set_flags = instr & b!(20) != 0;
            let operand = if instr & b!(25) == 0 {
                let reg = instr & b!(4) != 0;
                DpOperTy::Shft {
                    is_reg: reg,
                    ty: ShiftTy::from_bits((instr >> 5) & 0b11),
                }
            } else {
                DpOperTy::Imm
            };
            CondInstr::Dp(Dp {
                flags: set_flags,
                opc: opcode,
                oper: operand,
            })
        }
        // Move immediate to status register
        else if instr & 0x0FB0_0000 == 0x0320_0000 {
            CondInstr::Msr(Msr {
                r: instr & b!(22) != 0,
                imm: true,
            })
        }
        // Load/store immediate/register offset
        else if instr & 0x0E00_0000 == 0x0400_0000 || instr & 0x0E00_0010 == 0x0600_0000 {
            let w = instr & b!(21) != 0;
            let p = instr & b!(24) != 0;
            let adr_ty = AdrModeTy::from_w_p(w, p);
            CondInstr::Transf(Transf {
                load: instr & b!(20) != 0,
                ty: if instr & b!(22) != 0 {
                    TransfTy::Byte
                } else {
                    TransfTy::Word
                },
                add_ofs: instr & b!(23) != 0,
                oper: if instr & b!(25) != 0 {
                    // register
                    TransfOperTy::Reg {
                        shift: ShiftTy::from_bits((instr >> 5) & 0b11),
                    }
                } else {
                    // immediate
                    TransfOperTy::Imm
                },
                adr_ty,
            })
        }
        // Load/store multiple
        else if instr & 0x0E00_0000 == 0x0800_0000 {
            let p = (instr >> 24) != 0;
            let u = (instr >> 23) != 0;
            let adr_ty = match (p, u) {
                (false, true) => AdrMode4::IB,
                (false, false) => AdrMode4::DB,
                (true, true) => AdrMode4::IA,
                (true, false) => AdrMode4::DA,
            };
            CondInstr::TransfMult(TransfMult {
                load: instr & b!(20) != 0,
                base_update: instr & b!(21) != 0,
                privilige_mode: instr & b!(22) != 0,
                adr_ty,
            })
        }
        // Branch and branch with link
        else if instr & 0x0E00_0000 == 0x0A00_0000 {
            CondInstr::B(B {
                link: instr & b!(24) != 0,
            })
        }
        // Coprocessor load/store and double register transfers
        else if instr & 0x0E00_0000 == 0x0C00_0000 {
            let p = instr & b!(24) != 0;
            let u = instr & b!(23) != 0;
            let w = instr & b!(21) != 0;
            let adr_ty = {
                match (p, w) {
                    (false, false) => {
                        if u {
                            AdrMode5::Unindexed
                        } else {
                            // Might be undefined depending on version.
                            return CondInstr::Unpred;
                        }
                    }
                    (false, true) => AdrMode5::Post,
                    (true, false) => AdrMode5::Offset,
                    (true, true) => AdrMode5::Pre,
                }
            };
            CondInstr::CpTransf(CpTransf {
                load: instr & b!(20) != 0,
                adr_ty,
                n: instr & b!(22) != 0,
                add_ofs: u,
            })
        }
        // Coprocessor data processing
        else if instr & 0x0F00_0010 == 0x0E00_0000 {
            CondInstr::CpDp
        }
        // Coprocessor register transfers
        else if instr & 0x0F00_0010 == 0x0E00_0010 {
            CondInstr::CpMov(CpMov {
                arm_reg_load: instr & b!(20) != 0,
            })
        }
        // Software interrupt
        else if instr & 0x0F00_0000 == 0x0F00_0000 {
            CondInstr::Swi
        }
        // Undefined
        else {
            CondInstr::Undef
        }
    }

    pub const fn decode_uncond(&self, instr: u32) -> UnCondInstr {
        if instr & 0x0E00_0000 == 0x0A00_0000 {
            UnCondInstr::BlxImm
        } else {
            UnCondInstr::Undef
        }
    }
}

pub fn is_cond_instr(instr: u32) -> bool {
    instr >> 28 == 0b1111
}

pub const fn decode_cond(proc: Processor, instr: u32) -> CondInstr {
    proc.decode_cond(instr)
}

pub const fn decode_uncond(proc: &Processor, instr: u32) -> UnCondInstr {
    proc.decode_uncond(instr)
}
