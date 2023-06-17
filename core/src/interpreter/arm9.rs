mod branch;
mod data;
mod mem;
mod misc;

use crate::{Core, Interpreter};

static COND_INSTR_LUT: [fn(&mut Core<Interpreter>, u32); 4096] = {
    use arm_decode::*;

    include!("../../gen/arm9_cond_lut.txt")
};

static UNCOND_INSTR_LUT: [fn(&mut Core<Interpreter>, u32); 4096] =
    include!("../../gen/arm9_uncond_lut.txt");

pub fn step() {}
