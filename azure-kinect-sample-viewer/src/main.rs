use azure_kinect::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

fn main() {
    if let Err(e) = main2() {
        println!("{:?}", e);
    }
}

fn main2() -> Result<(), Box<dyn std::error::Error>> {
    let factory = Factory::new()?;
    let device = factory.device_open(0)?;
    let camera_config = k4a_device_configuration_t::default();
    let camera = device.start_cameras(&camera_config)?;

    #[cfg(feature = "depth-view")]
    let calibration = device.get_calibration(camera_config.depth_mode, camera_config.color_resolution)?;
    #[cfg(feature = "depth-view")]
    let transformation = Transformation::new(&factory, &calibration);

    let color_image_dimension = camera_config.color_resolution.get_dimension();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(
            "azure-kinect-sample-viewer",
            color_image_dimension.width as u32,
            color_image_dimension.height as u32,
        )
        .position_centered()
        .opengl()
        .build()?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(
            PixelFormatEnum::ARGB8888,
            color_image_dimension.width as u32,
            color_image_dimension.height as u32,
        )
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        if let Ok(capture) = camera.get_capture(1) {
            texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
                #[cfg(feature = "depth-view")]
                {
                    let depth_image = capture.get_depth_image();
                    let cvtd = transformation.depth_image_to_color_camera(&depth_image);
                    if let Ok(cvtd) = cvtd {
                        let width = cvtd.get_width_pixels();
                        unsafe {
                            for y in 0..cvtd.get_height_pixels() as usize {
                                let p = cvtd.get_buffer().add(y * cvtd.get_stride_bytes() as usize) as *const u16;
                                let p2 = buffer.as_mut_ptr().add(y * pitch) as *mut u32;
                                for x in 0..width as isize {
                                    let value = *p.offset(x);
                                    *p2.offset(x) = 0xff000000 | value as u32;
                                }
                            }
                        }
                    }
                }

                #[cfg(not(feature = "depth-view"))]
                {
                    let image = capture.get_color_image();
                    let width = image.get_width_pixels();
                    for y in 0..image.get_height_pixels() as usize {
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                image
                                    .get_buffer()
                                    .add(y * image.get_stride_bytes() as usize),
                                buffer.as_mut_ptr().add(y * pitch),
                                (width * 4) as usize,
                            );
                        }
                    }
                }
            })?;
            canvas.clear();
            canvas.copy(&texture, None, None)?;
            canvas.present();
        }
    }

    Ok(())
}
