pub mod ptrs;

mod fallback;

use crate::{
    bus::{Access, CPUAccess, DebugAccess},
    Core, Engine,
};
use ptrs::Ptrs;

macro_rules! def_read {
    ($($fn_ident:ident, $ty:ty, $fallback:path;)*) => {
        $(
            #[inline(always)]
            fn $fn_ident<A: Access, E: Engine>(core: &mut Core<E>, adr: u32) -> $ty {
                if let Some(ptr) = core.arm9.bus_ptrs.read(adr) {
                    unsafe {
                        let mask = core::mem::size_of::<$ty>() - 1;
                        let mask = Ptrs::PG_MASK as usize & !mask;
                        ptr.add(adr as usize & mask).cast::<$ty>().read().to_le()
                    }
                } else {
                    $fallback(core, adr)
                }
            }
        )*

    };
}

macro_rules! def_write {
    ($($fn_ident:ident, $ty:ty, $write_fn:ident, $fallback:path;)*) => {
        $(
            #[inline(always)]
            fn $fn_ident<A: Access, E: Engine>(core: &mut Core<E>, adr: u32, val: $ty) {
                if let Some(ptr) = core.arm9.bus_ptrs.$write_fn(adr) {
                    unsafe {
                        let mask = core::mem::size_of::<$ty>() - 1;
                        let mask = Ptrs::PG_MASK as usize & !mask;
                        let val = val.to_le();
                        ptr.add(adr as usize & mask).cast::<$ty>().write(val)
                    };
                } else {
                    $fallback(core, adr, val);
                }
            }
        )*

    };
}

def_read! {
    __read8, u8, fallback::read8::<E, A>;
    __read16, u16, fallback::read16::<E, A>;
    __read32, u32, fallback::read32::<E, A>;
}

def_write! {
    __write8, u8, write8, fallback::write8::<E, A>;
    __write16, u16, write32_16, fallback::write16::<E, A>;
    __write32, u32, write32_16, fallback::write32::<E, A>;
}

pub fn read32<E: Engine>(core: &mut Core<E>, adr: u32) -> u32 {
    __read32::<CPUAccess, E>(core, adr)
}

pub fn read32_dbg<E: Engine>(core: &mut Core<E>, adr: u32) -> u32 {
    __read32::<DebugAccess, E>(core, adr)
}

pub fn read16<E: Engine>(core: &mut Core<E>, adr: u32) -> u16 {
    __read16::<CPUAccess, E>(core, adr)
}

pub fn read16_dbg<E: Engine>(core: &mut Core<E>, adr: u32) -> u16 {
    __read16::<DebugAccess, E>(core, adr)
}

pub fn read8<E: Engine>(core: &mut Core<E>, adr: u32) -> u8 {
    __read8::<CPUAccess, E>(core, adr)
}

pub fn read8_dbg<E: Engine>(core: &mut Core<E>, adr: u32) -> u8 {
    __read8::<DebugAccess, E>(core, adr)
}

pub fn write32<E: Engine>(core: &mut Core<E>, adr: u32, val: u32) {
    __write32::<CPUAccess, E>(core, adr, val)
}

pub fn write32_dbg<E: Engine>(core: &mut Core<E>, adr: u32, val: u32) {
    __write32::<DebugAccess, E>(core, adr, val)
}

pub fn write16<E: Engine>(core: &mut Core<E>, adr: u32, val: u16) {
    __write16::<CPUAccess, E>(core, adr, val)
}

pub fn write16_dbg<E: Engine>(core: &mut Core<E>, adr: u32, val: u16) {
    __write16::<DebugAccess, E>(core, adr, val)
}

pub fn write8<E: Engine>(core: &mut Core<E>, adr: u32, val: u8) {
    __write8::<CPUAccess, E>(core, adr, val)
}

pub fn write8_dbg<E: Engine>(core: &mut Core<E>, adr: u32, val: u8) {
    __write8::<DebugAccess, E>(core, adr, val)
}
