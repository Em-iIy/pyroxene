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
	},
	command_buffer::{
		self, allocator::{
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
// https://vulkano.rs/03-buffer-creation/02-example-operation.html#submission-and-synchronization
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

    for queue in queues {
        println!("{:?}", queue);
    }
    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
	let iter = (0..128).map(|_| 4u8);

	let source_content: Vec<i32> = (0..64).collect();
	let source = Buffer::from_iter(
		memory_allocator.clone(),
		BufferCreateInfo {
			usage: BufferUsage::TRANSFER_SRC,
			..Default::default()
		},
		AllocationCreateInfo {
			memory_type_filter: MemoryTypeFilter::PREFER_HOST
				| MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
			..Default::default()
		},
		source_content,
	)
	.expect("failed to create source buffer");

	let destination_content: Vec<i32> = (0..64).map(|_| 0).collect();
	let destination = Buffer::from_iter(
		memory_allocator.clone(),
		BufferCreateInfo {
			usage: BufferUsage::TRANSFER_DST,
			..Default::default()
		},
		AllocationCreateInfo {
			memory_type_filter: MemoryTypeFilter::PREFER_HOST
			    | MemoryTypeFilter::HOST_RANDOM_ACCESS,
			..Default::default()
		},
		destination_content,
	)
	.expect("failed to create destination buffer");

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

	builder
		.copy_buffer(CopyBufferInfo::buffers(source.clone(), destination.clone()))
		.unwrap();

	let command_buffer = builder.build().unwrap();
}
