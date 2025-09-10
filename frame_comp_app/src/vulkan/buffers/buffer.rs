use anyhow::*;
use vulkanalia::prelude::v1_3::*;

use crate::{
    app::AppData,
    vulkan::commands::{begin_single_time_commands, end_single_time_commands},
};

pub fn create_buffer(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        // This buffer will be used only by a single queue (the graphics queue),
        // so we can make it exclusive for better performance.
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    // This creates a buffer handle, but no memory is allocated for it yet.
    let buffer = unsafe { device.create_buffer(&buffer_info, None) }?;

    // These are the requirements for the buffer that we need to find suitable memory for.
    // The memory must be big enough and satisfy the right requirements (needs to have
    // the right memory type bits set).
    let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    let memory_type_index = get_memory_type_index(instance, data, properties, requirements)?;

    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(memory_type_index);

    let buffer_memory = unsafe { device.allocate_memory(&memory_info, None)? };

    // If the offset happens to be non-zero, it must be divisible by requirements.alignment.
    unsafe { device.bind_buffer_memory(buffer, buffer_memory, 0) }?;

    Ok((buffer, buffer_memory))
}

/// Returns a memory type index for memory that satisfies the given requirements
/// and has the given properties.
pub fn get_memory_type_index(
    instance: &Instance,
    data: &mut AppData,
    properties: vk::MemoryPropertyFlags,
    requirements: vk::MemoryRequirements,
) -> Result<u32> {
    let memory: vk::PhysicalDeviceMemoryProperties =
        unsafe { instance.get_physical_device_memory_properties(data.physical_device) };

    (0..memory.memory_type_count)
        .find(|i| {
            let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
            let memory_type: vk::MemoryType = memory.memory_types[*i as usize];

            suitable && memory_type.property_flags.contains(properties)
        })
        .ok_or_else(|| anyhow!("Failed to find suitable memory type."))
}

/// Copies data from one buffer to another.
/// The source buffer has to have the vk::BufferUsageFlags::TRANSFER_SRC
/// and the destination buffer has to have the VK::BufferUsageFlags::TRANSFER_DST flags.
pub fn copy_buffer(
    device: &Device,
    data: &AppData,
    source: vk::Buffer,
    destination: vk::Buffer,
    size: vk::DeviceSize,
) -> Result<()> {
    let region = vk::BufferCopy::builder().size(size);

    unsafe {
        let command_buffer = begin_single_time_commands(device, data)?;
        device.cmd_copy_buffer(command_buffer, source, destination, &[region]);
        end_single_time_commands(device, data, command_buffer)?;
    }

    Ok(())
}
