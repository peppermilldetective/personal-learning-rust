#[macro_use]
extern crate vulkano;
extern crate vulkano_shaders;
extern crate vulkano_win;
extern crate winit;

use std::sync::Arc;

use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, DeviceExtensions};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, Subpass, RenderPassAbstract};
use vulkano::image::SwapchainImage;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain::{AcquireError, PresentMode, SurfaceTransform, Swapchain, SwapchainCreationError};
use vulkano::swapchain;
use vulkano::sync::{GpuFuture, FlushError};
use vulkano::sync;

use vulkano_win::VkSurfaceBuild;

use winit::{EventsLoop, Window, WindowBuilder, Event, WindowEvent};

fn main() {
   let instance = {
      let extensions = vulkano_win::required_extensions();
      Instance::new(None, &extensions, None).expect("Failed to create Vulkan instance.")
   };

   let physical = PhysicalDevice::enumerate(&instance).next().expect("No device available.");

   println!("Using device: {} (type: {:?})", physical.name(), physical.ty());

   let mut events_loop = EventsLoop::new();
   let surface = WindowBuilder::new().build_vk_surface(&events_loop, instance.clone()).unwrap();
   let window = surface.window();

   let queue_family = physical.queue_families().find(|&q| {
      q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
   }).unwrap();

   let (device, mut queues) = {
      let device_ext = vulkano::device::DeviceExtensions {
         khr_swapchain: true,
         .. vulkano::device::DeviceExtensions::none()
      };

      Device::new(physical, physical.supported_features(), &device_ext,
         [(queue_family, 0.5)].iter().cloned()).expect("Failed to create device.")
   };

   let queue = queues.next().unwrap();

   let (mut swapchain, images) = {
      let caps = surface.capabilities(physical).unwrap();

      let usage = caps.supported_usage_flags;

      let alpha = caps.supported_composite_alpha.iter().next().unwrap();
      let format = caps.supported_formats[0].0;

      let initial_dimensions = if let Some(dimensions) = window.get_inner_size() {
         let dimensions: (u32, u32) = dimensions.to_physical(window.get_hidpi_factor()).into();
         [dimensions.0, dimensions.1]
      } else {
         // The window no longer exists, so exit.
         return;
      };

      Swapchain::new(device.clone(), surface.clone(), caps.min_image_count, format, 
         initial_dimensions, 1, usage, &queue, SurfaceTransform::Identity, alpha, 
         PresentMode::Fifo, true, None).unwrap()
   };

   #[derive(Debug, Clone)]
   struct Vertex { position: [f32; 2] }
   impl_vertex!(Vertex, position);

   let vertex_buffer = {
      CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), [
         Vertex { position: [-0.5, -0.25] },
         Vertex { position: [0.0, 0.5] },
         Vertex { position: [0.25, -0.1] }
      ].iter().cloned()).unwrap()
   };

   mod vs {
      vulkano_shaders::shader!{
         ty: "vertex",
         src: "
#version 450

layout(location = 0) in vec2 position;

void main() {
   gl_Position = vec4(position, 0.0, 1.0);
}
"
      }
   }

   mod fs {
      vulkano_shaders::shader!{
         ty: "fragment",
         src: "
#version 450

layout(location = 0) out vec4 f_color;

void main() {
   f_color = vec4(1.0, 0.0, 0.0, 1.0);
}
"
      }
   }

   let vs = vs::Shader::load(device.clone()).unwrap();
   let fs = fs::Shader::load(device.clone()).unwrap();

   let render_pass = Arc::new(single_pass_renderpass!(
      device.clone(),
      attachments: {
         color: {
            load: Clear,
            store: Store,
            format: swapchain.format(),
            samples: 1,
         }
      },
      pass: {
         color: [color],
         depth_stencil: {}
      }
   ).unwrap());

   let pipeline = Arc::new(GraphicsPipeline::start()
      .vertex_input_single_buffer::<Vertex>()
      .vertex_shader(vs.main_entry_point(), ())
      .triangle_list()
      .viewports_dynamic_scissors_irrelevant(1)
      .fragment_shader(fs.main_entry_point(), ())
      .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
      .build(device.clone())
      .unwrap()); 

   let mut dynamic_state = DynamicState { line_width: None, viewports: None, scissors: None };

   let mut framebuffers = {
      let dimensions = images[0].dimensions();

      let viewport = Viewport {
         origin: [0.0, 0.0],
         dimensions: [dimensions[0] as f32, dimensions[1] as f32],
         depth_range: 0.0 .. 1.0,
      };

      dynamic_state.viewports = Some(vec!(viewport));

      images.iter().map(|image| {
         Arc::new(
            Framebuffer::start(render_pass.clone())
               .add(image.clone()).unwrap()
               .build().unwrap()
         ) as Arc<FramebufferAbstract + Send + Sync>
      }).collect::<Vec<_>>()
   };

   let mut recreate_swapchain = false;

   let mut previous_frame_end = Box::new(sync::now(device.clone())) as Box<GpuFuture>;

   loop {
      previous_frame_end.cleanup_finished();

      if recreate_swapchain {
         let dimensions = if let Some(dimensions) = window.get_inner_size() {
            let dimensions: (u32, u32) = dimensions.to_physical(window.get_hidpi_factor()).into();
            [dimensions.0, dimensions.1]
         } else {
            return;
         };

         let (new_swapchain, new_images) = match swapchain.recreate_with_dimension(dimensions) {
            Ok(r) => r,
            Err(SwapchainCreationError::UnsupportedDimensions) => continue,
            Err(err) => panic!("{:?}", err)
         };

         swapchain = new_swapchain;

         let mut framebuffers = {
            let dimensions = images[0].dimensions();

            let viewport = Viewport {
               origin: [0.0, 0.0],
               dimensions: [dimensions[0] as f32, dimensions[1] as f32],
               depth_range: 0.0 .. 1.0,
            };

            dynamic_state.viewports = Some(vec!(viewport));

            images.iter().map(|image| {
               Arc::new(
                  Framebuffer::start(render_pass.clone())
                     .add(image.clone()).unwrap()
                     .build().unwrap()
               ) as Arc<FramebufferAbstract + Send + Sync>
            }).collect::<Vec<_>>()
         };

         recreate_swapchain = false;
      }

      let (image_num, acquire_future) = match swapchain::acquire_next_image(swapchain.clone(), None) {
         Ok(r) => r,
         Err(AcquireError::OutOfDate) => {
            recreate_swapchain = true;
            continue;
         },
         Err(err) => panic!("{:?}", err)
      };

      let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap()
         .begin_render_pass(framebuffers[image_num].clone(), false, vec![[0.0, 0.0, 1.0].into()])
         .unwrap()
         .end_render_pass()
         .unwrap()
         .build().unwrap();

      let future = previous_frame_end.join(acquire_future)
         .then_execute(queue.clone(), command_buffer).unwrap()
         .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
         .then_signal_fence_and_flush();

      match future {
         Ok(future) => {
            previous_frame_end = Box::new(future) as Box<_>;
         }
         Err(FlushError::OutOfDate) => {
            recreate_swapchain = true;
            previous_frame_end = Box::new(sync::now(device.clone())) as Box<_>;
         }
         Err(e) => {
            println!("{:?}", e);
            previous_frame_end = Box::new(sync::now(device.clone())) as Box<_>;
         }
      }

      let mut done = false;
      events_loop.poll_events(|ev| {
         match ev {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => done = true,
            Event::WindowEvent { event: WindowEvent::Resized(_), .. } => recreate_swapchain = true,
            _ => ()
         }
      });

      if done { return; }
   }
}
