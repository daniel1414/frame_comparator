use anyhow::Result;
use vulkanalia::bytecode::Bytecode;
use vulkanalia::prelude::v1_3::*;

use crate::app::AppData;

use super::vertex::Vertex;

/// The graphics pipeline in Vulkan is a sequence of steps that the GPU follows to
/// transform input data (like vertex positions) into a rendered image on a framebuffer.
/// It consists of several fixed-function stages (like input assembly and rasterization)
/// and programmable stages (vertex and fragment shaders).
///
/// The pipeline specifies how to render to the render pass' attachments, including:
/// 1. How to write to the color attachment (via fragment shaders)
/// 2. Whether to use depth/stencil attachments for testing.
///
/// The pipeline is linked to a specific subpass of the render pass.
/// Multiple pipelines can be used in different subpasses, each with its own configuration.
///
/// Must output data to the color attachment in the right format (same as in the render pass
/// and swapchain).
pub fn create_pipeline(device: &Device, data: &mut AppData) -> Result<()> {
    let vert = include_bytes!("shaders/vert.spv");
    let frag = include_bytes!("shaders/frag.spv");

    let vert_module = create_shader_module(device, vert)?;
    let frag_module = create_shader_module(device, frag)?;

    let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_module)
        .name(b"main\0");

    let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_module)
        .name(b"main\0");

    let binding_descriptions = &[Vertex::binding_description()];
    let attribute_descriptions = Vertex::attribute_descriptions();
    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(binding_descriptions)
        .vertex_attribute_descriptions(&attribute_descriptions);

    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);

    // Area of the framebuffer to render to. In our case the whole area.
    let viewport = vk::Viewport::builder()
        .x(0.0)
        .y(0.0)
        .width(data.swapchain_extent.width as f32)
        .height(data.swapchain_extent.height as f32)
        .min_depth(0.0)
        .max_depth(1.0);

    // Area of the framebuffer that fragments are allowed to affect. In our case the whole area.
    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D { x: 0, y: 0 })
        .extent(vk::Extent2D {
            width: data.swapchain_extent.width / 2,
            height: data.swapchain_extent.height / 2,
        });

    let viewports = &[viewport];
    let scissors = &[scissor];

    let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewports(viewports)
        .scissors(scissors);

    // The rasterization state divides polygons into fragments (which end up being pixels on the screen)
    // and performs fragment culling - removing fragments that don't make it into the view.
    let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(false);

    let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(true)
        .min_sample_shading(0.2)
        .rasterization_samples(data.msaa_samples);

    let attachment = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::all())
        .blend_enable(false)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD);

    let attachments = &[attachment];

    // Blending new fragments with the existing ones in the framebuffer.
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0]);

    // The pipeline layout is like a blueprint that defines:
    // 1. Descriptor sets: How resources like textures and uniform buffers are accessed
    //    by the shaders.
    // 2. Push constants: Small amounts of data sent to shaders for per-draw customization.
    let set_layouts = &[data.descriptor_set_layout];
    let layout_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(set_layouts);

    data.pipeline_layout = unsafe { device.create_pipeline_layout(&layout_info, None) }?;

    let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
        // Specifies if the depth of new fragments should be compared to the depth buffer
        // to see if they should be discarded.
        .depth_test_enable(true)
        // Specifies if the new depth of fragments that pass the depth test should actually
        // be written to the depth buffer.
        .depth_write_enable(true)
        // Comparison that is performed to keep or discard fragments. For us lower depth = closer,
        // So the depth of new fragments should be less.
        .depth_compare_op(vk::CompareOp::LESS)
        // These three parameters are used for the optional depth bound test. This allows to
        // only keep fragments that fall within the specified depth range. This is optional.
        //.depth_bounds_test_enable(true)
        //.min_depth_bounds(0.0)
        //.max_depth_bounds(1.0)
        .depth_bounds_test_enable(false)
        // We will not be using stencil operations here.
        .stencil_test_enable(false);

    let stages = &[vert_stage, frag_stage];
    let info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(stages)
        .vertex_input_state(&vertex_input_state)
        .input_assembly_state(&input_assembly_state)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .multisample_state(&multisample_state)
        .color_blend_state(&color_blend_state)
        .layout(data.pipeline_layout)
        .depth_stencil_state(&depth_stencil_state)
        // Link this pipeline to the correct render pass.
        .render_pass(data.render_pass)
        // And the right subpass.
        .subpass(0);

    unsafe {
        data.pipeline = device
            .create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)?
            .0[0];

        device.destroy_shader_module(vert_module, None);
        device.destroy_shader_module(frag_module, None);
    }

    Ok(())
}

fn create_shader_module(device: &Device, bytecode: &[u8]) -> Result<vk::ShaderModule> {
    let bytecode = Bytecode::new(bytecode).unwrap();
    let info = vk::ShaderModuleCreateInfo::builder()
        .code_size(bytecode.code_size())
        .code(bytecode.code());

    let module = unsafe { device.create_shader_module(&info, None) }?;
    Ok(module)
}
