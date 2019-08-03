/* External crate definitions */
extern crate image;
extern crate vulkano;
extern crate vulkano_shaders;

/* All uses needed for this program */
use std::sync::Arc;

use image::{ImageBuffer, Rgba};

use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBuffer, DynamicState};
use vulkano::device::{Device, DeviceExtensions, Features};
use vulkano::framebuffer::{Framebuffer, Subpass};
use vulkano::format::Format;
use vulkano::image::{Dimensions, StorageImage};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::viewport::Viewport;
use vulkano::sync::GpuFuture;

/* Declare the vertex definition we will pass to the pipeline. */
#[derive(Copy, Clone)]
struct Vertex {
   position: [f32; 2],
}

vulkano::impl_vertex!(Vertex, position);

/* Declare the vertex shader */
mod vs {
   vulkano_shaders::shader!{
      ty: "vertex",
      src: "
#version 450

layout(location = 0) in vec2 position;

void main() {
   gl_Position = vec4(position, 0.0, 1.0);
}"
   }
}

/* Declare the fragment shader */
mod fs {
   vulkano_shaders::shader!{
      ty: "fragment",
      src: "
#version 450

layout(location = 0) out vec4 f_color;

void main() {
   f_color = vec4(1.0, 0.0, 0.0, 1.0);
}"
   }
}

fn main() {
   // Create the instance.
   let instance = Instance::new(None, &InstanceExtensions::none(), None)
      .expect("Failed to create instance.");

   // Get the physical interface.
   let physical = PhysicalDevice::enumerate(&instance).next().expect("No device available.");

   // Get the queue families (vulkan specific implementaiton).
   let queue_family = physical.queue_families()
      .find(|&q| q.supports_graphics())
      .expect("could not find a graphical queue family.");

   // Get the device and a list of queues we can use.
   let (device, mut queues) = {
      Device::new(physical, &Features::none(), &DeviceExtensions::none(),
         [(queue_family, 0.5)].iter().cloned()).expect("Failed to create device.")
   };

   // We only need one queue for this program.
   let queue = queues.next().unwrap();

   // Declare the vertex and fragment shaders
   let vs = vs::Shader::load(device.clone()).expect("Failed to create vertex shader.");
   let fs = fs::Shader::load(device.clone()).expect("Failed to create fragment shader.");

   // Create the vertexes we will use.
   let vertex1 = Vertex { position: [-0.5, -0.5] };
   let vertex2 = Vertex { position: [0.5, 0.5] };
   let vertex3 = Vertex { position: [0.5, -0.25] };

   // Create the vertex buffer and pass the vertexes into it.
   let vertex_buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), 
      vec![vertex1, vertex2, vertex3].into_iter()).unwrap();

   // Graphics pipelines use a render pass to do drawing, so create one.
   let render_pass = Arc::new(vulkano::single_pass_renderpass!(device.clone(),
      attachments: {
         color: {
            load: Clear,
            store: Store,
            format: Format::R8G8B8A8Unorm,
            samples: 1,
         }
      },
      pass: {
         color: [color],
         depth_stencil: {}
      }
   ).unwrap());

   // Create the pipeline object
   let pipeline = Arc::new(GraphicsPipeline::start()
      .vertex_input_single_buffer::<Vertex>()
      .vertex_shader(vs.main_entry_point(), ())
      .viewports_dynamic_scissors_irrelevant(1)
      .fragment_shader(fs.main_entry_point(), ())
      .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
      .build(device.clone())
      .unwrap());

   // Create the image object that will hold the final product.
   let image = StorageImage::new(device.clone(), Dimensions::Dim2d { width: 1024, height: 1024 },
      Format::R8G8B8A8Unorm, Some(queue.family())).unwrap();

   // Create the framebuffer.
   let framebuffer = Arc::new(Framebuffer::start(render_pass.clone())
      .add(image.clone()).unwrap()
      .build().unwrap());

   // Instantiate the dynamic state. This allows us to use draw() multiple times.
   let dynamic_state = DynamicState {
      viewports: Some(vec![Viewport {
         origin: [0.0, 0.0],
         dimensions: [1024.0, 1024.0],
         depth_range: 0.0 .. 1.0,
      }]),
      .. DynamicState::none()
   };

   // Create the buffer that will hold the final result.
   let buf = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(),
      (0 .. 1024*1024*4).map(|_| 0u8))
      .expect("Failed to create buffer.");

   // Create the input buffer and draw our picture.
   let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap()
      .begin_render_pass(framebuffer.clone(), false, vec![[0.0, 0.0, 1.0, 1.0].into()])
      .unwrap()
      .draw(pipeline.clone(), &dynamic_state, vertex_buffer.clone(), (), ())
      .unwrap()
      .end_render_pass()
      .unwrap()
      .copy_image_to_buffer(image.clone(), buf.clone())
      .unwrap()
      .build()
      .unwrap();

   // Finish and output.
   let finished = command_buffer.execute(queue.clone()).unwrap();
   finished.then_signal_fence_and_flush().unwrap()
      .wait(None).unwrap();

   let buffer_content = buf.read().unwrap();
   let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024,1024, &buffer_content[..]).unwrap();
   image.save("triangle.png").unwrap();
}
