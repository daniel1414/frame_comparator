use anyhow::Result;
use vulkanalia::prelude::v1_3::*;

use frame_comp::FrameCompareInfo;

use crate::app::AppData;

use super::queue::QueueFamilyIndices;

/// A command pool is an object used to manage the memory allocation of command buffers.
/// Since command buffers are stored in GPU-accessible memory, the command pool
/// serves as the allocator for these buffers. Command pools also define the scope of
/// command buffer lifecycle and help manage their reuse.
///
/// Command pools are needed for:
///
/// Efficient memory management: Instead of individually allocating GPU memory for each
///     command buffer, Vulkan uses pools for bulk allocation, reducing overhead.
/// Thread safety: Each thread typically gets its own command pool to avoid contention
///     during command buffer allocation and recording.
///
/// When a command pool is destroyed, all command buffers allocated from it
/// are reset or destroyed automatically.
pub fn create_command_pool(instance: &Instance, device: &Device, data: &mut AppData) -> Result<()> {
    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;

    let info = vk::CommandPoolCreateInfo::builder()
        // Options for a command pool:
        // 1. Transient: Optimized for short-lived command buffers.
        // 2. Resettable command buffers: Command buffers allocated from this pool
        //      can be individually reset, rather than resetting the entire pool.
        .flags(vk::CommandPoolCreateFlags::empty())
        // The command pool created is tied to a specific queue family, and thus
        // all the buffers allocated from it are tied to the same queue family as well.
        .queue_family_index(indices.graphics)
        .build();

    data.command_pool = unsafe { device.create_command_pool(&info, None) }?;

    Ok(())
}

/// A command buffer is a container that stores a sequence of GPU commands. These commands
/// tell Vulkan what to do, such as rendering, memory transfers, or pipeline state changes.
///
/// They are pre-recorded and submitted to a Vulkan queue for execution. This enables efficient
/// batching of commands and offloading work to the GPU.
///
/// Buffers are needed for:
///
/// Minimized overhead: Commands are once recorded and reused, avoiding expensive per-frame CPU processing.
/// Parallelism: Multiple threads can record commands into different command buffers
///     simultaneously maximizing CPU utilization.
/// Synchronization: By organizing commands into discrete units (command buffers),
///     Vulkan can better synchronize GPU workloads and resource access.
///
/// Primary buffers: Can be submitted directly to the queue - used for
///     high-level commands and orchestration.
/// Secondary buffers: Cannot be submitted directly: they must be executed
///     by a primary command buffer. Useful for splitting work across threads
///     (recording different parts of a scene)
///
pub fn create_command_buffers(device: &Device, data: &mut AppData) -> Result<()> {
    let alloc_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(data.command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(data.swapchain_image_views.len() as u32);

    data.command_buffers = unsafe { device.allocate_command_buffers(&alloc_info) }?;

    for (i, command_buffer) in data.command_buffers.iter().enumerate() {
        let inheritance = vk::CommandBufferInheritanceInfo::builder();

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::empty())
            .inheritance_info(&inheritance);

        //let vbar_width: u32 = 4;
        //let left_width = (data.swapchain_extent.width as f64 * data.vbar_percentage) as u32;

        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(vk::Extent2D {
                width: data.swapchain_extent.width,
                height: data.swapchain_extent.height,
            })
            .build();

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.02, 0.02, 0.02, 1.0],
            },
        };

        let depth_clear_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            },
        };

        // The order of clear values should be identical to the order of attachments.
        let clear_values = &[color_clear_value, depth_clear_value];
        let left_render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(data.render_pass[0])
            .framebuffer(data.left_framebuffers[i])
            .render_area(render_area)
            .clear_values(clear_values);

        let right_render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(data.render_pass[1])
            .framebuffer(data.right_framebuffers[i])
            .render_area(render_area)
            .clear_values(clear_values);

        let loop_data = &[
            (
                left_render_pass_begin_info,
                data.left_pipeline_layout,
                data.left_pipeline,
            ),
            (
                right_render_pass_begin_info,
                data.right_pipeline_layout,
                data.right_pipeline,
            ),
        ];

        unsafe {
            device.begin_command_buffer(*command_buffer, &info)?;

            device.cmd_bind_vertex_buffers(*command_buffer, 0, &[data.vertex_buffer], &[0]);
            device.cmd_bind_index_buffer(
                *command_buffer,
                data.index_buffer,
                0,
                vk::IndexType::UINT32,
            );

            for (render_pass_begin_info, pipeline_layout, pipeline) in loop_data {
                // Render pass for the left half of the image.
                device.cmd_begin_render_pass(
                    *command_buffer,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                );

                // The command buffer tracks state changes (e.g., pipeline bindings) and
                // ensures dependencies are managed correctly.
                // The pipeline is meant to operate on attachments and the render pass describes them
                // so the pipeline needs to be bound only after the render pass begins.
                device.cmd_bind_pipeline(
                    *command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    *pipeline,
                );
                device.cmd_bind_descriptor_sets(
                    *command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    *pipeline_layout,
                    0,
                    &[data.descriptor_sets[i]],
                    &[],
                );

                device.cmd_draw_indexed(*command_buffer, data.indices.len() as u32, 1, 0, 0, 0);
                device.cmd_end_render_pass(*command_buffer);
            }

            // Compare the outputs
            if let Some((comparator, framebuffers)) = &data.frame_comparator {
                let compare_info = FrameCompareInfo::builder()
                    .command_buffer(*command_buffer)
                    .out_framebuffer(framebuffers[i])
                    .divider_position(data.vbar_percentage)
                    .build()?;

                comparator.compare(&compare_info)?;
            }

            device.end_command_buffer(*command_buffer)?;
        }
    }

    Ok(())
}

pub fn begin_single_time_commands(device: &Device, data: &AppData) -> Result<vk::CommandBuffer> {
    let info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(data.command_pool)
        .command_buffer_count(1);

    let command_buffer = unsafe { device.allocate_command_buffers(&info) }?[0];

    let info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe { device.begin_command_buffer(command_buffer, &info) }?;

    Ok(command_buffer)
}

pub fn end_single_time_commands(
    device: &Device,
    data: &AppData,
    command_buffer: vk::CommandBuffer,
) -> Result<()> {
    let command_buffers = &[command_buffer];
    let info = vk::SubmitInfo::builder().command_buffers(command_buffers);

    unsafe {
        device.end_command_buffer(command_buffer)?;

        device.queue_submit(data.graphics_queue, &[info], vk::Fence::null())?;
        device.queue_wait_idle(data.graphics_queue)?;

        device.free_command_buffers(data.command_pool, command_buffers);
    }

    Ok(())
}
