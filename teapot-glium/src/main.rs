#[macro_use]
extern crate glium;

mod teapot;

fn main() {
   use glium::{glutin, Surface};

   let mut events_loop = glutin::EventsLoop::new();
   let window = glutin::WindowBuilder::new();
   let context = glutin::ContextBuilder::new().with_depth_buffer(24);
   let display = glium::Display::new(window, context, &events_loop).unwrap();

   let positions = glium::VertexBuffer::new(&display, &teapot::VERTICES).unwrap();
   let normals = glium::VertexBuffer::new(&display, &teapot::NORMALS).unwrap();
   
   let indices = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList,
      &teapot::INDICES).unwrap();

   let vertex_shader_src = r#"
      #version 150

      in vec3 position;
      in vec3 normal;

      out vec3 v_normal;

      uniform mat4 perspective;
      uniform mat4 matrix;

      void main() {
         v_normal = transpose(inverse(mat3(matrix))) * normal;
         gl_Position = perspective * matrix * vec4(position, 1.0);
      }
   "#;

   let fragment_shader_src = r#"
      #version 140

      in vec3 v_normal;
      out vec4 color;

      uniform vec3 u_light;

      void main() {
         float brightness = dot(normalize(v_normal), normalize(u_light));
         vec3 dark_color = vec3(0.6, 0.0, 0.0);
         vec3 regular_color = vec3(1.0, 0.0, 0.0);
         color = vec4(mix(dark_color, regular_color, brightness), 1.0);
      }
   "#;

   let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None)
      .unwrap();

   let mut closed = false;
   while !closed {
      let mut target = display.draw();

      let matrix = [
            [0.01, 0.0, 0.0, 0.0],
            [0.0, 0.01, 0.0, 0.0],
            [0.0, 0.0, 0.01, 0.0],
            [0.0, 0.0, 2.0, 1.0f32],
      ];

      let light = [-1.0, 0.4, 0.9f32];

      let perspective = {
         let (width, height) = target.get_dimensions();
         let aspect_ratio = height as f32 / width as f32;

         let fov: f32 = 3.141592 / 3.0;
         let zfar = 1024.0;
         let znear = 0.1;

         let f = 1.0 / (fov / 2.0).tan();

         [
            [f * aspect_ratio   ,    0.0,            0.0                ,   0.0],
            [       0.0         ,     f ,            0.0                ,   0.0],
            [       0.0         ,    0.0,   (zfar+znear)/(zfar-znear)   ,   1.0],
            [       0.0         ,    0.0, -(2.0*zfar*znear)/(zfar-znear),   0.0],
         ]
      };

      target.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);

      let params = glium::DrawParameters {
         depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            .. Default::default()
         },
         .. Default::default()
      };

      target.draw((&positions, &normals), &indices, &program, 
         &uniform! { matrix: matrix, perspective: perspective, u_light: light }, 
         &params).unwrap();

      target.finish().unwrap();

      events_loop.poll_events(|ev| {
         match ev {
            glutin::Event::WindowEvent { event, .. } => match event {
               glutin::WindowEvent::CloseRequested => closed = true,
               _ => (),
            },
            _ => (),
         }
      });
   }
}
