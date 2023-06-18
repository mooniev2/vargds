use std::path::Display;

use crate::error::{Error, Result};
use crate::mmap;

const CART_HEADER_LEN: usize = 0x200;

pub struct Cartridge<'a>(&'a [u8]);

impl<'a> Cartridge<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self> {
        if data.len() < CART_HEADER_LEN {
            return Err(Error::Cartridge(format!(
                "the cartridge is too small, has to be at least '{}' but got '{}'",
                CART_HEADER_LEN,
                data.len(),
            )));
        }
        Ok(Self(data))
    }

    pub fn validate(&self) -> Result<()> {
        let header = self.header();

        // arm9 loading.
        let arm9_rom = header.arm9_rom_offset();
        let arm9_size = header.arm9_size();
        let arm9_entry = header.arm9_entry_address();
        let arm9_ram = header.arm9_ram_address();

        // arm7 loading.
        let arm7_rom = header.arm7_rom_offset();
        let arm7_size = header.arm7_size();
        let arm7_entry = header.arm7_entry_address();
        let arm7_ram = header.arm7_ram_address();

        // sanity check the loading rom.

        // assure entries in the header have correct values.
        macro_rules! check_range {
            ($var:expr, $($range:expr),+; $($arg:tt)+) => {
                if ! ($(($range).contains(&($var)))||+) {
                    let mut expected = String::new();
                    $(
                        expected.push_str(&format!("'{:x?}' ", $range));
                    )+
                    let msg = format!(
                        "{}\nexpected: {}got: '{:x}'\n",
                        format!($($arg)+),
                        expected,
                        $var
                    );
                    return Err(Error::Cartridge(msg));
                }
            };
        }

        check_range!(arm9_entry, 0x2000000..0x23bfe00; "ARM9 entry address: outside of range");
        check_range!(
            arm9_ram,
            0x2000000..0x23bfe00;
            "ARM9 RAM address: outside of range"
        );
        check_range!(arm9_size, 0..=0x3bfe00; "ARM9 ROM size too large.");

        check_range!(
            arm7_entry,
            0x2000000..0x23bfe00, 0x37f8000..0x3807e00;
            "ARM7 entry address: outside of range"
        );
        let arm7_second_area_start = 0x37f8000;
        check_range!(
            arm7_ram,
            0x2000000..0x23bfe00, 0x37f8000..0x3807e00;
            "ARM7 RAM address: outside of range"
        );
        if arm7_ram >= arm7_second_area_start {
            check_range!(arm7_size, 0..=0xFE00; "ARM7 ROM size too large.");
        } else {
            check_range!(arm7_size, 0..=0x3bfe00; "ARM7 ROM size too large.");
        }

        // make sure rom offsets are aligned and a above 0x4000.

        let minimum_rom_offs = 0x4000;
        let rom_offs_mask = !(0x1000 - 1);

        if arm7_rom < minimum_rom_offs {
            return Err(Error::Cartridge(format!(
                "ARM7 ROM had incorrect offset. offset: '{arm7_rom}'"
            )));
        }

        if arm7_rom & rom_offs_mask != 0 {
            return Err(Error::Cartridge(format!(
                "unaligned ARM7 ROM offset {arm7_rom}"
            )));
        }

        if arm9_rom < minimum_rom_offs {
            return Err(Error::Cartridge(format!(
                "ARM9 ROM had incorrect offset. offset: '{arm9_rom}'"
            )));
        }

        if arm7_rom & rom_offs_mask != 0 {
            return Err(Error::Cartridge(format!(
                "unaligned ARM9 ROM offset {arm7_rom}"
            )));
        }

        // make sure that the loaded rom doesn't go outside the bounds of the main memory.

        let main_memory_range = (mmap::MAIN_MEMORY_START..mmap::MAIN_MEMORY_END);

        let arm9_rom_load_end = arm9_ram + arm9_size;
        if !main_memory_range.contains(&arm9_rom_load_end) {
            return Err(Error::Cartridge(format!(
                "ARM9 ROM attempted to load at unmapped memory. ROM start: '{}', ROM length: '{}'",
                arm9_ram, arm9_size
            )));
        }

        let arm7_rom_load_end = arm7_ram + arm7_size;
        if !main_memory_range.contains(&arm7_rom_load_end) {
            return Err(Error::Cartridge(format!(
                "ARM7 ROM attempted to load at unmapped memory. ROM start: '{}', ROM length: '{}'",
                arm7_ram, arm7_size
            )));
        }

        Ok(())
    }

    pub fn header(&self) -> CartridgeHeader<'a> {
        debug_assert!(self.0.len() >= CART_HEADER_LEN);
        CartridgeHeader(&self.0[0..CART_HEADER_LEN])
    }

    pub fn arm9_rom(&self) -> &[u8] {
        let header = self.header();
        let start = header.arm9_rom_offset() as usize;
        let size = header.arm9_size() as usize;
        &self.0[start..(start + size)]
    }

    pub fn arm7_rom(&self) -> &[u8] {
        let header = self.header();
        let start = header.arm7_rom_offset() as usize;
        let size = header.arm7_size() as usize;
        &self.0[start..(start + size)]
    }
}

pub struct CartridgeHeader<'a>(&'a [u8]);

macro_rules! impl_header_fields {
    (
        $(
            $field:ident, $ty:ty, $offs:expr;
        )*
    ) => {
        $(
            pub fn $field(&self) -> $ty {
                unsafe { self.from_offs($offs) }
            }
        )*

        pub fn display_header(&self) -> String{
            let mut string = String::new();
            $(
                string.push_str(&format!("{}: {:x}\n", stringify!($field), self.$field()));
            )*
            string
        }
    };
}

impl<'a> CartridgeHeader<'a> {
    pub const LEN: usize = CART_HEADER_LEN;

    pub fn as_ref(&self) -> &[u8] {
        self.0
    }

    pub fn game_title(&self) -> &str {
        todo!()
    }

    unsafe fn from_offs<T>(&self, offs: usize) -> T {
        (self.0.as_ptr().add(offs) as *const T).read()
    }

    impl_header_fields!(
        arm9_rom_offset, u32, 0x020;
        arm9_entry_address, u32, 0x024;
        arm9_ram_address, u32, 0x028;
        arm9_size, u32, 0x02c;
        arm7_rom_offset, u32, 0x30;
        arm7_entry_address, u32, 0x034;
        arm7_ram_address, u32, 0x038;
        arm7_size, u32, 0x03c;
    );
}
