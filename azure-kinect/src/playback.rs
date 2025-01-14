use super::utility::*;
use super::*;
use crate::playback_data_block::PlaybackDataBlock;
use crate::playback_track::PlaybackTrack;
use std::ptr;
use std::sync::Arc;

pub struct Playback {
    pub(crate) api: Arc<Api>,
    pub(crate) api_record: Arc<ApiRecord>,
    pub(crate) handle: k4a_playback_t,
}

impl Playback {
    /// Opens a K4A recording for playback.
    pub fn playback_open(
        api: Arc<Api>,
        api_record: Arc<ApiRecord>,
        path: &str,
    ) -> Result<Playback, Error> {
        let mut handle: k4a_playback_t = ptr::null_mut();
        let path = std::ffi::CString::new(path).unwrap_or_default();
        Error::from((api_record.k4a_playback_open)(path.as_ptr(), &mut handle)).to_result_fn(|| {
            Self {
                api,
                api_record,
                handle,
            }
        })
    }

    /// Get the raw calibration blob for the K4A device that made the recording.
    pub fn get_raw_calibration(&self) -> Result<Vec<u8>, Error> {
        get_k4a_binary_data(&|calibration, buffer| {
            (self.api_record.k4a_playback_get_raw_calibration)(
                self.handle,
                calibration,
                buffer as *mut size_t,
            )
        })
    }

    /// Get the camera calibration for the K4A device that made the recording, which is used for all transformation
    pub fn get_calibration(&self) -> Result<Calibration, Error> {
        let mut calibaraion = k4a_calibration_t::default();
        Error::from((self.api_record.k4a_playback_get_calibration)(
            self.handle,
            &mut calibaraion,
        ))
        .to_result_fn(|| Calibration::from_handle(self.api.clone(), calibaraion))
    }

    /// Gets the configuration of the recording
    pub fn get_record_configuration(&self) -> Result<k4a_record_configuration_t, Error> {
        let mut configuration = k4a_record_configuration_t::default();
        Error::from((self.api_record.k4a_playback_get_record_configuration)(
            self.handle,
            &mut configuration,
        ))
        .to_result(configuration)
    }

    /// Get the next capture in the recording.
    pub fn get_next_capture(&self) -> Result<Capture, Error> {
        let mut handle: k4a_capture_t = ptr::null_mut();
        Error::from((self.api_record.k4a_playback_get_next_capture)(
            self.handle,
            &mut handle,
        ))
        .to_result_fn(|| Capture::from_handle(self.api.clone(), handle))
    }

    /// Get the previous capture in the recording.
    pub fn get_previous_capture(&self) -> Result<Capture, Error> {
        let mut handle: k4a_capture_t = ptr::null_mut();
        Error::from((self.api_record.k4a_playback_get_previous_capture)(
            self.handle,
            &mut handle,
        ))
        .to_result_fn(|| Capture::from_handle(self.api.clone(), handle))
    }

    /// Reads the value of a tag from the recording
    pub fn get_tag(&self, name: &str) -> Result<String, Error> {
        let name = std::ffi::CString::new(name).unwrap_or_default();
        get_k4a_string(&|tag, buffer| {
            (self.api_record.k4a_playback_get_tag)(
                self.handle,
                name.as_ptr(),
                tag,
                buffer as *mut size_t,
            )
        })
    }

    /// Get the next IMU sample in the recording.
    pub fn get_next_imu_sample(&self) -> Result<k4a_imu_sample_t, Error> {
        let mut imu_sample = k4a_imu_sample_t::default();
        Error::from((self.api_record.k4a_playback_get_next_imu_sample)(
            self.handle,
            &mut imu_sample,
        ))
        .to_result(imu_sample)
    }

    /// Get the previous IMU sample in the recording.
    pub fn get_previous_imu_sample(&self) -> Result<k4a_imu_sample_t, Error> {
        let mut imu_sample = k4a_imu_sample_t::default();
        Error::from((self.api_record.k4a_playback_get_previous_imu_sample)(
            self.handle,
            &mut imu_sample,
        ))
        .to_result(imu_sample)
    }

    /// Seeks to a specific time point in the recording
    pub fn seek_timestamp(
        &self,
        offset_usec: i64,
        origin: k4a_playback_seek_origin_t,
    ) -> Result<(), Error> {
        Error::from((self.api_record.k4a_playback_seek_timestamp)(
            self.handle,
            offset_usec,
            origin,
        ))
        .to_result(())
    }

    /// Get the last valid timestamp in the recording
    pub fn get_recording_length_usec(&self) -> u64 {
        (self.api_record.k4a_playback_get_recording_length_usec)(self.handle)
    }

    /// Set the image format that color captures will be converted to. By default the conversion format will be the
    /// same as the image format stored in the recording file, and no conversion will occur.
    pub fn set_color_conversion(&self, format: k4a_image_format_t) -> Result<(), Error> {
        Error::from((self.api_record.k4a_playback_set_color_conversion)(
            self.handle,
            format,
        ))
        .to_result(())
    }

    /// Get the next data block in the recording.
    pub fn get_next_data_block(&self, track: &str) -> Result<PlaybackDataBlock, Error> {
        let mut block_handle: k4a_playback_data_block_t = ptr::null_mut();
        let track = std::ffi::CString::new(track).unwrap_or_default();

        Error::from((self.api_record.k4a_playback_get_next_data_block)(
            self.handle,
            track.as_ptr(),
            &mut block_handle,
        ))
        .to_result_fn(|| PlaybackDataBlock::from_handle(&self.api_record, block_handle))
    }

    /// Get the previous data block from the recording.
    pub fn get_previous_data_block(&self, track: &str) -> Result<PlaybackDataBlock, Error> {
        let mut block_handle: k4a_playback_data_block_t = ptr::null_mut();
        let track = std::ffi::CString::new(track).unwrap_or_default();

        Error::from((self.api_record.k4a_playback_get_previous_data_block)(
            self.handle,
            track.as_ptr(),
            &mut block_handle,
        ))
        .to_result_fn(|| PlaybackDataBlock::from_handle(&self.api_record, block_handle))
    }

    /// Get the attachment block from the recording.
    pub fn get_attachment(&self, attachment: &str) -> Result<Vec<u8>, Error> {
        let attachment = std::ffi::CString::new(attachment).unwrap_or_default();
        get_k4a_binary_data(&|data, data_size| {
            (self.api_record.k4a_playback_get_attachment)(
                self.handle,
                attachment.as_ptr(),
                data,
                data_size as *mut size_t,
            )
        })
    }

    /// Get the number of tracks in a playback file.
    pub fn get_track_count(&self) -> usize {
        (self.api_record.k4a_playback_get_track_count)(self.handle) as usize
    }

    /// Gets the track at a specific index.
    pub fn get_track(&self, track_index: usize) -> Result<PlaybackTrack, Error> {
        Ok(PlaybackTrack::new(
            &self,
            get_k4a_cstring(&|track_name, track_name_size| {
                (self.api_record.k4a_playback_get_track_name)(
                    self.handle,
                    track_index as size_t,
                    track_name,
                    track_name_size as *mut size_t,
                )
            })?,
        ))
    }
}

impl Drop for Playback {
    fn drop(&mut self) {
        (self.api_record.k4a_playback_close)(self.handle);
        self.handle = ptr::null_mut();
    }
}
