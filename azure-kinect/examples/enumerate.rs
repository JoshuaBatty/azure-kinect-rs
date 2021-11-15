use azure_kinect::*;

pub fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let api = Api::new()?;
    let device_count = api.device_get_installed_count();
    println!("Found {} connected devices:", device_count);

    for device_idx in 0..device_count {
        match Device::new(api.clone(), device_idx) {
            Ok(device) => {
                println!("Device {} | serial number: {} | firmware version: {:#?}", 
                device_idx, 
                device.get_serialnum()?,
                device.get_version()?);
            },
            Err(err) => print!("Couldn't open Device {}", device_idx)
        }
    }
    Ok(())
}
