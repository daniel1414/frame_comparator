use log::{debug, error, trace, warn};
use std::os::raw::c_void;
use std::{collections::HashMap, ffi::CStr, sync::Mutex};
use vulkanalia::prelude::v1_3::*;

// Tracks how many times each message has been printed
lazy_static::lazy_static! {
    static ref MESSAGE_COUNT: Mutex<HashMap<String, u32>> = Mutex::new(HashMap::new());
}

pub extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    type_: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    let data = unsafe { *data };
    let message = unsafe { CStr::from_ptr(data.message) }
        .to_string_lossy()
        .to_string();

    // Increment the counter for this message
    let mut counts = MESSAGE_COUNT.lock().unwrap();
    let entry = counts.entry(message.clone()).or_insert(0);

    if *entry < 5 {
        *entry += 1;

        if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
            error!("({:?}) {}", type_, message);
        } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
            warn!("({:?}) {}", type_, message);
        } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
            debug!("({:?}) {}", type_, message);
        } else {
            trace!("({:?}) {}", type_, message);
        }
    }

    // Don't abort the Vulkan call
    vk::FALSE
}
