use super::*;
use std::ptr;
use std::sync::Arc;

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
}

impl Drop for Tracker {
    fn drop(&mut self) {
        (self.api_tracker.k4abt_tracker_shutdown)(self.handle);
        self.handle = ptr::null_mut();
    }
}

