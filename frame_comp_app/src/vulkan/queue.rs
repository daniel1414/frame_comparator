use super::errors;
use crate::app::AppData;
use anyhow::{Result, anyhow};
use vk::KhrSurfaceExtension;
use vulkanalia::prelude::v1_3::*;

#[derive(Copy, Clone, Debug)]
pub struct QueueFamilyIndices {
    pub graphics: u32,
    pub present: u32,
}

impl QueueFamilyIndices {
    pub fn get(
        instance: &Instance,
        data: &AppData,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Self> {
        let mut present = None;

        let properties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let graphics = properties
            .iter()
            .position(|p| p.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|i| i as u32);

        for (index, properties) in properties.iter().enumerate() {
            if (unsafe {
                instance.get_physical_device_surface_support_khr(
                    physical_device,
                    index as u32,
                    data.surface,
                )
            })? {
                present = Some(index as u32);
                break;
            }
        }

        if let (Some(graphics), Some(present)) = (graphics, present) {
            Ok(Self { graphics, present })
        } else {
            Err(anyhow!(errors::SuitabilityError(
                "Mssing required queue families."
            )))
        }
    }
}
