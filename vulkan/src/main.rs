#[macro_use]
extern crate vulkano;
extern crate vulkano_shaders;

use std::sync::Arc;

use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;

use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;

use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;

use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBuffer;

use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;

use vulkano::pipeline::ComputePipeline;

use vulkano::sync::GpuFuture;

mod cs {
   vulkano_shaders::shader!{
      ty: "compute",
      src: "
#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer Data {
   uint data[];
} buf;

void main() {
   uint idx = gl_GlobalInvocationID.x;
   buf.data[idx] *= 12;
}"
   }
}

fn main() {
   let instance = Instance::new(None, &InstanceExtensions::none(), None)
      .expect("Failed to create instance.");

   let physical = PhysicalDevice::enumerate(&instance).next().expect("No device available");

   let queue_family = physical.queue_families()
      .find(|&q| q.supports_graphics())
      .expect("Could not find a graphical queue family.");

   let (device, mut queues) = {
      Device::new(physical, &Features::none(), &DeviceExtensions::none(),
         [(queue_family, 0.5)].iter().cloned()).expect("Failed to create device.")
   };

   let queue = queues.next().unwrap();

   let source_content = 0 .. 64;
   let source = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), source_content)
      .expect("Failed to create buffer.");

   let dest_content = (0 .. 64).map(|_| 0);
   let dest = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), dest_content)
      .expect("Failed to create buffer.");

   let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap()
      .copy_buffer(source.clone(), dest.clone()).unwrap()
      .build().unwrap();

   let finished = command_buffer.execute(queue.clone()).unwrap();

   finished.then_signal_fence_and_flush().unwrap().wait(None).unwrap();

   let src_content = source.read().unwrap();
   let dest_content = dest.read().unwrap();
   assert_eq!(&*src_content, &*dest_content);

   let data_iter = 0 .. 65536;
   let data_buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), data_iter)
      .expect("Failed to create buffer");

   let shader = cs::Shader::load(device.clone())
      .expect("Failed to create shader module");

   let compute_pipeline = Arc::new(ComputePipeline::new(device.clone(), &shader.main_entry_point(), &())
      .expect("Failed to create compute pipeline"));

   let set = Arc::new(PersistentDescriptorSet::start(compute_pipeline.clone(), 0)
      .add_buffer(data_buffer.clone()).unwrap()
      .build().unwrap());

   let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap()
      .dispatch([1024, 1, 1], compute_pipeline.clone(), set.clone(), ()).unwrap()
      .build().unwrap();

   let finished = command_buffer.execute(queue.clone()).unwrap();

   finished.then_signal_fence_and_flush().unwrap()
      .wait(None).unwrap();

   let content = data_buffer.read().unwrap();

   for (n, val) in content.iter().enumerate() {
      assert_eq!(*val, n as u32 * 12);
   }

   println!("Everything succeeded!");
}