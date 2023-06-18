mod branch;
mod data;
mod mem;
mod misc;

use crate::bus::arm9 as bus;
use crate::cpu::arm9;
use crate::{Core, Interpreter};

static COND_INSTR_LUT: [fn(&mut Core<Interpreter>, u32); 4096] = {
    use arm_decode::*;

    include!("../../gen/arm9_cond_lut.inc")
};

static UNCOND_INSTR_LUT: [fn(&mut Core<Interpreter>, u32); 4096] =
    include!("../../gen/arm9_uncond_lut.inc");

impl Core<Interpreter> {
    fn fetch(&mut self) -> u32 {
        let fetch = bus::read32(self, self.arm9.pc());
        self.arm9.pc_set(self.arm9.pc().wrapping_add(4));
        fetch
    }

    fn check_cond(&self, cond_bits: u32) -> bool {
        true
    }
}

pub fn step(core: &mut Core<Interpreter>) {
    let fetch = core.fetch();
    let is_cond = arm_decode::ARM9.is_cond_instr(fetch);
    let index = arm_decode::ARM9.extract_instr_bits(fetch) as usize;
    if is_cond {
        let cond_bits = arm_decode::ARM9.cond_bits(fetch);
        if core.check_cond(cond_bits) {
            COND_INSTR_LUT[index](core, fetch);
        }
    } else {
        UNCOND_INSTR_LUT[index](core, fetch)
    }
}
