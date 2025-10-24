use std::rc::Rc;
use vulkanalia::prelude::v1_3::*;

use anyhow::Result;
use frame_comp::{FrameComparator, FrameComparatorCreateInfo};

use crate::app::AppData;

pub fn create_comparators(
    device: &Rc<Device>,
    data: &AppData,
) -> Result<Option<Vec<FrameComparator>>> {
    let comparators = data
        .swapchain_image_views
        .iter()
        .map(|i| {
            let info = FrameComparatorCreateInfo::builder()
                .device(Rc::clone(device))
                .descriptor_pool(data.descriptor_pool)
                .format(data.swapchain_format)
                .extent(data.swapchain_extent)
                .in_image_views([data.resolve_image_view, data.color_image_view[1]])
                .out_image_view(*i)
                .build()?;

            FrameComparator::new(&info)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Some(comparators))
}
