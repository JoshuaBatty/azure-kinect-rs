use super::bindings::*;

pub(crate) type k4abt_tracker_create = fn(
    sensor_calibration: *const k4a_calibration_t,
    config: k4abt_tracker_configuration_t,
    tracker_handle: *mut k4abt_tracker_t,
) -> k4a_result_t;

pub(crate) type k4abt_tracker_destroy = fn(tracker_handle: k4abt_tracker_t);

pub(crate) type k4abt_tracker_set_temporal_smoothing =
    fn(tracker_handle: k4abt_tracker_t, smoothing_factor: f32);

pub(crate) type k4abt_tracker_enqueue_capture = fn(
    tracker_handle: k4abt_tracker_t,
    sensor_capture_handle: k4a_capture_t,
    timeout_in_ms: i32,
) -> k4a_wait_result_t;

pub(crate) type k4abt_tracker_pop_result = fn(
    tracker_handle: k4abt_tracker_t,
    body_frame_handle: *mut k4abt_frame_t,
    timeout_in_ms: i32,
) -> k4a_wait_result_t;

pub(crate) type k4abt_tracker_shutdown = fn(tracker_handle: k4abt_tracker_t);

pub(crate) type k4abt_frame_release = fn(body_frame_handle: k4abt_frame_t);

pub(crate) type k4abt_frame_reference = fn(body_frame_handle: k4abt_frame_t);

pub(crate) type k4abt_frame_get_num_bodies = fn(body_frame_handle: k4abt_frame_t) -> u32;

pub(crate) type k4abt_frame_get_body_skeleton = fn(
    body_frame_handle: k4abt_frame_t,
    index: u32,
    skeleton: *mut k4abt_skeleton_t,
) -> k4a_result_t;

pub(crate) type k4abt_frame_get_body_id = fn(body_frame_handle: k4abt_frame_t, index: u32) -> u32;

pub(crate) type k4abt_frame_get_device_timestamp_usec = fn(body_frame_handle: k4abt_frame_t) -> u64;

pub(crate) type k4abt_frame_get_body_index_map =
    fn(body_frame_handle: k4abt_frame_t) -> k4a_image_t;

pub(crate) type k4abt_frame_get_capture = fn(body_frame_handle: k4abt_frame_t) -> k4a_capture_t;
