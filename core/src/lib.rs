//TMP
#![allow(dead_code)]
#![allow(unused)]
//
#![feature(stmt_expr_attributes)]
#![allow(incomplete_features)]
#![feature(adt_const_params)]
#![feature(int_roundings)]

#[macro_use]
extern crate log;

pub mod cpu;
pub mod debug;
pub mod interpreter;

mod bus;

pub use interpreter::Interpreter;

use cpu::arm9::ARM9;

pub type NDSInterp = Core<Interpreter>;

pub trait Engine {
    type GlobalData: Default;
    type ARM9Data: Default;
    type ARM7Data: Default;
}

pub struct Core<E: Engine> {
    global_data: E::GlobalData,
    pub arm9: ARM9<E>,
}

impl<E: Engine> Core<E> {
    pub fn new(rom: Box<[u8]>) -> Self {
        let mut arm9 = ARM9::<E>::new();
        arm9.init();
        Self {
            global_data: Default::default(),
            arm9,
        }
    }
}

unsafe impl<E: Engine> Send for Core<E> {}
