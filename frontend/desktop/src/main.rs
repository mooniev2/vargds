#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate slog;

extern crate vargds_core as nds;

mod cargs;
mod gui;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("crate::*", log::LevelFilter::max())
        .filter_module("nds::*", log::LevelFilter::max())
        .init();

    gui::main();
}
