mod arm9;

use crate::{Core, Engine};

pub struct Interpreter;

impl Engine for Interpreter {
    type GlobalData = ();
    type ARM9Data = ();
    type ARM7Data = ();
}

pub fn step(core: &mut Core<Interpreter>) {}
