use std::rc::Rc;
use vulkanalia::prelude::v1_3::*;

use anyhow::Result;
use frame_comp::FrameComparator;

use crate::app::AppData;

pub fn create_comparators(device: &Device, data: &AppData) -> Result<Option<Vec<FrameComparator>>> {
    let comparators = data
        .swapchain_image_views
        .iter()
        .map(|i| {
            FrameComparator::new(
                unsafe { Rc::from_raw(device) },
                data.descriptor_pool,
                data.swapchain_format,
                data.swapchain_extent,
                None,
                data.resolve_image_view,
                *i,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Some(comparators))
}
