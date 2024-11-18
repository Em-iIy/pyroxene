/*
 * File Created: Wednesday, 13th November 2024 2:47:58 pm
 * Author: Emily (Em-iIy) Winnink (emily.winnink@gmail.com)
*/


use std::sync::Arc;

use vulkano::{
    VulkanLibrary,
    instance::{
        Instance,
        InstanceCreateFlags,
        InstanceCreateInfo,
    },
    device::{
        Device,
        DeviceCreateInfo,
        QueueCreateInfo,
        QueueFlags,
    },
    memory::{
        allocator::{
            StandardMemoryAllocator,
        },
    },
};


// current step:
// https://vulkano.rs/03-buffer-creation/01-buffer-creation.html
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
}
