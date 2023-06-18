pub mod arm9;

use crate::{Core, Engine};

pub struct Interpreter;

impl Engine for Interpreter {
    type GlobalData = ();
    type ARM9Data = ();
    type ARM7Data = ();
}

pub fn run(core: &mut Core<Interpreter>) {
    for _ in 0..100_000 {
        arm9::step(core)
    }
}
