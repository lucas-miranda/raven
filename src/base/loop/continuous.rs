use crate::{
    base::{ApplicationState, ApplicationError},
    ecs::{Domain, FrameState},
    input::Event,
    rendering::GraphicAdapter,
    time::Time,
    window::WindowContext,
};

use super::ApplicationLoop;

const WINDOW_SIZE: [u32; 2] = [320, 180];

pub struct ContinuousLoop {
    window_context: WindowContext,
    domains: Vec<Box<dyn Domain>>,
}

impl ContinuousLoop {
}

impl ApplicationLoop for ContinuousLoop {
    fn new(window_context: WindowContext) -> Self {
         Self {
            window_context,
            domains: Vec::new(),
        }
    }

    fn register_domain<D: Into<Box<dyn Domain>>>(&mut self, domain: D) {
        self.domains.push(domain.into());
    }

    fn run(mut self) -> Result<(), ApplicationError> {
        let (logical_window_size, physical_window_size)
            = self.window_context.calculate_window_size(
                (WINDOW_SIZE[0], WINDOW_SIZE[1])
            );

        // window
        let window = self.window_context
            .create_window()
            .with_inner_size(logical_window_size)
            .build()
            .unwrap();

        // time
        let mut last_update_instant = Time::now();
        let mut last_render_instant = Time::now();

        // rendering
        let graphic_adapter = GraphicAdapter::with_surface_size(
                &window,
                (physical_window_size.width, physical_window_size.height),
            )
            .unwrap();

        println!(
            "Window started with (Logical: {:?}, Physical: {:?}",
            logical_window_size,
            physical_window_size,
        );

        // compose application state
        let mut state = ApplicationState::new(window, graphic_adapter);

        // setup all domains
        for domain in &mut self.domains {
            domain.setup(&mut state, &mut self.window_context);
        }

        self.window_context.run(move |event, event_handler| {
            match event {
                winit::event::Event::WindowEvent { event: win_event, .. } => {
                    match win_event {
                        winit::event::WindowEvent::CloseRequested => {
                            println!("Window closed by user");
                            event_handler.request_close();
                        },
                        winit::event::WindowEvent::Resized(dims) => {
                            println!("Window resized to {:?}", dims);
                            state.graphic_adapter.borrow_mut().request_resize_surface(dims.width, dims.height);
                        },
                        winit::event::WindowEvent::RedrawRequested => {
                            let delta_time = state.time.delta(&mut last_render_instant);
                            state.diagnostics.draw_calls = 0; // TODO  handle this line properly
                            let mut frame_state = FrameState {
                                delta: delta_time,
                                app: &mut state,
                            };

                            let render_timer_instant = Time::now();

                            for domain in &mut self.domains {
                                domain.render(&mut frame_state);
                            }

                            state.diagnostics.render_timer = Time::now() - render_timer_instant;
                        },
                        _ => {
                            state.input.handle(Event::from(win_event));

                            for domain in &mut self.domains {
                                domain.input(&mut state);
                            }
                        },
                    }
                },
                winit::event::Event::AboutToWait => {
                    // TODO  set max wait time to be able to change framerate
                    let delta_time = state.time.delta(&mut last_update_instant);
                    let mut frame_state = FrameState {
                        delta: delta_time,
                        app: &mut state,
                    };

                    let update_timer_instant = Time::now();

                    for domain in &mut self.domains {
                        domain.update(&mut frame_state);
                    }

                    state.diagnostics.update_timer = Time::now() - update_timer_instant;
                    state.main_window.request_redraw();
                },
                _ => ()
            }
        }).map_err(ApplicationError::from)
    }
}
