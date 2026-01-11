use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
};

pub struct InputHandler;

impl InputHandler {
    pub fn handle_window_event(
        event: &WindowEvent, 
        event_loop: &ActiveEventLoop
    ) -> bool {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed. Stopping 🛑");
                event_loop.exit();
                true
            }
            WindowEvent::KeyboardInput { event, .. } => {
                use winit::keyboard::{KeyCode, PhysicalKey};
                if event.state.is_pressed() {
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::KeyQ) | 
                        PhysicalKey::Code(KeyCode::Escape) => {
                            println!("Quit key pressed. Stopping 🛑");
                            event_loop.exit();
                            true
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}