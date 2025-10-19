use anyhow::Result;
use vulkanalia::prelude::v1_3::*;

use crate::app::AppData;

use super::buffer::create_buffer;

pub type Mat4 = cgmath::Matrix4<f32>;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UniformBufferObject {
    pub model: Mat4,
    pub view: Mat4,
    pub proj: Mat4,
}

pub fn create_uniform_buffers(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    data.uniform_buffers.clear();
    data.uniform_buffers_memory.clear();

    for _ in 0..data.swapchain_images.len() {
        let (uniform_buffer, uniform_buffer_memory) = create_buffer(
            instance,
            device,
            data,
            size_of::<UniformBufferObject>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        data.uniform_buffers.push(uniform_buffer);
        data.uniform_buffers_memory.push(uniform_buffer_memory);
    }

    Ok(())
}

/// This function should probably take in a descriptor type and the stage flags
/// for more flexibility. That's to be done when we will need descriptor set layouts
/// other than the one for the uniform buffer.
///
/// A descriptor set layout defines the structure of descriptors visible to shaders.
pub fn create_descriptor_set_layout(device: &Device, data: &mut AppData) -> Result<()> {
    let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX);

    let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);

    let bindings = &[ubo_binding, sampler_binding];
    let info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings);

    data.descriptor_set_layout = unsafe { device.create_descriptor_set_layout(&info, None) }?;

    Ok(())
}

/// A descriptor pool is an object that manages the memory required for allocating descriptor sets.
/// Pools alow efficient batch allocation and destruction of descriptor sets.
pub fn create_descriptor_pool(device: &Device, data: &mut AppData) -> Result<()> {
    let ubo_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER)
        // We want to allocate one UBO for every swapchain image.
        .descriptor_count(data.swapchain_images.len() as u32 * 2);

    let sampler_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(data.swapchain_images.len() as u32 * 20);

    let pool_sizes = &[ubo_size, sampler_size];
    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(pool_sizes)
        .max_sets(data.swapchain_images.len() as u32 * 2);

    data.descriptor_pool = unsafe { device.create_descriptor_pool(&info, None) }?;

    Ok(())
}

/// A descriptor is an object, that specifies how a shader accesses a resource.
/// It is metadata that tells Vulkan:
/// What resource to access (e.g., a uniform buffer, storage buffer, sampled image, etc.)
/// How to access (e.g., read-only, read-write, etc.)
///
/// Descriptor types:
///
/// UNIFORM_BUFFER: Used for UBOs like the MVP matrix.
/// STORAGE_BUFFER: Used for general-purpose storage buffers.
/// SAMPLED_IMAGE/COMBINED_IMAGE_SAMPLER: Used for sampled textures and their samplers.
/// STORAGE_IMAGE: Used for images that shaders can read from or write to directly.
///
/// Each descriptor is associated with a binding point in the shader (binding = n in the shader).
///
/// A descriptor set is a collection of descriptors grouped together. Represents a set of
/// resources that are made available to the shaders at the same time.
/// The sets are bound to the pipeline before issuing draw calls.
///
///
pub fn create_descriptor_sets(device: &Device, data: &mut AppData) -> Result<()> {
    // We use the same layout for all swapchain images.
    let layouts = vec![data.descriptor_set_layout; data.swapchain_images.len()];

    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(data.descriptor_pool)
        .set_layouts(&layouts);

    data.descriptor_sets = unsafe { device.allocate_descriptor_sets(&info) }?;

    for i in 0..data.swapchain_images.len() {
        let buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(data.uniform_buffers[i])
            .offset(0)
            .range(size_of::<UniformBufferObject>() as u64);

        let buffer_infos = &[buffer_info];

        let ubo_write = vk::WriteDescriptorSet::builder()
            .dst_set(data.descriptor_sets[i])
            .dst_binding(0)
            // Descriptors can be arrays, but we're not using one.
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            // The buffer_info field is used for descriptors that refer to buffer data,
            // image_info - descriptors that refer to image data and
            // texel_buffer_view -descriptors that refer to buffer views.
            .buffer_info(buffer_infos);

        let image_info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(data.texture_image_view)
            .sampler(data.texture_sampler);

        let image_infos = &[image_info];

        let sampler_write = vk::WriteDescriptorSet::builder()
            .dst_set(data.descriptor_sets[i])
            .dst_binding(1)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(image_infos);

        // The second argument can be used to copy descriptor sets to each other.
        unsafe {
            device.update_descriptor_sets(
                &[ubo_write, sampler_write],
                &[] as &[vk::CopyDescriptorSet],
            )
        };
    }

    Ok(())
}
