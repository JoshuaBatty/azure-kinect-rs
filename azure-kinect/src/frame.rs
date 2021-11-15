use super::*;
use std::ptr;
use std::sync::Arc;

pub struct Frame {
    api_tracker: Arc<ApiTracker>,
    pub handle: k4abt_frame_t,
}

impl Frame {
    pub fn from_handle(api_tracker: Arc<ApiTracker>, handle: k4abt_frame_t) -> Frame {
        Frame {
            api_tracker,
            handle: handle,
        }
    }
}
impl Drop for Frame {
    fn drop(&mut self) {
        (self.api_tracker.k4abt_frame_release)(self.handle);
        self.handle = ptr::null_mut();
    }
}
