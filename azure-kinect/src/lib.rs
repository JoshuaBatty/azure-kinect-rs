#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod calibration;
pub mod capture;
pub mod default;
pub mod device;
pub mod display;
pub mod error;
pub mod api;
pub mod format;
pub mod image;
pub mod k4a_functions;
pub mod playback;
pub mod playback_data_block;
pub mod playback_track;
pub mod record;
pub mod transformation;
pub mod utility;

pub mod bindings {
    include!("bindings.rs");
}

pub use calibration::Calibration;
pub use capture::Capture;
pub use device::Device;
pub use error::Error;
pub use api::{DebugMessageHandler, Api, ApiRecord};
pub use format::{Dimension, Range};
pub use image::Image;
pub use transformation::Transformation;

pub use bindings::{
    k4a_buffer_result_t, k4a_calibration_camera_t, k4a_calibration_extrinsics_t,
    k4a_calibration_intrinsics_t, k4a_calibration_t, k4a_calibration_type_t, k4a_capture_t,
    k4a_color_control_command_t, k4a_color_control_mode_t, k4a_color_resolution_t,
    k4a_depth_mode_t, k4a_device_configuration_t, k4a_device_t, k4a_firmware_build_t, k4a_float2_t,
    k4a_float3_t, k4a_fps_t, k4a_hardware_version_t, k4a_image_format_t, k4a_image_t,
    k4a_imu_sample_t, k4a_log_level_t, k4a_memory_destroy_cb_t, k4a_playback_data_block_t,
    k4a_playback_seek_origin_t, k4a_playback_t, k4a_record_configuration_t,
    k4a_record_subtitle_settings_t, k4a_record_t, k4a_record_video_settings_t, k4a_result_t,
    k4a_stream_result_t, k4a_transformation_interpolation_type_t, k4a_transformation_t,
    k4a_wait_result_t, k4a_wired_sync_mode_t, size_t, K4A_DEVICE_DEFAULT,
    K4A_VERSION_BUILD_METADATA, K4A_VERSION_MAJOR, K4A_VERSION_MINOR, K4A_VERSION_PATCH,
    K4A_VERSION_PRERELEASE, K4A_VERSION_STR, K4A_WAIT_INFINITE,
};
