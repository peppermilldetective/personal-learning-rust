extern crate winit;

mod engine;

fn main() {
   let winit_state = engine::create_default_window();
   engine::run(winit_state);
}