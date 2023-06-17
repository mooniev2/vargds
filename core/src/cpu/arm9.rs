pub mod bus;

use self::bus::ptrs::Ptrs;
use super::psr::PSR;
use crate::Engine;

pub struct ARM9<E: Engine> {
    pub gpr: [u32; 16],
    pub cpsr: PSR,
    bus_ptrs: Box<Ptrs>,
    data: E::ARM9Data,
}

impl<E: Engine> ARM9<E> {
    pub fn new() -> Self {
        Self {
            bus_ptrs: Box::new(Ptrs::default()),
            gpr: [0; 16],
            data: Default::default(),
            cpsr: PSR::new(),
        }
    }

    pub fn init(&mut self) {}

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
