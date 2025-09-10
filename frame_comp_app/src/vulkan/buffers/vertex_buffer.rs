use anyhow::*;
use std::ptr::copy_nonoverlapping as memcpy;
use vulkanalia::prelude::v1_3::*;

use crate::{app::AppData, vulkan::vertex::Vertex};

use super::buffer::{copy_buffer, create_buffer};

pub fn create_vertex_buffer(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    // The buffer needs to hold all our vertex data.
    let size = (size_of::<Vertex>() * data.vertices.len()) as u64;

    // Creates a staging buffer accessible to both the CPU and GPU so that we can
    // transfer the vertex data to a more optimal buffer, which the GPU will read
    // the data from when it's needed. It won't be accessible from the CPU anymore.
    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance,
        device,
        data,
        size,
        // This buffer can be used as a source in a memory transfer operation, meaning
        // the GPU will only perform transfer operations that copy from the buffer, not to it.
        vk::BufferUsageFlags::TRANSFER_SRC,
        // The memory must be host-coherent and host-visible to allow CPU access.
        // HOST_VISIBLE: The memory can be accessed by the CPU.
        // HOST_COHERENT: Ensures that changes made by the CPU are automatically visible
        //   to the GPU without the need for explicit flushing.
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    unsafe {
        let memory =
            device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

        // Copies the vertex data from CPU memory to the GPU-accessible memory region.
        memcpy(data.vertices.as_ptr(), memory.cast(), data.vertices.len());

        // Unmap the memory after writing to ensure all changes are visible to the GPU.
        device.unmap_memory(staging_buffer_memory);
    }

    let (vertex_buffer, vertex_buffer_memory) = create_buffer(
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    data.vertex_buffer = vertex_buffer;
    data.vertex_buffer_memory = vertex_buffer_memory;

    copy_buffer(device, data, staging_buffer, vertex_buffer, size)?;

    unsafe {
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);
    }

    Ok(())
}
