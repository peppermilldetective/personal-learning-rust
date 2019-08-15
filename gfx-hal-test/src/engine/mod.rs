#![cfg_attr(
   not(any(
      feature = "vulkan",
      feature = "dx12",
      feature = "gl"
   )),
   allow(dead_code, unused_extern_crates, unused_imports)
)]

extern crate winit;

#[cfg(feature = "dx12")]
extern crate gfx_backend_dx12 as back;
#[cfg(feature = "gl")]
extern crate gfx_backend_gl as back;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as back;

extern crate gfx_hal as hal;

mod winitstate;

use winit::{
   Event,
   WindowEvent,
   DeviceEvent,
   dpi::LogicalSize
};

use winitstate::WinitState;

pub fn create_window(title: &str, width: f64, height: f64) -> WinitState {
   WinitState::new(title, LogicalSize { width, height }).unwrap()
}

pub fn create_default_window() -> WinitState {
   WinitState::default()
}

pub fn run(winit_state: WinitState) {
   
   // Pull what's necessary from the input.

   let mut events_loop = winit_state.events_loop;

   // Run the main loop for the program.
   let mut running = true;
   while running {
      
      // Poll for events.
      events_loop.poll_events(|event| {
         match event {

            // IS WINDOW EVENT
            Event::WindowEvent { event, .. } => {
               match event {
                  WindowEvent::Resized(size) => {
                     println!("Resized to {}x{}", size.width, size.height);
                  },

                  WindowEvent::CloseRequested => {
                     running = false;
                  },

                  _ => ()
               }
            },

            // IS DEVICE EVENT
            Event::DeviceEvent { event, .. } => {
               use winit::{
                  ElementState,
                  VirtualKeyCode
               };

               match event {

                  // KEY PRESSED
                  DeviceEvent::Key(key) => {

                     let keycode = key.virtual_keycode.unwrap();

                     if key.state == ElementState::Pressed && keycode == VirtualKeyCode::Escape {
                        running = false;
                     }
                  },

                  // BUTTON PRESSED
                  DeviceEvent::Button{ button, state } => {
                     if state == ElementState::Pressed {
                        // NOTE for button:
                        //    1 = Left Mouse Button
                        //    2 = Middle Mouse Button
                        //    3 = Right Mouse Button

                        println!("Pressed: {}", button);
                     }
                  },

                  _ => ()
               }
            },
            _ => ()
         }
      });
   }
}