use super::utility::*;
use super::*;
use std::ptr;
use std::sync::Arc;

unsafe impl Send for Device {}
unsafe impl Sync for Device {}

pub struct Device {
    pub(crate) api: Arc<Api>,
    pub(crate) handle: k4a_device_t,
}

#[derive(Copy, Clone, Default)]
pub struct ColorControlCapabilities {
    supports_auto: bool,
    min_value: i32,
    max_value: i32,
    step_value: i32,
    default_value: i32,
    default_mode: k4a_color_control_mode_t,
}

impl Device {
    /// Open a k4a device.
    pub fn new(api: Arc<Api>, index: u32) -> Result<Device, Error> {
        let mut handle: k4a_device_t = ptr::null_mut();
        Error::from((api.k4a_device_open)(index, &mut handle)).to_result_fn(|| Self { api, handle })
    }

    /// Starts the K4A device's cameras
    pub fn start_cameras(&self, configuration: &k4a_device_configuration_t) -> Result<(), Error> {
        Error::from((self.api.k4a_device_start_cameras)(
            self.handle,
            configuration,
        ))
        .to_result(())
    }

    /// Stops the K4A device's cameras
    pub fn stop_cameras(&self) {
        (self.api.k4a_device_stop_cameras)(self.handle);
        (self.api.k4a_device_stop_imu)(self.handle);
    }

    /// Reads a sensor capture into cap.  Returns true if a capture was read, false if the read timed out.
    pub fn get_capture(&self, timeout_in_ms: i32) -> Result<Capture, Error> {
        let mut handle: k4a_capture_t = ptr::null_mut();
        Error::from((self.api.k4a_device_get_capture)(
            self.handle,
            &mut handle,
            timeout_in_ms,
        ))
        .to_result_fn(|| Capture::from_handle(self.api.clone(), handle))
    }

    /// Reads a sensor capture into cap.  Returns true if a capture was read, false if the read timed out.
    pub fn get_capture_wait_infinite(&self) -> Result<Capture, Error> {
        self.get_capture(K4A_WAIT_INFINITE)
    }

    /// Get the K4A device serial number
    pub fn get_serialnum(&self) -> Result<String, Error> {
        get_k4a_string(&|serialnum, buffer| {
            (self.api.k4a_device_get_serialnum)(self.handle, serialnum, buffer)
        })
    }

    /// Get the K4A color sensor control value
    pub fn get_color_control(
        &self,
        command: k4a_color_control_command_t,
    ) -> Result<(k4a_color_control_mode_t, i32), Error> {
        let mut mode: k4a_color_control_mode_t =
            k4a_color_control_mode_t::K4A_COLOR_CONTROL_MODE_AUTO;
        let mut value: i32 = 0;
        Error::from((self.api.k4a_device_get_color_control)(
            self.handle,
            command,
            &mut mode,
            &mut value,
        ))
        .to_result((mode, value))
    }

    /// Set the K4A color sensor control value
    pub fn set_color_control(
        &self,
        command: k4a_color_control_command_t,
        mode: k4a_color_control_mode_t,
        value: i32,
    ) -> Result<(), Error> {
        Error::from((self.api.k4a_device_set_color_control)(
            self.handle,
            command,
            mode,
            value,
        ))
        .to_result(())
    }

    pub fn get_color_control_capabilities(
        &self,
        command: k4a_color_control_command_t,
    ) -> Result<ColorControlCapabilities, Error> {
        let mut capabilties = ColorControlCapabilities::default();
        Error::from((self.api.k4a_device_get_color_control_capabilities)(
            self.handle,
            command,
            &mut capabilties.supports_auto,
            &mut capabilties.min_value,
            &mut capabilties.max_value,
            &mut capabilties.step_value,
            &mut capabilties.default_value,
            &mut capabilties.default_mode,
        ))
        .to_result(capabilties)
    }

    /// Get the raw calibration blob for the entire K4A device.
    pub fn get_raw_calibration(&self) -> Result<Vec<u8>, Error> {
        get_k4a_binary_data(&|calibration, buffer| {
            (self.api.k4a_device_get_raw_calibration)(self.handle, calibration, buffer)
        })
    }

    /// Get the camera calibration for the entire K4A device, which is used for all transformation functions.
    pub fn get_calibration(
        &self,
        depth_mode: k4a_depth_mode_t,
        color_resolution: k4a_color_resolution_t,
    ) -> Result<Calibration, Error> {
        let mut calibaraion = k4a_calibration_t::default();
        Error::from((self.api.k4a_device_get_calibration)(
            self.handle,
            depth_mode,
            color_resolution,
            &mut calibaraion,
        ))
        .to_result_fn(|| Calibration::from_handle(self.api.clone(), calibaraion))
    }

    /// Get the device jack status for the synchronization connectors
    pub fn is_sync_connected(&self) -> Result<(bool, bool), Error> {
        let mut sync_in_jack_connected = false;
        let mut sync_out_jack_connected = false;
        Error::from((self.api.k4a_device_get_sync_jack)(
            self.handle,
            &mut sync_in_jack_connected,
            &mut sync_out_jack_connected,
        ))
        .to_result((sync_in_jack_connected, sync_out_jack_connected))
    }

    /// Get the device jack status for the synchronization in connector
    pub fn is_sync_in_connected(&self) -> Result<bool, Error> {
        Ok(self.is_sync_connected()?.0)
    }

    /// Get the device jack status for the synchronization out connector
    pub fn is_sync_out_connected(&self) -> Result<bool, Error> {
        Ok(self.is_sync_connected()?.1)
    }

    /// Get the version numbers of the K4A subsystems' firmware
    pub fn get_version(&self) -> Result<k4a_hardware_version_t, Error> {
        let mut version = k4a_hardware_version_t::default();
        Error::from((self.api.k4a_device_get_version)(self.handle, &mut version)).to_result(version)
    }

    /// Starts the K4A IMU
    pub fn start_imu(&self) -> Result<(), Error> {
        Error::from((self.api.k4a_device_start_imu)(self.handle)).to_result(())
    }

    /// Reads an IMU sample.  Returns true if a sample was read, false if the read timed out.
    pub fn get_imu_sample(&self, timeout_in_ms: i32) -> Result<k4a_imu_sample_t, Error> {
        let mut imu_sample = k4a_imu_sample_t::default();
        Error::from((self.api.k4a_device_get_imu_sample)(
            self.handle,
            &mut imu_sample,
            timeout_in_ms,
        ))
        .to_result(imu_sample)
    }

    pub fn get_imu_sample_wait_infinite(&self) -> Result<k4a_imu_sample_t, Error> {
        self.get_imu_sample(K4A_WAIT_INFINITE)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        (self.api.k4a_device_close)(self.handle);
        self.handle = ptr::null_mut();
    }
}
