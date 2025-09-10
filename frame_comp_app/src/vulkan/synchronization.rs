use anyhow::Result;
use vulkanalia::prelude::v1_3::*;

use crate::app::{AppData, MAX_FRAMES_IN_FLIGHT};

/// Semaphores
///
/// Semaphores are GPU-side synchronization objects.
/// They ensure ordering between GPU operations. Their primary use is to synchronize
/// operations between different queues or within the same queue without involving the CPU.
///
/// Semaphores are used to:
/// 1. Signal the completion of one set of GPU commands.
/// 2. Wait for another set of GPU commands to start only when
///    the previous commands are complete.
///
/// Key characteristics:
/// - Binary semaphore: Can only be signaled or unsignaled (on/off)
/// - Used for queue(s) synchronization
/// - Cannot be reset: Once signaled, they remain signaled until consumed by a waiting
///   operation. After consumption, they automatically reset to an unsignaled state.
/// - Example use case: Signaling the completion of rendering, or waiting for the
///   swapchain image to become available before starting rendering.
///
/// Used in queue submission (vkQueueSubmit): Pass semaphores to signal when GPU work is complete,
/// and presentation (vkQueuePresentKHR): Wait for a semaphore before presenting a swapchain
/// image to ensure rendering is complete.
///
///
/// Fences
///
/// Fences are CPU-side synchronization objects.
/// They enable GPU-to-GPU synchronization, allowing the CPU to wait for the GPU
/// to complete specific tasks.
///
/// Fences are used to:
/// 1. Signal the CPU when the GPU finishes executing a set of commands.
/// 2. Let the CPU wait until the GPU has finished a particular operation.
///
/// Unlike semaphores, fences allow the CPU to directly query or block until the GPU
/// work is complete.
///
/// Key characteristics:
/// - Signaled/Unsignaled state: The GPU signals the dence when its associated work is complete.
///   The CPU can wait for the fence to become signaled.
/// - Reset manually: Fences must be explicitly reset to an unsignaled state before reuse
///   using vkResetFences.
/// - Designed for CPU-to-GPU synchronization: The CPU uses fences to know when it can safely
///   procees (e.g., reuse resources).
/// - Example use case: Ensuring that the CPI waits for the FPU rendering to finish
///   before starting another frame.
///
/// Used in:
/// 1. Queue submission (vkQueueSubmit): associate a fence with GPu work. The fence
///    signals when the work is complete,
/// 2. Waiting (vkWaitForFences): The CPU waits for one or more fences to be signaled.
/// 3. Querying 9vkGetFenceStatus): Check if a fence is signaled without blocking.
///
///
/// Signaling: When a synchronization object transitions to the "signaled" state,
///     indicating that the associated work is complete.
///
pub fn create_sync_objects(device: &Device, data: &mut AppData) -> Result<()> {
    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

    unsafe {
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            data.image_available_semaphores
                .push(device.create_semaphore(&semaphore_info, None)?);
            data.render_finished_semaphores
                .push(device.create_semaphore(&semaphore_info, None)?);
            data.command_completion_fences
                .push(device.create_fence(&fence_info, None)?);
        }
    }

    data.image_usage_fences = vec![vk::Fence::null(); data.swapchain_images.len()];

    Ok(())
}
