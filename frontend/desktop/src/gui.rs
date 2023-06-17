mod gfx;

use std::mem::ManuallyDrop;

use egui::TexturesDelta;
use egui_winit::egui;
use egui_winit::winit;
use egui_winit::winit::event::Event;
use egui_winit::winit::event::WindowEvent;

pub fn run<S: 'static>(
    state: S,
    mut on_kbd: impl FnMut(&mut S) + 'static,
    mut on_frame: impl FnMut(&mut S, &egui::Context) + 'static,
    mut on_exit: impl FnMut(S) + 'static,
) -> ! {
    let event_loop = winit::event_loop::EventLoop::new();
    let mut window = gfx::Window::new(&event_loop).expect("failed to open window");
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
