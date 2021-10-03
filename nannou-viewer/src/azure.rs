use std::sync::atomic::{self, AtomicBool};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time;
use nannou::image::{Bgra, LumaA, ImageBuffer};

use azure_kinect::*;


// /// Owned form of depth frame data produced by a kinect azure.
// pub struct DepthFrame {
//     pub rows: u32,
//     pub cols: u32,
//     pub data: Vec<u16>,
// }

/// Owned form of color frame data produced by a kinect azure.
// pub struct ColorFrame {
//     pub rows: u32,
//     pub cols: u32,
//     pub data: Vec<u8>,
// }

/// A type for retrieving data from the azure thread.
pub struct Receivers {
    pub depth: mpsc::Receiver<ImageBuffer<LumaA<u16>, Vec<u16>>>,
    pub color: mpsc::Receiver<ImageBuffer<Bgra<u8>, Vec<u8>>>,
}

/// A type acting as a handle to the kinect azure thread.
pub struct Handle {
    closed: Arc<AtomicBool>,
    thread: thread::JoinHandle<()>,
}

impl Handle {
    /// Close and join the azure thread.
    pub fn close(self) {
        let Handle { closed, thread } = self;
        closed.store(true, atomic::Ordering::Relaxed);
        if let Err(err) = thread.join() {
            eprintln!("failed to join kinect azure thread: {:?}", err);
        }
    }
}


pub fn live_cam() -> (Handle, Receivers) {
    let (depth_tx, depth) = mpsc::channel();
    let (color_tx, color) = mpsc::channel();
    let closed = Arc::new(AtomicBool::new(false));
    
    let thread = thread::Builder::new()
        .name("kinect azure".into())
        .spawn(move || {
            let factory = Factory::new().expect("no factory");
            let device = factory.device_open(0).expect("can't open device");
            let camera_config = k4a_device_configuration_t {
                depth_mode: k4a_depth_mode_t::K4A_DEPTH_MODE_NFOV_2X2BINNED,
                ..k4a_device_configuration_t::default()
            };
            let camera = device.start_cameras(&camera_config).expect("can't start camera");

            loop {
                if let Ok(capture) = camera.get_capture(0) {
                    let colour_image = capture.get_color_image();
                    let width = colour_image.get_width_pixels();
                    let height = colour_image.get_height_pixels();
                    let len = colour_image.get_size();
                    //let pitch = width as usize * 4;

                    if width > 0 && height > 0 {
                        //println!("width = {} - height = {} - len = {} - pitch = {}", width, height, len, pitch);
                        let slice = unsafe { std::slice::from_raw_parts(colour_image.get_buffer(), len) };
                        let colour_image_buf: ImageBuffer<Bgra<u8>, _> = ImageBuffer::from_raw(width as u32, height as u32, slice.to_vec())
                                .expect("can't create Image Buffer from raw pixels");
                        color_tx.send(colour_image_buf).ok();                    
                    }

                    // let depth_image = capture.get_depth_image();
                    // let width = depth_image.get_width_pixels();
                    // let height = depth_image.get_height_pixels();
                    // let len = depth_image.get_size();
                    // //let pitch = width as usize * 4;

                    // if width > 0 && height > 0 {
                    //     //println!("width = {} - height = {} - len = {} - pitch = {}", width, height, len, pitch);
                    //     let slice = unsafe { std::slice::from_raw_parts(depth_image.get_buffer(), len) };
                    //     let depth_image_buf: ImageBuffer<LumaA<u16>, _> = ImageBuffer::from_raw(width as u32, height as u32, slice.to_vec())
                    //             .expect("can't create Image Buffer from raw pixels");
                    //     depth_tx.send(depth_image_buf).ok();                    
                    // }
                }

                //thread::sleep(time::Duration::from_millis(11));
            }
        })
        .expect("failed to spawn `nuitrack_player` thread");

    let handle = Handle { closed, thread };
    let receiver = Receivers {
        depth,
        color,
    };
    (handle, receiver)
}



//---------------------------------------------------------------------------
pub struct Kinect<'a> {
    factory: Factory,
    device: Device<'a>,
    camera: Camera<'a>,
}