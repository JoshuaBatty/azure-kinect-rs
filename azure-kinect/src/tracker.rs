use super::*;
use std::ptr;
use std::sync::Arc;

unsafe impl Send for Tracker {}
unsafe impl Sync for Tracker {}

pub struct Tracker {
    pub(crate) api_tracker: Arc<ApiTracker>,
    pub(crate) handle: k4abt_tracker_t,
}

impl Tracker {
    pub fn new(
        api_tracker: Arc<ApiTracker>,
        sensor_calibration: &k4a_calibration_t,
        config: k4abt_tracker_configuration_t,
    ) -> Result<Tracker, Error> {
        let mut handle: k4abt_tracker_t = ptr::null_mut();
        Error::from((api_tracker.k4abt_tracker_create)(
            sensor_calibration,
            config,
            &mut handle,
        ))
        .to_result_fn(|| Self {
            api_tracker,
            handle,
        })
    }

    /// Add a k4a sensor capture to the tracker input queue to generate its body tracking result asynchronously.
    pub fn enqueue_capture(
        &self,
        sensor_capture_handle: k4a_capture_t,
        timeout_in_ms: i32,
    ) -> Result<(), Error> {
        Error::from((self.api_tracker.k4abt_tracker_enqueue_capture)(
            self.handle,
            sensor_capture_handle,
            timeout_in_ms,
        ))
        .to_result(())
    }

    /// Gets the next available body frame.
    pub fn pop_result(&self, timeout_in_ms: i32) -> Result<Frame, Error> {
        let mut handle: k4abt_frame_t = ptr::null_mut();
        Error::from((self.api_tracker.k4abt_tracker_pop_result)(
            self.handle,
            &mut handle,
            timeout_in_ms,
        ))
        .to_result_fn(|| Frame::from_handle(self.api_tracker.clone(), handle))
    }

    /// Get the number of people from the k4abt_frame_t
    pub fn get_num_bodies(&self, body_frame: &Frame) -> u32 {
        (self.api_tracker.k4abt_frame_get_num_bodies)(body_frame.handle)
    }

    pub fn get_body(&self, body_frame: &Frame, index: u32) -> Result<k4abt_body_t, Error> {
        let id = self.get_body_id(body_frame, index);
        let skeleton = self.get_body_skeleton(body_frame, index)?;

        Ok(k4abt_body_t { skeleton, id })
    }

    /// Get the joint information for a particular person index from the k4abt_frame_t.
    pub fn get_body_skeleton(&self, body_frame: &Frame, index: u32) -> Result<k4abt_skeleton_t, Error> {
        let mut skeleton = k4abt_skeleton_t::default();
        Error::from((self.api_tracker.k4abt_frame_get_body_skeleton)(
            body_frame.handle,
            index,
            &mut skeleton,
        ))
        .to_result(skeleton)
    }

    /// Get the body id for a particular person index from the k4abt_frame_t.
    pub fn get_body_id(&self, body_frame: &Frame, index: u32) -> u32 {
        (self.api_tracker.k4abt_frame_get_body_id)(body_frame.handle, index)
    }

    /// Control the temporal smoothing across frames.
    /// 
    /// Set between 0 for no smoothing and 1 for full smoothing. 
    /// Less smoothing will increase the responsiveness of the detected skeletons 
    /// but will cause more positional and orientational jitters.
    pub fn set_temporal_smoothing(&self, smoothing_factor: f32) {
        (self.api_tracker.k4abt_tracker_set_temporal_smoothing)(self.handle, smoothing_factor)
    }

    /// Get the body frame's device timestamp in microseconds.
    /// 
    /// Returns the timestamp of the body frame. If the body_frame_handle is invalid this function will return 0. 
    /// It is also possible for 0 to be a valid timestamp originating from the beginning of a recording or the start of streaming.
    pub fn get_device_timestamp_usec(&self, body_frame: &Frame) -> u64 {
        (self.api_tracker.k4abt_frame_get_device_timestamp_usec)(body_frame.handle)
    }
}

impl Drop for Tracker {
    fn drop(&mut self) {
        (self.api_tracker.k4abt_tracker_shutdown)(self.handle);
        self.handle = ptr::null_mut();
    }
}

