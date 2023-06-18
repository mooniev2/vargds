mod gfx;

use nds::Interpreter;

use slog::Level;
use slog::Logger;

use std::fs;
use std::mem::ManuallyDrop;

use egui::TexturesDelta;
use egui_winit::egui;
use egui_winit::winit;
use egui_winit::winit::event::Event;
use egui_winit::winit::event::WindowEvent;

use crate::cargs;

fn run<S: 'static>(
    state: S,
    windows_logger: Logger,
    mut on_kbd: impl FnMut(&mut S) + 'static,
    mut on_frame: impl FnMut(&mut S, &egui::Context) + 'static,
    mut on_exit: impl FnMut(S) + 'static,
) -> ! {
    let event_loop = winit::event_loop::EventLoop::new();
    let mut window = gfx::Window::new(&event_loop, windows_logger).expect("failed to open window");
    let mut egui_state = egui_winit::State::new(&event_loop);
    let egui_ctx = egui::Context::default();
    let mut state = ManuallyDrop::new(state);
    event_loop.run(move |event, _, ctrl_flow| {
        ctrl_flow.set_poll();
        match event {
            Event::WindowEvent { event, .. } => {
                // configure this to not always consume events?
                let _ = egui_state.on_event(&egui_ctx, &event);
                match event {
                    WindowEvent::Resized(_) => window.resized(),
                    WindowEvent::KeyboardInput { .. } => on_kbd(&mut state),
                    WindowEvent::CloseRequested => ctrl_flow.set_exit(),
                    _ => {}
                }
            }
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(_) => {
                let output =
                    egui_ctx.run(egui_state.take_egui_input(window.winit_window()), |ctx| {
                        on_frame(&mut state, ctx);
                    });
                egui_state.handle_platform_output(
                    window.winit_window(),
                    &egui_ctx,
                    output.platform_output,
                );
                let clipped_primitives = egui_ctx.tessellate(output.shapes);
                let TexturesDelta {
                    set: texture_set,
                    free: texture_free,
                } = output.textures_delta;
                window.render(texture_set, texture_free, &clipped_primitives);
            }
            Event::LoopDestroyed => {
                let dropped = unsafe { ManuallyDrop::take(&mut state) };
                on_exit(dropped)
            }
            _ => {}
        }
    })
}

pub struct Drain;

impl slog::Drain for Drain {
    type Ok = ();

    type Err = slog::Never;

    fn log(
        &self,
        record: &slog::Record,
        _values: &slog::OwnedKVList,
    ) -> std::result::Result<Self::Ok, Self::Err> {
        let file = record.file();
        let line = record.line();
        let module = record.module();
        let func = record.function();
        let _tag = record.tag();
        let msg = record.msg();
        match record.level() {
            Level::Critical => {
                println!("[CRITICAL] {file}:{line} {module} {func} '{msg}'",)
            }
            Level::Error => println!("[ERROR] {file}:{line} '{msg}'"),
            Level::Warning => println!("[WARN] {file}:{line} '{msg}'"),
            Level::Info => println!("{msg}"),
            Level::Debug => println!("[DEBUG] {msg}"),
            Level::Trace => println!("[TRACE] {msg}"),
        }
        Ok(())
    }
}

pub fn main() {
    let cargs = cargs::from_env();
    let logger = Logger::root(Drain, o!("vargds" => "vds"));
    let mut core =
        nds::Core::<nds::Interpreter>::new(Logger::root(Drain, slog::o!("core" => "core")));
    if let Err(err) = unsafe {
        core.load_unvalidated_rom(
            fs::read(&cargs.rom.expect("didn't supply rom"))
                .expect("failed to read rom")
                .into_boxed_slice(),
        )
    } {
        panic!("failed to load rom\n{err}");
    }
    struct State {
        core: nds::Core<Interpreter>,
        logger: Logger,
    }
    let window_logger = logger.new(o!("window" => "window"));
    run(
        State { core, logger },
        window_logger,
        |state| {
            info!(state.logger, "kbd input");
        },
        |state, ctx| {
            egui::Window::new("test").show(ctx, |ui| {
                ui.label("hello");
            });
            nds::interpreter::run(&mut state.core);
        },
        |state| info!(state.logger, "exiting"),
    );
}
