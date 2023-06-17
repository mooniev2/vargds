pub type Attr = u8;
pub mod masks {
    use super::Attr;

    pub const R: Attr = 0x01;
    pub const W_8: Attr = 0x02;
    pub const W_16_32: Attr = 0x04;
}

pub struct Ptrs {
    attrs: [Attr; Self::ENTRIES],
    ptrs: [*mut u8; Self::ENTRIES],
}

impl Default for Ptrs {
    #[inline(always)]
    fn default() -> Self {
        Self {
            attrs: [0; Self::ENTRIES],
            ptrs: [core::ptr::null_mut(); Self::ENTRIES],
        }
    }
}

impl Ptrs {
    pub const PG_SHIFT: usize = 14;
    pub const PG_SIZE: usize = 1 << Self::PG_SHIFT;
    pub const PG_MASK: u32 = Self::PG_SIZE as u32 - 1;
    pub const ENTRIES: usize = 1 << (32 - Self::PG_SHIFT);

    pub fn read(&self, adr: u32) -> Option<*const u8> {
        let index = adr as usize >> Self::PG_SHIFT;
        let attrs = self.attrs[index];
        if attrs & masks::R != 0 {
            let ptrs = self.ptrs[index];
            Some(ptrs)
        } else {
            None
        }
    }

    pub fn write8(&self, adr: u32) -> Option<*mut u8> {
        let index = adr as usize >> Self::PG_SHIFT;
        let attrs = self.attrs[index];
        if attrs & masks::W_8 != 0 {
            let ptrs = self.ptrs[index];
            Some(ptrs)
        } else {
            None
        }
    }

    pub fn write32_16(&self, adr: u32) -> Option<*mut u8> {
        let index = adr as usize >> Self::PG_SHIFT;
        let attrs = self.attrs[index];
        if attrs & masks::W_16_32 != 0 {
            let ptrs = self.ptrs[index];
            Some(ptrs)
        } else {
            None
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn map(&mut self, adr: u32, length: usize, attr: u8, ptr: *mut u8) {
        debug_assert!(adr & Self::PG_MASK == 0);
        debug_assert!(length != 0);
        let index = adr as usize >> Self::PG_SHIFT;
        let pg_cnt = length.div_ceil(Self::PG_SIZE);
        for page in 0..pg_cnt {
            let index = index + page;
            self.attrs[index] = attr;
            self.ptrs[index] = unsafe { ptr.add(page << Self::PG_SHIFT) };
        }
    }
}
