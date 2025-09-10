use std::collections::HashSet;

use super::errors::SuitabilityError;
use super::queue::QueueFamilyIndices;
use super::swapchain::SwapchainSupport;
use crate::app::AppData;
use anyhow::{Result, anyhow};
use log::*;
use vulkanalia::prelude::v1_3::*;

pub const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];

pub fn pick_physical_device(instance: &Instance, data: &mut AppData) -> Result<()> {
    unsafe {
        for physical_device in instance.enumerate_physical_devices()? {
            let properties = instance.get_physical_device_properties(physical_device);

            if let Err(error) = check_physical_device(instance, data, physical_device) {
                warn!(
                    "Skipping physical device ('{}'): {}",
                    properties.device_name, error
                );
            } else {
                info!("Selected physical device  ('{}').", properties.device_name);
                data.physical_device = physical_device;
                data.msaa_samples = get_max_msaa_samples(instance, data);
                return Ok(());
            }
        }
    }

    Err(anyhow!("Failed to find a suitable physical device."))
}

pub fn check_physical_device(
    instance: &Instance,
    data: &AppData,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    // Check if the physical device contains the needed queue indices (for graphics and presentation queues)
    QueueFamilyIndices::get(instance, data, physical_device)?;

    // Check if the physical device supports swapchains
    let support = SwapchainSupport::get(instance, data, physical_device)?;
    if support.formats.is_empty() || support.present_modes.is_empty() {
        return Err(anyhow!(SuitabilityError("Insufficient swapchain support.")));
    }

    // Check if the physical device has the required extensions
    check_physical_device_extensions(instance, physical_device)?;

    let features = unsafe { instance.get_physical_device_features(physical_device) };
    if features.sampler_anisotropy != vk::TRUE {
        return Err(anyhow!(SuitabilityError("No sampler anisotrophy.")));
    }

    Ok(())
}

pub fn check_physical_device_extensions(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    let extensions =
        unsafe { instance.enumerate_device_extension_properties(physical_device, None) }?
            .iter()
            .map(|e| e.extension_name)
            .collect::<HashSet<_>>();

    if DEVICE_EXTENSIONS.iter().all(|e| extensions.contains(e)) {
        Ok(())
    } else {
        Err(anyhow!(SuitabilityError(
            "Missing required device extensions!"
        )))
    }
}

pub fn get_max_msaa_samples(instance: &Instance, data: &AppData) -> vk::SampleCountFlags {
    let properties = unsafe { instance.get_physical_device_properties(data.physical_device) };
    let counts = properties.limits.framebuffer_color_sample_counts
        & properties.limits.framebuffer_depth_sample_counts;

    [
        vk::SampleCountFlags::_64,
        vk::SampleCountFlags::_32,
        vk::SampleCountFlags::_16,
        vk::SampleCountFlags::_8,
        vk::SampleCountFlags::_4,
        vk::SampleCountFlags::_2,
    ]
    .into_iter()
    .find(|c| counts.contains(*c))
    .unwrap_or(vk::SampleCountFlags::_1)
}
