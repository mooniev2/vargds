use crate::{bus::Access, Core, Engine};

pub fn read8<E: Engine, A: Access>(core: &mut Core<E>, adr: u32) -> u8 {
    if A::CPU {
        panic!("fallback {adr:08X}");
    }
    u8::MAX
}

pub fn read16<E: Engine, A: Access>(core: &mut Core<E>, adr: u32) -> u16 {
    if A::CPU {
        panic!("fallback {adr:08X}");
    }
    u16::MAX
}

pub fn read32<E: Engine, A: Access>(core: &mut Core<E>, adr: u32) -> u32 {
    if A::CPU {
        panic!("fallback {adr:08X}");
    }
    u32::MAX
}

pub fn write8<E: Engine, A: Access>(core: &mut Core<E>, adr: u32, val: u8) {
    if A::CPU {
        panic!("fallback {adr:08X}");
    }
}

pub fn write16<E: Engine, A: Access>(core: &mut Core<E>, adr: u32, val: u16) {
    if A::CPU {
        panic!("fallback {adr:08X}");
    }
}

pub fn write32<E: Engine, A: Access>(core: &mut Core<E>, adr: u32, val: u32) {
    if A::CPU {
        panic!("fallback {adr:08X}");
    }
}
