use crate::app::AppData;
use anyhow::Result;
use vulkanalia::prelude::v1_3::*;

use super::buffers::depth_buffer::get_depth_format;

/// A Vulkan render pass is a high-level container for rendering operations.
/// It defines attachments (images used during rendering),
/// subpasses (a sequence of operations that may reuse the same attachments),
/// and dependencies (define how data flows between subpasses or rendering stages).
///
/// Image views created for the swapchain images are the resources that will be
/// attached to the render pass during rendering.
pub fn create_render_pass(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<vk::RenderPass> {
    let color_attachment = vk::AttachmentDescription::builder()
        // Format of the color attachment should be same as the swapchain images.
        .format(data.swapchain_format)
        // For multisampling (anti-aliasing)
        .samples(data.msaa_samples)
        // Defines what happens to the attachment at the start of rendering
        .load_op(vk::AttachmentLoadOp::CLEAR)
        // What happens to the attachment after rendering
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        // Expected layout of the attachment before rendering.
        .initial_layout(vk::ImageLayout::UNDEFINED)
        // Defines what the final layout of the attachment should be after rendering.
        .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .build();

    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let color_attachments = &[color_attachment_ref];

    let depth_stencil_attachment = vk::AttachmentDescription::builder()
        .format(get_depth_format(instance, data)?)
        .samples(data.msaa_samples)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        // We don't care about the depth data as it won't be used after drawing
        // has finished. Contrary to the color attachment, which is used to
        // present images to the screen. This may allow the hardware to perform
        // additional optimizations.
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
        .build();

    let depth_stencil_attachment_ref = vk::AttachmentReference::builder()
        .attachment(1)
        .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

    let color_resolve_attachment = vk::AttachmentDescription::builder()
        .format(data.swapchain_format)
        .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::DONT_CARE)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .build();

    let color_resolve_attachment_ref = vk::AttachmentReference::builder()
        .attachment(2)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let resolve_attachments = &[color_resolve_attachment_ref];

    let subpass = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(color_attachments)
        .depth_stencil_attachment(&depth_stencil_attachment_ref)
        .resolve_attachments(resolve_attachments);

    // This dependency makes sure that the swapchain image is ready to be written to
    // in the first subpass. Ensures pipeline and memory synchronization.
    let dependency = vk::SubpassDependency::builder()
        // The source of the dependency is not part of the render pass. It refers to operations
        // outside the render pass, such as image layout transitions after aquiring a swapchain image
        // and any preceding rendering or compute operations that could affect the attachments.
        .src_subpass(vk::SUBPASS_EXTERNAL)
        // The destination subpass is the one and only we have, which has been created above.
        .dst_subpass(0)
        // Specifies the pipeline stage(s) in the source scope that need to be synchronized.
        // COLOR_ATTACHMENT_OUTPUT represents the stage where color attachment writes occur.
        // In this case, Vulkan ensures that color attachment output from operations
        // outside the render pass is finished before continuing (e.g. presenting to the user).
        .src_stage_mask(
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        )
        // Specifies the memory access type(s) in the source scope that need synchronization.
        // In this case there are no specific memory accesses that need synchronization in this dependency.
        .src_access_mask(vk::AccessFlags::empty())
        // Specifies the pipeline stage(s) in the destination scope that depend on the source.
        // Again, this is COLOR_ATTACHMENT_OUTPUT, meaning the rendering commands in the first
        // subpass that write to the color attachment depend on the completion of prior operations.
        // This ensures that the destination subpass starts writing to the color attachment
        // only after it's safe to do so.
        .dst_stage_mask(
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        )
        // Specifies the memory access type(s) required in the destination scope.
        // COLOR_ATTACHMENT_WRITE indicates that the subpass will write to the color attachment.
        // This ensures proper synchronization of memory for writing, so the render pass
        // doesn't overwrite data that's still being processed from prior operations.
        .dst_access_mask(
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        );

    let attachments = &[
        color_attachment,
        depth_stencil_attachment,
        color_resolve_attachment,
    ];
    let subpasses = &[subpass];
    let dependencies = &[dependency];

    let info = vk::RenderPassCreateInfo::builder()
        .attachments(attachments)
        .subpasses(subpasses)
        .dependencies(dependencies);

    Ok(unsafe { device.create_render_pass(&info, None) }?)
}
