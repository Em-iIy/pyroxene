/*
 * File Created: Wednesday, 13th November 2024 2:47:58 pm
 * Author: Emily (Em-iIy) Winnink (emily.winnink@gmail.com)
*/


use std::sync::Arc;

use vulkano::{
	buffer::{
		Buffer,
		BufferCreateInfo,
		BufferUsage,
	}, command_buffer::{
		self, allocator::{
			StandardCommandBufferAllocator,
			StandardCommandBufferAllocatorCreateInfo,
		}, AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo
	}, descriptor_set::{
		allocator::StandardDescriptorSetAllocator,
		PersistentDescriptorSet,
		WriteDescriptorSet,
	}, device::{
        Device,
        DeviceCreateInfo,
        QueueCreateInfo,
        QueueFlags,
    }, instance::{
        Instance,
        InstanceCreateFlags,
        InstanceCreateInfo,
    }, memory::allocator::{
        AllocationCreateInfo,
		MemoryTypeFilter,
		StandardMemoryAllocator
    }, pipeline::{
		compute::ComputePipelineCreateInfo, layout::PipelineDescriptorSetLayoutCreateInfo, ComputePipeline, Pipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo
	}, sync::{
		self,
		GpuFuture,
	}, VulkanLibrary
};


// current step:
// https://vulkano.rs/05-images/01-image-creation.html
fn main() {
    let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
    let instance = Instance::new(
    library,
    InstanceCreateInfo {
        flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
        ..Default::default()
    },
    )
    .expect("failed to create instance");

	for device in instance.enumerate_physical_devices().expect("could not enumerate devices") {
		println!("device: {:?}", device.properties().device_name);
	}

    let physical_device = instance
		.enumerate_physical_devices()
		.expect("could not enumerate devices")
		.next()
		.expect("no devices available");

    println!("{}", physical_device.properties().device_name);
    for family in physical_device.queue_family_properties() {
        println!("Found a queue family with {:?} queue(s)", family.queue_count);
    }

    let queue_family_index = physical_device
        .queue_family_properties()
        .iter()
        .enumerate()
        .position(|(_queue_family_index, queue_family_properties)| {
            queue_family_properties.queue_flags.contains(QueueFlags::GRAPHICS)
        })
        .expect("couldn't find a graphical queue family") as u32;

    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .expect("failed to create device");

    let queue = queues.next().unwrap();

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

	let data_iter = 0..65536u32;
	let data_buffer = Buffer::from_iter(
		memory_allocator.clone(),
		BufferCreateInfo {
			usage: BufferUsage::STORAGE_BUFFER,
			..Default::default()
		},
		AllocationCreateInfo {
			memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
				| MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
			..Default::default()
		},
		data_iter,
	)
	.expect("failed to create buffer");

	let command_buffer_allocator = StandardCommandBufferAllocator::new(
		device.clone(),
		StandardCommandBufferAllocatorCreateInfo::default(),
	);
	let mut builder = AutoCommandBufferBuilder::primary(
		&command_buffer_allocator,
		queue_family_index,
		CommandBufferUsage::OneTimeSubmit,
	)
	.unwrap();

	mod cs {
		vulkano_shaders::shader!{
			ty: "compute", // type?
			src: r"
				#version 460

				layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

				layout(set = 0, binding = 0) buffer Data {
					uint data[];
				} buf;

				void main() {
					uint idx = gl_GlobalInvocationID.x;
					buf.data[idx] *= 12;
				}
			",
		}
	};

	let shader = cs::load(device.clone()).expect("failed to create shader module");
	let cs = shader.entry_point("main").unwrap();
	let stage = PipelineShaderStageCreateInfo::new(cs);
	let layout = PipelineLayout::new(
		device.clone(),
		PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
			.into_pipeline_layout_create_info(device.clone())
			.unwrap(),
	)
	.unwrap();

	let compute_pipeline = ComputePipeline::new(
		device.clone(),
		None,
		ComputePipelineCreateInfo::stage_layout(stage, layout),
	)
	.expect("failed to create compute pipeline");

	let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone(), Default::default());
	let pipeline_layout = compute_pipeline.layout();
	let descriptor_set_layouts = pipeline_layout.set_layouts();

	let descriptor_set_layout_index = 0;
	let descriptor_set_layout = descriptor_set_layouts
		.get(descriptor_set_layout_index)
		.unwrap();
	let descriptor_set = PersistentDescriptorSet::new(
		&descriptor_set_allocator,
		descriptor_set_layout.clone(),
		[WriteDescriptorSet::buffer(0, data_buffer.clone())],
		[],
	)
	.unwrap();

	let command_buffer_allocator = StandardCommandBufferAllocator::new(
		device.clone(),
		StandardCommandBufferAllocatorCreateInfo::default(),
	);

	let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
		&command_buffer_allocator,
		queue.queue_family_index(),
		CommandBufferUsage::OneTimeSubmit,
	)
	.unwrap();

	let work_group_counts = [1024, 1, 1];

	command_buffer_builder
		.bind_pipeline_compute(compute_pipeline.clone())
		.unwrap()
		.bind_descriptor_sets(
			PipelineBindPoint::Compute,
			compute_pipeline.layout().clone(),
			descriptor_set_layout_index as u32,
			descriptor_set,
		)
		.unwrap()
		.dispatch(work_group_counts)
		.unwrap();

	let command_buffer = command_buffer_builder.build().unwrap();
	// builder
	// 	.copy_buffer(CopyBufferInfo::buffers(source.clone(), destination.clone()))
	// 	.unwrap();

	// let command_buffer = builder.build().unwrap();

	let future = sync::now(device.clone())
		.then_execute(queue.clone(), command_buffer)
		.unwrap()
		.then_signal_fence_and_flush()
		.unwrap();

	future.wait(None).unwrap();

	let content = data_buffer.read().unwrap();
	for (n, val) in content.iter().enumerate() {
		assert_eq!(*val, n as u32 * 12);
	}
	println!("Compute shader sucessfully ran!");
	// let src_content = source.read().unwrap();
	// let dst_content = destination.read().unwrap();
	// assert_eq!(&*src_content, &*dst_content);
	// println!("Copy successful! dstcontent: {:?}", dst_content);
}
