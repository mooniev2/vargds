use super::psr::Psr;

use crate::bus::{self, masks, PtrTable};
use crate::mmap::{MAIN_MEMORY_END, MAIN_MEMORY_START};
use crate::{mmap, Core, Engine};

use slog::Logger;

pub struct Arm9<E: Engine> {
    pub gpr: [u32; 16],
    pub cpsr: Psr,
    pub(crate) bus_ptrs: Box<PtrTable>,
    pub(crate) data: E::ARM9Data,
    pub(crate) logger: Logger,
}

impl<E: Engine> Arm9<E> {
    pub fn new(#[cfg(feature = "log")] logger: Logger) -> Self {
        Self {
            bus_ptrs: Box::new(PtrTable::default()),
            gpr: [0; 16],
            data: Default::default(),
            cpsr: Psr::new(),
            #[cfg(feature = "log")]
            logger,
        }
    }

    pub fn init(&mut self) {
        self.gpr = Default::default();
        self.cpsr = Default::default();
    }

    pub fn gpr(&self, index: usize) -> u32 {
        debug_assert!(index < self.gpr.len());
        match index & 0xF {
            i @ 0..=14 => unsafe { *self.gpr.get_unchecked(i & 0xF) },
            15 => unsafe { self.gpr.get_unchecked(15) }.wrapping_add(4),
            _ => unreachable!(),
        }
    }

    pub fn gpr_set(&mut self, index: usize, val: u32) {
        debug_assert!(index < self.gpr.len());
        unsafe { *self.gpr.get_unchecked_mut(index & 0xF) = val };
    }

    pub fn lr_set(&mut self, val: u32) {
        self.gpr_set(14, val)
    }

    pub fn pc(&self) -> u32 {
        self.gpr(15)
    }

    pub fn pc_set(&mut self, val: u32) {
        self.gpr_set(15, val)
    }
}
