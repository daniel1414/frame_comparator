use crate::app::AppData;
use anyhow::Result;
use vulkanalia::prelude::v1_3::*;
use vulkanalia::vk::KhrSwapchainExtensionDeviceCommands;
use vulkanalia::{vk, vk::KhrSurfaceExtensionInstanceCommands};
use winit::window::Window;

use super::image::create_image_view;
use super::queue::QueueFamilyIndices;

#[derive(Clone, Debug)]
pub struct SwapchainSupport {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupport {
    pub fn get(
        instance: &Instance,
        data: &AppData,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Self> {
        Ok(unsafe {
            Self {
                capabilities: instance
                    .get_physical_device_surface_capabilities_khr(physical_device, data.surface)?,
                formats: instance
                    .get_physical_device_surface_formats_khr(physical_device, data.surface)?,
                present_modes: instance
                    .get_physical_device_surface_present_modes_khr(physical_device, data.surface)?,
            }
        })
    }
}

/// Creates a vulkan swapchain that holds images that are displayed in the window
///
/// A swapchain is a collection of images that are used for presenting rendered content
/// to the screen. It manages the back-and-forth process of rendering images and
/// displaying them, often referred to as buffer swapping.
///
/// 1. Render content to images stored in GPU memory.
/// 2. Vulkan handles the process of displaying the images to the user.
///
/// Buffering allows rendering and presentation to happen asynchronously.
/// Ensures that rendering and presentation do not interfere with each other.
/// Links Vulkan with the OS's windowing system (X11, Win32 etc.).
///
/// The front buffer is the image currently displayed on the screen.
/// The back buffer(s) are the images where the rendering takes place.
/// Once rendering is complete, a back buffer is swapped with the front buffer for display.
///
/// Tripple buffering - Two images are used for rendering while one is displayed,
/// reducing latency and increasing smoothness.
///
/// The presentation mode defines how images are swapped between the front and back buffers.
///
pub fn create_swapchain(
    window: &Window,
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;
    let support = SwapchainSupport::get(instance, data, data.physical_device)?;

    let surface_format = get_swapchain_surface_format(&support.formats);
    let present_mode = get_swapchain_present_mode(&support.present_modes);
    let extent = get_swapchain_extent(window, support.capabilities);

    let mut image_count = support.capabilities.min_image_count + 1;
    if support.capabilities.max_image_count != 0
        && image_count > support.capabilities.max_image_count
    {
        image_count = support.capabilities.max_image_count;
    }

    let mut queue_family_indices = vec![];
    let image_sharing_mode = if indices.graphics != indices.present {
        queue_family_indices.push(indices.graphics);
        queue_family_indices.push(indices.present);
        vk::SharingMode::CONCURRENT
    } else {
        vk::SharingMode::EXCLUSIVE
    };

    let info = vk::SwapchainCreateInfoKHR::builder()
        .surface(data.surface)
        .min_image_count(image_count)
        .image_format(surface_format.format)
        .image_color_space(surface_format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(image_sharing_mode)
        .queue_family_indices(&queue_family_indices)
        .pre_transform(support.capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true)
        .old_swapchain(vk::SwapchainKHR::null());

    unsafe {
        data.swapchain = device.create_swapchain_khr(&info, None)?;
        data.swapchain_images = device.get_swapchain_images_khr(data.swapchain)?;
    }
    data.swapchain_format = surface_format.format;
    data.swapchain_extent = extent;

    Ok(())
}

pub fn get_swapchain_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
    formats
        .iter()
        .cloned()
        .find(|f| {
            f.format == vk::Format::B8G8R8A8_SRGB
                && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or_else(|| formats[0])
}

pub fn get_swapchain_present_mode(present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
    present_modes
        .iter()
        .cloned()
        .find(|m| *m == vk::PresentModeKHR::MAILBOX)
        .unwrap_or(vk::PresentModeKHR::FIFO)
}

pub fn get_swapchain_extent(
    window: &Window,
    capabilities: vk::SurfaceCapabilitiesKHR,
) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        capabilities.current_extent
    } else {
        vk::Extent2D::builder()
            .width(window.inner_size().width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ))
            .height(window.inner_size().height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ))
            .build()
    }
}

/// Swapchain images are stored in GPU memory and are often allocated in a format
/// optimized for the display hardware. An image view is an object that describes how
/// to access a specigic portion of a Vulkan image. This separates the raw data (image)
/// from the way it is interpreted and accessed (view).
///
/// Vulkan images can be multi-layered (array textures or cubemaps),
/// multi-leveled (mipmaps),
/// multi-aspected (color, depth, stencil).
/// The image view specifies which portion is accessible and in what format.
///
/// Images have specific layouts for different stages:
/// VK_IMAGE_LAYOUT_PRESENT_SRC_KHR: Used when presenting the image.
/// VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL: Used when rendering to the image.
///
/// For rendering, the image view is bound to the framebuffer.
pub fn create_swapchain_image_views(device: &Device, data: &mut AppData) -> Result<()> {
    data.swapchain_image_views = data
        .swapchain_images
        .iter()
        .map(|i| {
            create_image_view(
                device,
                *i,
                data.swapchain_format,
                vk::ImageAspectFlags::COLOR,
                1,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(())
}
