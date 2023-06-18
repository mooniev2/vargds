use std::ptr::null_mut;

pub type Attr = u8;
pub mod masks {
    use super::Attr;

    pub const R: Attr = b!(0);
    pub const W_8: Attr = b!(1);
    pub const W_16_32: Attr = b!(2);
}

pub struct PtrTable {
    attrs: [Attr; Self::ENTRIES],
    ptrs: [*mut u8; Self::ENTRIES],
}

impl Default for PtrTable {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
pub fn adr_to_page(adr: u32) -> usize {
    PtrTable::adr_to_page(adr)
}

impl PtrTable {
    pub const PG_SHIFT: usize = 14;
    pub const PG_SIZE: usize = 1 << Self::PG_SHIFT;
    pub const PG_MASK: u32 = Self::PG_SIZE as u32 - 1;
    pub const ENTRIES: usize = 1 << (32 - Self::PG_SHIFT);

    #[inline]
    pub fn new() -> Self {
        Self {
            attrs: [0; Self::ENTRIES],
            ptrs: [core::ptr::null_mut(); Self::ENTRIES],
        }
    }

    #[inline]
    pub fn adr_to_page(adr: u32) -> usize {
        adr as usize >> Self::PG_SHIFT
    }

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
    pub fn map(&mut self, page: usize, attrs: Attr, ptr: *mut u8) {
        self.attrs[page] = attrs;
        self.ptrs[page] = ptr;
    }

    pub fn unmap(&mut self, page: usize) {
        self.ptrs[page] = null_mut();
        self.attrs[page] = 0;
    }
}
