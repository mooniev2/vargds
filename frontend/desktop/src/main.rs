use std::fs;

use nds::Interpreter;

#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate log;

mod cargs;
mod gui;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Warn)
        .init();
    let cargs = cargs::from_env();
    let core = nds::Core::<nds::Interpreter>::new(
        fs::read(&cargs.rom.expect("didn't supply rom"))
            .expect("failed to read rom")
            .into_boxed_slice(),
    );
    struct State {
        core: nds::Core<Interpreter>,
    }
    gui::run(
        State { core },
        |state| {
            info!("kbd input");
        },
        |state, ctx| {
            egui::Window::new("test").show(ctx, |ui| {
                ui.label("hello");
            });
            nds::interpreter::step(&mut state.core);
        },
        |state| info!("exiting"),
    );
}
