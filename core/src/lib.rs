//TMP
#![allow(dead_code)]
#![allow(unused)]
//
#![feature(stmt_expr_attributes)]
#![allow(incomplete_features)]
#![feature(adt_const_params)]
#![feature(int_roundings)]

#[macro_use]
mod macros;

#[cfg(feature = "log")]
extern crate slog;

pub mod debug;
pub mod error;

pub mod interpreter;
pub use interpreter::Interpreter;

pub mod cpu;
pub use cpu::arm9::Arm9;

// components
mod bus;

mod cartridge;
pub use cartridge::{Cartridge, CartridgeHeader};

// utility
mod mmap;
use mmap::{MAIN_MEMORY_END, MAIN_MEMORY_START};

mod unsafemem;
use unsafemem::UnsafeMem;

// core impl module
mod core_impl;

pub type NDSInterp = Core<Interpreter>;

pub use error::{Error, Result};

use slog::Logger;

pub trait Engine {
    type GlobalData: Default;
    type ARM9Data: Default;
    type ARM7Data: Default;
}

pub struct Core<E: Engine> {
    global_data: E::GlobalData,
    pub arm9: Arm9<E>,
    main_memory: UnsafeMem<[u8; mb!(4)]>,
    logger: Logger,
}

unsafe impl<E: Engine> Send for Core<E> {}
