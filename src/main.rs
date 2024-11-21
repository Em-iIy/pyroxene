/*
 * File Created: Wednesday, 13th November 2024 2:47:58 pm
 * Author: Emily (Em-iIy) Winnink (emily.winnink@gmail.com)
*/


use std::sync::Arc;

use vulkano::{
	sync::{
		self,
		GpuFuture,
	},
    buffer::{
		Buffer,
		BufferCreateInfo,
		BufferUsage,
	},
	command_buffer::{
		allocator::{
			StandardCommandBufferAllocator,
			StandardCommandBufferAllocatorCreateInfo,
		}, AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo
	},
	device::{
        Device,
        DeviceCreateInfo,
        QueueCreateInfo,
        QueueFlags,
    },
	instance::{
        Instance,
        InstanceCreateFlags,
        InstanceCreateInfo,
    },
	memory::allocator::{
        AllocationCreateInfo,
		MemoryTypeFilter,
		StandardMemoryAllocator
    },
	VulkanLibrary
};


// current step:
// https://vulkano.rs/04-compute-pipeline/02-compute-pipeline.html#compute-pipelines
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

	// builder
	// 	.copy_buffer(CopyBufferInfo::buffers(source.clone(), destination.clone()))
	// 	.unwrap();

	// let command_buffer = builder.build().unwrap();

	// let future = sync::now(device.clone())
	// 	.then_execute(queue.clone(), command_buffer)
	// 	.unwrap()
	// 	.then_signal_fence_and_flush()
	// 	.unwrap();

	// future.wait(None).unwrap();

	// let src_content = source.read().unwrap();
	// let dst_content = destination.read().unwrap();
	// assert_eq!(&*src_content, &*dst_content);
	// println!("Copy successful! dstcontent: {:?}", dst_content);
}
