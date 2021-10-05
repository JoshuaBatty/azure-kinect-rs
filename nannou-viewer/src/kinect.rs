use std::sync::Arc;
use nannou::image::{Bgra, Luma, ImageBuffer};
use nannou::prelude::*;
use byteorder::{ByteOrder, LittleEndian}; // 1.3.4

use azure_kinect::*;

pub struct Kinect {
    device: Device,
    pub colour_texture: Option<wgpu::Texture>,
    pub depth_texture: Option<wgpu::Texture>,
}

impl Kinect {
    pub fn new(azure_api: Arc<Api>, device_idx: u32) -> Result<Kinect, Box<dyn std::error::Error>> {
        let device = Device::new(azure_api, device_idx)?;
        let camera_config = k4a_device_configuration_t {
            depth_mode: k4a_depth_mode_t::K4A_DEPTH_MODE_NFOV_UNBINNED,
            camera_fps: k4a_fps_t::K4A_FRAMES_PER_SECOND_30,
            color_resolution: k4a_color_resolution_t::K4A_COLOR_RESOLUTION_1080P,
            synchronized_images_only: true,
            ..k4a_device_configuration_t::default()
        };
        device.start_cameras(&camera_config)?;

        Ok(Self{
            device,
            colour_texture: None,
            depth_texture: None,
        })
    }

    pub fn update(&mut self, wgpu_device: &wgpu::Device, queue: &wgpu::Queue) {
        let texture_usage = wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING;

        if let Ok(capture) = self.device.get_capture(0) {
            let colour_image = capture.get_color_image();
            let format = colour_image.get_format();
            let width = colour_image.get_width_pixels();
            let height = colour_image.get_height_pixels();
            let len = colour_image.get_size();
            let pitch = width as usize * 4;

            if width > 0 && height > 0 {
                //println!("format = {:?} - width = {} - height = {} - len = {} - pitch = {}", format, width, height, len, pitch);
                let slice = unsafe { std::slice::from_raw_parts(colour_image.get_buffer(), len) };
                let colour_image_buffer: ImageBuffer<Bgra<u8>, _> = ImageBuffer::from_raw(width as u32, height as u32, slice.to_vec())
                        .expect("can't create Image Buffer from raw pixels");

                self.colour_texture = Some(wgpu::Texture::load_from_image_buffer(
                    wgpu_device, 
                    queue,
                    texture_usage,
                    &colour_image_buffer));                 
            }

            let depth_image = capture.get_depth_image();
            let format = depth_image.get_format();
            let width = depth_image.get_width_pixels();
            let height = depth_image.get_height_pixels();
            let len = depth_image.get_size();
            let pitch = width as usize * 4;

            if width > 0 && height > 0 {
                //println!("format = {:?} - width = {} - height = {} - len = {} - pitch = {}", format, width, height, len, pitch);
                let slice = unsafe { std::slice::from_raw_parts(depth_image.get_buffer(), len) };
                // 1: Iterate over slice and grab a chunk of 2
                // 2: for each chunk convert the 2 u8's into a single u16
                // 3: Collect the u16's into a vector 
                // 4: Upload the data into an ImageBuffer of type Gray16
                let mut v = Vec::new();
                for chunk in slice.chunks(2) {
                    let data = [chunk[0], chunk[1]];
                    v.push(LittleEndian::read_u16(&data));
                }
                let depth_image_buffer: ImageBuffer<Luma<u16>, _> = ImageBuffer::from_raw(width as u32, height as u32, v)
                    .expect("can't create Image Buffer from raw pixels");
                self.depth_texture = Some(wgpu::Texture::load_from_image_buffer(
                    wgpu_device, 
                    queue,
                    texture_usage,
                    &depth_image_buffer));  
                    
                //println!("depth format {:#?}", &self.depth_texture.as_ref().unwrap().descriptor());
            }
        }
    }
}

// texture binding in shaders assume float 

// we need a copy of these shaders for each texture type
// let fs_desc = wgpu::include_wgsl!("shaders/fs.wgsl");

// when we call create_render_pipeline() we should choose what shader to pass in based on the 
// texture type.

// Have a hashmap from texture type to shader module

// Change this and follow the compile errors :):

// fs_mod: wgpu::ShaderModule field to fs_mod: HashMap<wgpu::TextureSampleType, wgpu::ShaderModule>