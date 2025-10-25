use std::rc::Rc;
use vulkanalia::prelude::v1_3::*;

use anyhow::Result;
use rtcmp::{RenderTargetComparator, RenderTargetComparatorCreateInfo};

use crate::app::AppData;

pub fn create_comparators(
    device: &Rc<Device>,
    data: &AppData,
) -> Result<Option<Vec<RenderTargetComparator>>> {
    /* let viewport = vk::Viewport::builder()
    .x((data.swapchain_extent.width / 10 * 7) as f32)
    .y((data.swapchain_extent.height / 10) as f32)
    .width((data.swapchain_extent.width / 10 * 2) as f32)
    .height((data.swapchain_extent.height / 10 * 2) as f32)
    .max_depth(1.0)
    .build(); */
    let viewport = vk::Viewport::builder()
        .width(data.swapchain_extent.width as f32)
        .height(data.swapchain_extent.height as f32)
        .max_depth(1.0)
        .build();

    let comparators = data
        .swapchain_image_views
        .iter()
        .map(|i| {
            let info = RenderTargetComparatorCreateInfo::builder()
                .device(Rc::clone(device))
                .descriptor_pool(data.descriptor_pool)
                .format(data.swapchain_format)
                .extent(data.swapchain_extent)
                .in_image_views([data.resolve_image_view, data.color_image_view[1]])
                .viewport(viewport)
                .out_image_view(*i)
                .build()?;

            RenderTargetComparator::new(&info)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Some(comparators))
}
