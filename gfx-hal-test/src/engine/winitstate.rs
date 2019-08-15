use winit::{
   dpi::LogicalSize,
   CreationError,
   WindowBuilder,
   EventsLoop,
   Window
};

#[derive(Debug)]
pub struct WinitState {
   pub events_loop: EventsLoop,
   pub window: Window,
}

impl WinitState {
   /// Constructs a new `EventsLoop` and `Window` pair.
   /// 
   /// The specified title and size are used, other elements are default.
   /// 
   /// ## Failure
   /// It is possible for the function to fail to create a window.
   pub fn new<T: Into<String>>(title: T, size: LogicalSize) -> Result<Self, CreationError> {
      // build the event loop.
      let mut events_loop = EventsLoop::new();

      // build the window for the program.
      let output = WindowBuilder::new()
         .with_dimensions(LogicalSize::new(800f64, 800f64))
         .with_title("Testing")
         .build(&events_loop);

      output.map(|window| Self {
         events_loop,
         window
      })
   }
}

impl Default for WinitState {
   /// Makes an 800x600 window with the name `Welcome` as the Title.
   /// 
   /// ## Panics
   /// 
   /// If a `CreationError` occurs.
   fn default() -> Self {
      Self::new(
         "Welcome",
         LogicalSize {
            width: 800.0,
            height: 600.0
         }
      ).expect("Unable to create window.")
   }
}