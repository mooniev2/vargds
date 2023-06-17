use crate::{Core, Interpreter};

pub fn undef(core: &mut Core<Interpreter>, _: u32) {
    panic!("undefined instruction")
}

pub fn unpred(core: &mut Core<Interpreter>, _: u32) {
    panic!("unpredictable instruction")
}

pub fn bkpt(core: &mut Core<Interpreter>, _: u32) {
    unimplemented!()
}

pub fn swi(core: &mut Core<Interpreter>, _: u32) {
    unimplemented!()
}
