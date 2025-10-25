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
    // === ATTACHMENTS ===

    // Attachment 0: Multisampled color attachment for scene rendering.
    let color_attachment = vk::AttachmentDescription2::builder()
        // Format of the color attachment should be same as the swapchain images.
        .format(data.swapchain_format)
        // For multisampling (anti-aliasing)
        .samples(data.msaa_samples)
        // Defines what happens to the attachment at the start of rendering
        .load_op(vk::AttachmentLoadOp::CLEAR)
        // What happens to the attachment after rendering - we resolve it, so we don't care
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        // Expected layout of the attachment before rendering.
        .initial_layout(vk::ImageLayout::UNDEFINED)
        // Defines what the final layout of the attachment should be after rendering.
        .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build();

    // Attachment 1: Multisampled depth (stencil) attachment for scene rendering.
    let depth_stencil_attachment = vk::AttachmentDescription2::builder()
        .format(get_depth_format(instance, data)?)
        .samples(data.msaa_samples)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        // We don't care about the depth data as it won't be used after drawing
        // has finished. Contrary to the color attachment, which is used to
        // present images to the screen. This may allow the hardware to perform
        // additional optimizations.
        // Edit: We do want to sample from the depth data in one of the render passes.
        // Edit: Edit: we resolve it, so we don't care.
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        // After we're done with it, we want to sample from this attachment in subpass 1
        .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
        .build();

    // Attachment 2: Resolved color attachment with the rendered scene (single-sampled).
    let color_resolve_attachment = vk::AttachmentDescription2::builder()
        .format(data.swapchain_format)
        .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::DONT_CARE)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .build();

    // Attachment 3: Color attachment for grayscale depth output (single-sampled)
    let grayscale_color_attachment = vk::AttachmentDescription2::builder()
        .format(data.swapchain_format)
        .load_op(vk::AttachmentLoadOp::DONT_CARE)
        .store_op(vk::AttachmentStoreOp::STORE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .samples(vk::SampleCountFlags::_1)
        .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .samples(vk::SampleCountFlags::_1)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .build();

    // Attachment 4: Resolved depth attachment (single-sampled)
    let depth_resolve_attachment = vk::AttachmentDescription2::builder()
        .format(get_depth_format(instance, data)?)
        .samples(vk::SampleCountFlags::_1)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .load_op(vk::AttachmentLoadOp::DONT_CARE)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .build();

    // === Subpass 0: Scene rendering ===

    let color_attachment_ref0 = vk::AttachmentReference2::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build();

    let depth_stencil_attachment_ref0 = vk::AttachmentReference2::builder()
        .attachment(1)
        .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
        .build();

    let color_resolve_attachment_ref0 = vk::AttachmentReference2::builder()
        .attachment(2)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build();

    let depth_resolve_attachment_ref = vk::AttachmentReference2::builder()
        .attachment(3)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build();

    let mut depth_stencil_resolve = vk::SubpassDescriptionDepthStencilResolve::builder()
        .depth_resolve_mode(vk::ResolveModeFlags::SAMPLE_ZERO)
        .stencil_resolve_mode(vk::ResolveModeFlags::NONE)
        .depth_stencil_resolve_attachment(&depth_resolve_attachment_ref)
        .build();

    let color_attachments0 = [color_attachment_ref0];
    let color_resolve_attachments0 = [color_resolve_attachment_ref0];

    let subpass0 = vk::SubpassDescription2::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&color_attachments0)
        .depth_stencil_attachment(&depth_stencil_attachment_ref0)
        .resolve_attachments(&color_resolve_attachments0)
        .push_next(&mut depth_stencil_resolve)
        .build();

    // === Subpass 1: Depth Visualization ===

    let color_attachment_ref1 = vk::AttachmentReference2::builder()
        .attachment(4)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build();

    let depth_input_attachment_ref1 = vk::AttachmentReference2::builder()
        .attachment(3)
        .layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .aspect_mask(vk::ImageAspectFlags::DEPTH)
        .build();

    let color_attachments1 = [color_attachment_ref1];
    let input_attachments1 = [depth_input_attachment_ref1];

    let subpass1 = vk::SubpassDescription2::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&color_attachments1)
        .input_attachments(&input_attachments1)
        .build();

    // === DEPENDENCIES ===

    let dependency0 = vk::SubpassDependency2::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        )
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        )
        .dst_access_mask(
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        )
        .build();

    let dependency1 = vk::SubpassDependency2::builder()
        .src_subpass(0)
        .dst_subpass(1)
        .src_stage_mask(vk::PipelineStageFlags::LATE_FRAGMENT_TESTS)
        .src_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
        .dst_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
        .dst_access_mask(vk::AccessFlags::SHADER_READ)
        .build();

    let attachments = &[
        color_attachment,
        depth_stencil_attachment,
        color_resolve_attachment,
        depth_resolve_attachment,
        grayscale_color_attachment,
    ];

    let subpasses = &[subpass0, subpass1];
    let dependencies = &[dependency0, dependency1];

    let info = vk::RenderPassCreateInfo2::builder()
        .attachments(attachments)
        .subpasses(subpasses)
        .dependencies(dependencies)
        .build();

    Ok(unsafe { device.create_render_pass2(&info, None) }?)
}
