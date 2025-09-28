use anyhow::Result;
use frame_comp::{FrameComparator, FrameComparatorCreateInfo};
use vulkanalia::prelude::v1_3::*;

use crate::{app::AppData, vulkan::buffers::depth_buffer::get_depth_format};

pub fn create_comparator<'a, 'b>(
    instance: &'a Instance,
    device: &'b Device,
    data: &AppData,
) -> Result<FrameComparator<'a, 'b>> {
    let info = FrameComparatorCreateInfo::new(
        &instance,
        &device,
        vk::Rect2D::default(),
        data.swapchain_format,
        get_depth_format(&instance, &data)?,
        data.msaa_samples,
    );
    FrameComparator::new(info)
}
