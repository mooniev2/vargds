use crate::bus::{self, masks, PtrTable};
use crate::cpu::arm9;
use crate::mmap::MAIN_MEMORY_START;
use crate::unsafemem::UnsafeMem;
use crate::{Arm9, Cartridge, Core, Engine, Result};

impl<E: Engine> Core<E> {
    pub fn new(#[cfg(feature = "log")] logger: slog::Logger) -> Self {
        let arm9 = Arm9::<E>::new(
            #[cfg(feature = "log")]
            logger.new(slog::o!("arm9" => "arm9")),
        );
        let mut core = Self {
            global_data: Default::default(),
            arm9,
            main_memory: UnsafeMem::from_box(
                vec![0; mb!(4)]
                    .into_boxed_slice()
                    .try_into()
                    .expect("failed to initialize main memory"),
            ),
            #[cfg(feature = "log")]
            logger,
        };
        core.init();
        core
    }

    fn init(&mut self) {
        self.arm9.init();

        let main_memory_slice = unsafe { &mut *self.main_memory.get() };
        let main_memory_ptr = unsafe { (*main_memory_slice).as_mut_ptr() };

        // map pointer tables in the arm9 core.

        // map main memory (mirrored 16MiB).
        let mut adr = MAIN_MEMORY_START;
        for _ in 0..4 {
            let mut ptr = main_memory_ptr;
            let pg_size = PtrTable::PG_SIZE;
            for _ in 0..(mb!(4) / pg_size) {
                self.arm9.bus_ptrs.map(
                    PtrTable::adr_to_page(adr),
                    masks::R | masks::W_16_32 | masks::W_8,
                    ptr,
                );
                debug!(self.logger, "{adr:08x} => {ptr:?}");
                adr += pg_size as u32;
                ptr = unsafe { ptr.add(pg_size) }
            }
        }
    }

    pub fn load_rom(&mut self, rom: Box<[u8]>) -> Result<()> {
        let cartridge = Cartridge::new(&rom)?;
        let header = cartridge.header();

        debug!(
            self.logger,
            "\
            loading rom\n\
            header: {}\
            ",
            header.display_header(),
        );

        // validate the cartridge has expected values.
        cartridge.validate()?;

        self.load_rom_internal(&cartridge);

        Ok(())
    }

    /// # Safety
    /// may cause out of bounds access if the ROM is malformed.
    pub unsafe fn load_unvalidated_rom(&mut self, rom: Box<[u8]>) -> Result<()> {
        let cartridge = Cartridge::new(&rom)?;
        let header = cartridge.header();

        debug!(
            self.logger,
            "\
            loading rom (unvalidated)\n\
            header: {}\
            ",
            header.display_header(),
        );

        self.load_rom_internal(&cartridge);

        Ok(())
    }

    fn load_rom_internal(&mut self, cartridge: &Cartridge) {
        let header = cartridge.header();
        let main_memory = unsafe { &mut *self.main_memory.get() };

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

        // map cartridge header.
        main_memory[0x3ffe00..0x400000].copy_from_slice(header.as_ref());

        // map the arm7 rom.
        let arm7_offset_beg = arm7_ram;
        let arm7_offset_end = arm7_offset_beg + arm7_size;
        let _arm7_rom = cartridge.arm7_rom();
        for (_i, _adr) in (arm7_offset_beg..arm7_offset_end).enumerate() {}

        // map the arm9 rom.
        let arm9_offset_beg = arm9_ram;
        let arm9_offset_end = arm9_offset_beg + arm9_size;
        let arm9_rom = cartridge.arm9_rom();
        for (i, adr) in (arm9_offset_beg..arm9_offset_end).enumerate() {
            bus::arm9::write8(self, adr, arm9_rom[i]);
        }

        // set the correct pc values.
        self.arm9.pc_set(arm9_entry);
    }
}
