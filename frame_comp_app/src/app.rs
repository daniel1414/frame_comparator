use std::ptr::copy_nonoverlapping as memcpy;
use std::u64;

use anyhow::{Result, anyhow};
use cgmath::{Deg, point3, vec3};
use std::time::Instant;
use vk::{KhrSurfaceExtension, KhrSwapchainExtension};
use vulkanalia::loader::{LIBRARY, LibloadingLoader};
use vulkanalia::prelude::v1_3::*;
use vulkanalia::vk::ExtDebugUtilsExtension;

use vulkanalia::Version;
use vulkanalia::window as vk_window;
use winit::window::Window;

use crate::vulkan::buffers::depth_buffer::create_depth_objects;
use crate::vulkan::buffers::index_buffer::create_index_buffer;
use crate::vulkan::buffers::uniform_buffer::{
    Mat4, UniformBufferObject, create_descriptor_pool, create_descriptor_set_layout,
    create_descriptor_sets, create_uniform_buffers,
};
use crate::vulkan::buffers::vertex_buffer::create_vertex_buffer;
use crate::vulkan::commands::{create_command_buffers, create_command_pool};
use crate::vulkan::device::create_logical_device;
use crate::vulkan::framebuffer::create_framebuffers;
use crate::vulkan::image::{
    create_color_objects, create_texture_image, create_texture_image_view, create_texture_sampler,
};
use crate::vulkan::instance::create_instance;
use crate::vulkan::model::load_model;
use crate::vulkan::physical_device::pick_physical_device;
use crate::vulkan::pipeline::create_pipeline;
use crate::vulkan::render_pass::create_render_pass;
use crate::vulkan::swapchain::{create_swapchain, create_swapchain_image_views};
use crate::vulkan::synchronization::create_sync_objects;
use crate::vulkan::vertex::Vertex;

pub const MAX_FRAMES_IN_FLIGHT: usize = 3;
pub const VALIDATION_ENABLED: bool = cfg!(debug_assertions);
pub const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");
pub const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);

/// The Vulkan App
#[derive(Clone, Debug)]
pub struct App {
    pub entry: Entry,
    pub instance: Instance,
    pub data: AppData,
    pub device: Device,
    pub frame: usize,
    pub resized: bool,
    pub start: Instant,
}

impl App {
    /// Creates our Vulkan app.
    pub fn create(window: &Window) -> Result<Self> {
        let loader = unsafe { LibloadingLoader::new(LIBRARY) }?;
        let entry = unsafe { Entry::new(loader).map_err(|b| anyhow!("{}", b)) }?;
        let mut data = AppData::default();
        let instance = create_instance(window, &entry, &mut data)?;
        data.surface = unsafe { vk_window::create_surface(&instance, &window, &window) }?;
        pick_physical_device(&instance, &mut data)?;
        let device = create_logical_device(&entry, &instance, &mut data)?;
        create_swapchain(window, &instance, &device, &mut data)?;
        create_swapchain_image_views(&device, &mut data)?;

        data.render_pass[0] = create_render_pass(&instance, &device, &mut data)?;
        data.render_pass[1] = create_render_pass(&instance, &device, &mut data)?;

        create_descriptor_set_layout(&device, &mut data)?;

        // One pipeline per render pass
        (data.left_pipeline_layout, data.left_pipeline) =
            create_pipeline(&device, &data, &data.render_pass[0])?;

        (data.right_pipeline_layout, data.right_pipeline) =
            create_pipeline(&device, &data, &data.render_pass[1])?;

        create_command_pool(&instance, &device, &mut data)?;
        create_color_objects(&instance, &device, &mut data)?;
        create_depth_objects(&instance, &device, &mut data)?;

        data.left_framebuffers = create_framebuffers(&device, &data, &data.render_pass[0])?;
        data.right_framebuffers = create_framebuffers(&device, &data, &data.render_pass[1])?;

        create_texture_image(&instance, &device, &mut data)?;
        create_texture_image_view(&device, &mut data)?;
        create_texture_sampler(&device, &mut data)?;
        load_model(&mut data)?;
        create_vertex_buffer(&instance, &device, &mut data)?;
        create_index_buffer(&instance, &device, &mut data)?;
        create_uniform_buffers(&instance, &device, &mut data)?;
        create_descriptor_pool(&device, &mut data)?;
        create_descriptor_sets(&device, &mut data)?;
        create_command_buffers(&device, &mut data)?;
        create_sync_objects(&device, &mut data)?;

        Ok(Self {
            entry,
            instance,
            data,
            device,
            frame: 0,
            resized: false,
            start: Instant::now(),
        })
    }

    pub fn recreate_swapchain(&mut self, window: &Window) -> Result<()> {
        // We shouldn't touch resources that may still be in use.
        unsafe {
            self.device.device_wait_idle()?;
            self.destroy_swapchain();
        }

        create_swapchain(window, &self.instance, &self.device, &mut self.data)?;
        create_swapchain_image_views(&self.device, &mut self.data)?;

        self.data.render_pass[0] =
            create_render_pass(&self.instance, &self.device, &mut self.data)?;
        self.data.render_pass[1] =
            create_render_pass(&self.instance, &self.device, &mut self.data)?;

        (self.data.left_pipeline_layout, self.data.left_pipeline) =
            create_pipeline(&self.device, &self.data, &self.data.render_pass[0])?;

        (self.data.right_pipeline_layout, self.data.right_pipeline) =
            create_pipeline(&self.device, &self.data, &self.data.render_pass[1])?;

        create_color_objects(&self.instance, &self.device, &mut self.data)?;
        create_depth_objects(&self.instance, &self.device, &mut self.data)?;

        self.data.left_framebuffers =
            create_framebuffers(&self.device, &self.data, &self.data.render_pass[0])?;

        self.data.right_framebuffers =
            create_framebuffers(&self.device, &self.data, &self.data.render_pass[1])?;

        create_uniform_buffers(&self.instance, &self.device, &mut self.data)?;
        create_descriptor_pool(&self.device, &mut self.data)?;
        create_descriptor_sets(&self.device, &mut self.data)?;
        create_command_buffers(&self.device, &mut self.data)?;
        self.data
            .image_usage_fences
            .resize(self.data.swapchain_images.len(), vk::Fence::null());
        Ok(())
    }

    /// Destroys our Vulkan app.
    pub fn destroy(&mut self) {
        unsafe {
            self.destroy_swapchain();

            self.device.destroy_sampler(self.data.texture_sampler, None);
            self.device
                .destroy_image_view(self.data.texture_image_view, None);
            self.device.destroy_image(self.data.texture_image, None);
            self.device
                .free_memory(self.data.texture_image_memory, None);
            self.device
                .destroy_descriptor_set_layout(self.data.descriptor_set_layout, None);
            self.device.destroy_buffer(self.data.vertex_buffer, None);
            self.device.free_memory(self.data.index_buffer_memory, None);
            self.device.destroy_buffer(self.data.index_buffer, None);
            self.device
                .free_memory(self.data.vertex_buffer_memory, None);
            self.data
                .command_completion_fences
                .iter()
                .for_each(|f| self.device.destroy_fence(*f, None));
            self.data
                .render_finished_semaphores
                .iter()
                .for_each(|s| self.device.destroy_semaphore(*s, None));
            self.data
                .image_available_semaphores
                .iter()
                .for_each(|s| self.device.destroy_semaphore(*s, None));

            self.device
                .destroy_command_pool(self.data.command_pool, None);
            self.device.destroy_device(None);
            if VALIDATION_ENABLED {
                self.instance
                    .destroy_debug_utils_messenger_ext(self.data.messenger, None);
            }
            self.instance.destroy_surface_khr(self.data.surface, None);
            self.instance.destroy_instance(None);
        }
    }

    fn destroy_swapchain(&mut self) {
        unsafe {
            self.device
                .destroy_image_view(self.data.color_image_view, None);
            self.device.free_memory(self.data.color_image_memory, None);
            self.device.destroy_image(self.data.color_image, None);
            self.device
                .destroy_image_view(self.data.depth_image_view, None);
            self.device.free_memory(self.data.depth_image_memory, None);
            self.device.destroy_image(self.data.depth_image, None);
            self.device
                .destroy_descriptor_pool(self.data.descriptor_pool, None);
            self.data
                .uniform_buffers
                .iter()
                .for_each(|b| self.device.destroy_buffer(*b, None));
            self.data
                .uniform_buffers_memory
                .iter()
                .for_each(|m| self.device.free_memory(*m, None));

            // Freeing the command buffers is not mandatory as they are freed automatically
            // when the command pool is destroyed.
            self.device
                .free_command_buffers(self.data.command_pool, &self.data.command_buffers);

            self.data
                .left_framebuffers
                .iter()
                .for_each(|f| self.device.destroy_framebuffer(*f, None));

            self.data
                .right_framebuffers
                .iter()
                .for_each(|f| self.device.destroy_framebuffer(*f, None));

            self.device.destroy_pipeline(self.data.left_pipeline, None);
            self.device.destroy_pipeline(self.data.right_pipeline, None);

            self.device
                .destroy_pipeline_layout(self.data.left_pipeline_layout, None);
            self.device
                .destroy_pipeline_layout(self.data.right_pipeline_layout, None);

            self.device
                .destroy_render_pass(self.data.render_pass[0], None);
            self.device
                .destroy_render_pass(self.data.render_pass[1], None);

            self.data
                .swapchain_image_views
                .iter()
                .for_each(|v| self.device.destroy_image_view(*v, None));

            self.device.destroy_swapchain_khr(self.data.swapchain, None);
        };
    }

    /// Renders a frame for our Vulkan app.
    pub fn render(&mut self, window: &Window) -> Result<()> {
        // Ensures that the GPU has finished executing the commands for the current frame
        // (rendering & presenting) before starting a new frame. This avoids overwriting
        // resources (like command buffers and semaphores) that are still in use.

        unsafe {
            self.device.wait_for_fences(
                &[self.data.command_completion_fences[self.frame]],
                true,
                u64::MAX,
            )
        }?;

        // This semaphore ensures synchronization between the swapchain and the rendering process.
        let this_frame_image_available_semaphore = self.data.image_available_semaphores[self.frame];

        // More like a request to aquire an image - we get the index instantly, but this
        // doesn't mean the image is ready to be processed. It will be once the semaphore
        // is signaled, only then can we perform operations on the image itself.
        let result = unsafe {
            self.device.acquire_next_image_khr(
                self.data.swapchain,
                u64::MAX,
                // When the GPU is finished aquiring the image, it will signal this
                // semahpore, which is used by the command buffer to wait until this
                // image is ready for rendering.
                this_frame_image_available_semaphore,
                vk::Fence::null(),
            )
        };

        let image_index = match result {
            Ok((image_index, _)) => image_index as usize,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => {
                dbg!("Swapchain out of date!");
                return self.recreate_swapchain(window);
            }
            Err(e) => return Err(anyhow!(e)),
        };

        // If a fence exists and hasn't been signaled for this image, means the GPU
        // is still processing it.
        if !self.data.image_usage_fences[image_index as usize].is_null() {
            // So we need to wait for the GPU to finish its operations on this image before proceeding.
            unsafe {
                self.device.wait_for_fences(
                    &[self.data.image_usage_fences[image_index as usize]],
                    true,
                    u64::MAX,
                )
            }?;
        }

        // Associates the fence for the current frame with the swapchain image
        // to track its usage.
        self.data.image_usage_fences[image_index as usize] =
            self.data.command_completion_fences[self.frame];

        self.update_uniform_buffer(image_index)?;

        let wait_semaphores = &[this_frame_image_available_semaphore];

        // The pipeline waits at the COLOR_ATTACHMENT_OUTPUT stage, which is where rendering
        // to the swapchain image occurs.
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.data.command_buffers[image_index]];
        let signal_semaphores = &[self.data.render_finished_semaphores[self.frame]];
        let submit_info = vk::SubmitInfo::builder()
            // The GPU will wait with processing this command buffer until this semaphore is
            // signaled and it is signaled when the GPU is finished aquiring the image
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            // The GPU will signal this semaphore when the command buffer is done executing,
            // which means the image will be fully rendered to. We need this semaphore to be
            // signaled in order to make the GPU wait for it, before presenting this image.
            .signal_semaphores(signal_semaphores);

        // This step ensures the fence associated with the current frame is ready for the next submission.
        // Resetting is mandatory for the fence to unsignal it so it can be signaled again
        // by the GPU when commands are finished executing in the queue.
        unsafe {
            self.device
                .reset_fences(&[self.data.command_completion_fences[self.frame]])
        }?;

        // Submission involved associating the command buffer with synchronization primitives
        // (semaphores and fences) to coordinate execution.
        unsafe {
            self.device.queue_submit(
                self.data.graphics_queue,
                &[submit_info],
                // The fence is signaled by the GPU when all commands in the submitted command buffer
                // have been fully executed by the graphics queue.
                self.data.command_completion_fences[self.frame],
            )
        }?;

        let swapchains = &[self.data.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            // The GPU will wait until this sepahore is signaled and it will be signaled when
            // the command buffer above (rendering to the image) will be finished.
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        let result = unsafe {
            self.device
                .queue_present_khr(self.data.present_queue, &present_info)
        };

        let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR)
            || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);

        if self.resized || changed {
            self.resized = false;
            self.recreate_swapchain(window)?;
        } else if let Err(e) = result {
            return Err(anyhow!(e));
        }

        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }

    fn update_uniform_buffer(&self, image_index: usize) -> Result<()> {
        let time = self.start.elapsed().as_secs_f32();

        let model = Mat4::from_axis_angle(
            vec3(0.0, 0.0, 1.0),
            Deg(45.0), // * time
        );

        let view = Mat4::look_at_rh(
            point3(0.0, 2.5, 2.5),
            point3(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, 1.0),
        );

        // Mat4::new constructs the matrix in a column-major order, so the matrix look like
        // [1,  0, 0  , 0  ]
        // [0, -1, 0  , 0  ]
        // [0,  0, 0.5, 0.5]
        // [0,  0, 0  , 1  ]
        let correction = Mat4::new(
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            -1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0 / 2.0,
            0.0,
            0.0,
            0.0,
            1.0 / 2.0,
            1.0,
        );

        // cgmath was originally designed for OpenGL, where the Y coordinate of the clip coordinates
        // is inverted. This is the easiest way to compensate it.
        let proj = correction
            * cgmath::perspective(
                Deg(45.0),
                self.data.swapchain_extent.width as f32 / self.data.swapchain_extent.height as f32,
                0.1,
                10.0,
            );

        // Passing in individual matrices to the GPU and multiplying them in the vertex shader
        // offloads work to the GPU, but is not recommended for low-poly meshes.
        // For static meshes (that don't change location) the MVP should be pre-calculated
        // on the CPU to save GPU overhead. Multiplication in the vertex shader is recommended for
        // dynamic scenes, high-poly meshes, CPU-bound applications, per-vertex transformations.
        // There is also the hybrid approach: Calculate the VP one the CPU and MVP = VP * model
        // in the vertex shader. This reduces data transfer while retaining some GPU flexibility.
        //let ubo = UniformBufferObject {
        //    model, view, proj
        //};

        let ubo = UniformBufferObject { model, view, proj };

        unsafe {
            let memory = self.device.map_memory(
                self.data.uniform_buffers_memory[image_index],
                0,
                size_of::<UniformBufferObject>() as u64,
                vk::MemoryMapFlags::empty(),
            )?;

            memcpy(&ubo, memory.cast(), 1);

            self.device
                .unmap_memory(self.data.uniform_buffers_memory[image_index]);
        }

        Ok(())
    }
}

/// The Vulkan handles and associated properties used by our Vulkan app.
#[derive(Clone, Debug, Default)]
pub struct AppData {
    pub messenger: vk::DebugUtilsMessengerEXT,
    pub physical_device: vk::PhysicalDevice,
    pub msaa_samples: vk::SampleCountFlags,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub surface: vk::SurfaceKHR,
    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,
    // 0 for the left side, 1 for the right side
    pub render_pass: [vk::RenderPass; 2],

    /// The layout of the descriptor set for the UBO that holds the MVP matrix.
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub left_pipeline_layout: vk::PipelineLayout,
    pub right_pipeline_layout: vk::PipelineLayout,
    pub left_pipeline: vk::Pipeline,
    pub right_pipeline: vk::Pipeline,
    pub left_framebuffers: Vec<vk::Framebuffer>,
    pub right_framebuffers: Vec<vk::Framebuffer>,
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,

    /// These semaphores corespond to swapchain images and are signaled
    /// when the GPU has finished aquiring an image from the swapchain.
    /// Used to synchronize rendering operations with image availability.
    pub image_available_semaphores: Vec<vk::Semaphore>,

    /// These semaphores are signaled when the GPU has finished rendering
    /// to a swapchain image. They synchronize rendering operations with the
    /// presentation engine, ensuring the image is ready to be presented.
    pub render_finished_semaphores: Vec<vk::Semaphore>,

    /// Signaled by the GPU when all commands for the current frame have
    /// finished executing. Ensures that the CPU does not overwrite
    /// or reuse resources still in use by the GPU.
    pub command_completion_fences: Vec<vk::Fence>,

    /// Fences associated with swapchain images currently in use by the GPU.
    /// Ensures that a swapchain image is not overwritten or reused
    /// while it is still being processed.
    pub image_usage_fences: Vec<vk::Fence>,

    pub vertices: Vec<Vertex>,
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,

    pub indices: Vec<u32>,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,

    /// One uniform buffer per swapchain image as we will have a different MVP matrix
    /// in every frame and we don't want to modify a buffer that is in use by the
    /// previous frame.
    pub uniform_buffers: Vec<vk::Buffer>,
    pub uniform_buffers_memory: Vec<vk::DeviceMemory>,

    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,

    /// Resources for textures
    pub mip_levels: u32,
    pub texture_image: vk::Image,
    pub texture_image_memory: vk::DeviceMemory,
    pub texture_image_view: vk::ImageView,
    pub texture_sampler: vk::Sampler,

    /// Resources for the depth buffer
    pub depth_image: vk::Image,
    pub depth_image_memory: vk::DeviceMemory,
    pub depth_image_view: vk::ImageView,

    // Resources for multisampling
    pub color_image: vk::Image,
    pub color_image_memory: vk::DeviceMemory,
    pub color_image_view: vk::ImageView,
}
