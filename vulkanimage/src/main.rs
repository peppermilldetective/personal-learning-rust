extern crate image;
extern crate vulkano;

use image::{ImageBuffer, Rgba};

use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;

use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::buffer::BufferUsage;

use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBuffer;

use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;

use vulkano::format::Format;
use vulkano::format::ClearValue;

use vulkano::sync::GpuFuture;

use vulkano::image::Dimensions;
use vulkano::image::StorageImage;

fn main() {
   let instance = Instance::new(None, &InstanceExtensions::none(), None)
      .expect("Failed to create instance");

   let physical = PhysicalDevice::enumerate(&instance).next().expect("No device available");

   let queue_family = physical.queue_families()
      .find(|&q| q.supports_graphics())
      .expect("Could not find a graphical queue family.");

   let (device, mut queues) = {
      Device::new(physical, &Features::none(), &DeviceExtensions::none(),
         [(queue_family, 0.5)].iter().cloned()).expect("Failed to create device.")
   };

   let queue = queues.next().unwrap();

   let image = StorageImage::new(device.clone(), Dimensions::Dim2d { width: 1024, height: 1024 }, 
      Format::R8G8B8A8Unorm, Some(queue.family())).unwrap();

   let buf = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), 
      (0 .. 1024*1024*4).map(|_| 0u8))
      .expect("Failed to create buffer.");

   let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap()
      .clear_color_image(image.clone(), ClearValue::Float([0.0, 0.0, 1.0, 1.0])).unwrap()
      .copy_image_to_buffer(image.clone(), buf.clone()).unwrap()
      .build().unwrap();

   let finished = command_buffer.execute(queue.clone()).unwrap();
   finished.then_signal_fence_and_flush().unwrap()
      .wait(None).unwrap();

   let buffer_content = buf.read().unwrap();
   let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();

   image.save("image.png").unwrap();
}
