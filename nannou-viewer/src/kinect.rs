use std::sync::Arc;
use nannou::image::{Bgra, Luma, ImageBuffer};
use nannou::prelude::*;
use byteorder::{ByteOrder, LittleEndian}; 

use azure_kinect::*;

pub struct Kinect {
    device: Device,
    pub colour_texture: Option<wgpu::Texture>,
    pub depth_dimension: Vec2,
    pub depth_range: Vec2,
    
    vertex_buffer: wgpu::Buffer,
    
    pipeline: Pipeline,
    depth_bind_group: Option<wgpu::BindGroup>,
    ir_bind_group: Option<wgpu::BindGroup>,
}

pub struct Pipeline {
    bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: wgpu::RenderPipeline,
}

impl Kinect {
    pub fn new(azure_api: Arc<Api>, device_idx: u32, wgpu_device: &wgpu::Device) -> Result<Kinect, Box<dyn std::error::Error>> {
        let device = Device::new(azure_api, device_idx)?;
        let camera_config = k4a_device_configuration_t {
            // depth_mode: k4a_depth_mode_t::K4A_DEPTH_MODE_NFOV_2X2BINNED,
            depth_mode: k4a_depth_mode_t::K4A_DEPTH_MODE_NFOV_UNBINNED,
            //depth_mode: k4a_depth_mode_t::K4A_DEPTH_MODE_WFOV_2X2BINNED,
            camera_fps: k4a_fps_t::K4A_FRAMES_PER_SECOND_30,
            color_resolution: k4a_color_resolution_t::K4A_COLOR_RESOLUTION_1080P,
            synchronized_images_only: true,
            ..k4a_device_configuration_t::default()
        };
        device.start_cameras(&camera_config)?;

        let r = camera_config.depth_mode.get_range();
        let depth_range = vec2(r.min as f32, r.max as f32);

        let d = camera_config.depth_mode.get_dimension();
        let depth_dimension = vec2(d.width as f32, d.height as f32);

        //------------------------------------------------------------------------------------//
        //------------------------------------------------------------------------------------//
        //------------------------------------------------------------------------------------//
        let vs_desc = wgpu::include_wgsl!("shaders/vs.wgsl");
        let vs_mod = wgpu_device.create_shader_module(&vs_desc);

        let fs_desc = wgpu::include_wgsl!("shaders/colour_fs.wgsl");
        let fs_mod = wgpu_device.create_shader_module(&fs_desc);

        let vertices_bytes = vertices_as_bytes(&VERTICES[..]);
        let usage = wgpu::BufferUsages::VERTEX;
        let vertex_buffer = wgpu_device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: vertices_bytes,
            usage,
        });

        let sample_type = wgpu::TextureSampleType::Float{filterable: true};

        let bind_group_layout = create_bind_group_layout(wgpu_device, sample_type);
        let pipeline_layout = create_pipeline_layout(wgpu_device, &bind_group_layout);
        let render_pipeline = create_render_pipeline(
            wgpu_device,
            &pipeline_layout,
            &vs_mod,
            &fs_mod,
            Frame::TEXTURE_FORMAT,
            1,
        );
        let pipeline = Pipeline {
            bind_group_layout,
            render_pipeline,
        };

        Ok(Self{
            device,
            colour_texture: None,
            depth_dimension,
            depth_range,

            pipeline,
            vertex_buffer,
            depth_bind_group: None,
            ir_bind_group: None,
        })
    }

    pub fn draw_colour_image(&self, draw: &Draw, x: f32, y: f32, w: f32, h: f32) {
        if let Some(colour_texture) = &self.colour_texture {
            draw.texture(&colour_texture).x_y(x,y).w_h(w,h);
        }
    }

    pub fn draw_depth_image(&self, draw: &Draw, frame: &Frame, wgpu_device: &wgpu::Device, x: f32, y: f32, w: f32, h: f32) {
        self.draw_renderpass(&draw, &frame, &wgpu_device, &self.depth_bind_group, x, y, w, h);
    }

    pub fn draw_ir_image(&self, draw: &Draw, frame: &Frame, wgpu_device: &wgpu::Device, x: f32, y: f32, w: f32, h: f32) {
        self.draw_renderpass(&draw, &frame, &wgpu_device, &self.ir_bind_group, x, y, w, h);
    }

    fn draw_renderpass(&self, draw: &Draw, frame: &Frame, wgpu_device: &wgpu::Device, bind_group: &Option<wgpu::BindGroup>, x: f32, y: f32, w: f32, h: f32) {
        let usage = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        if let Some(bind_group) = bind_group {
            let dim = self.depth_dimension;
            let texture = wgpu::TextureBuilder::new()
                .dimension(wgpu::TextureDimension::D2)
                .format(Frame::TEXTURE_FORMAT)
                .extent(wgpu::Extent3d {
                    width: dim.x as u32,
                    height: dim.y as u32,
                    depth_or_array_layers: 1,
                })
                .usage(usage)
                .build(&wgpu_device);
            let texture_view = texture.view().build();
        
            let mut encoder = frame.command_encoder();
            let mut render_pass = wgpu::RenderPassBuilder::new()
                .color_attachment(&texture_view, |color| color)
                .begin(&mut encoder);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.set_pipeline(&self.pipeline.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        
            let vertex_range = 0..VERTICES.len() as u32;
            let instance_range = 0..1;
            render_pass.draw(vertex_range, instance_range);
            draw.texture(&texture_view).x_y(x,y).w_h(w,h);
        } 
    }

    pub fn update(&mut self, wgpu_device: &wgpu::Device, queue: &wgpu::Queue) {
        // Create the sampler for sampling from the source texture.
        let sampler_desc = wgpu::SamplerBuilder::new().into_descriptor();
        let sampler_filtering = wgpu::sampler_filtering(&sampler_desc);
        let sampler = wgpu_device.create_sampler(&sampler_desc);
        
        //let texture_usage = wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT;
        let texture_usage = wgpu::TextureUsages::TEXTURE_BINDING;
        if let Ok(capture) = self.device.get_capture(0) {
            let colour_image = capture.get_color_image();
            let _format = colour_image.get_format();
            let width = colour_image.get_width_pixels();
            let height = colour_image.get_height_pixels();
            let len = colour_image.get_size();
            let _pitch = width as usize * 4;

            let slice = unsafe { std::slice::from_raw_parts(colour_image.get_buffer(), len) };
            let colour_image_buffer: ImageBuffer<Bgra<u8>, _> = ImageBuffer::from_raw(width as u32, height as u32, slice.to_vec())
                        .expect("can't create Image Buffer from raw pixels");

            if width > 0 && height > 0 {
                //println!("format = {:?} - width = {} - height = {} - len = {} - pitch = {}", format, width, height, len, pitch);
                

                self.colour_texture = Some(wgpu::Texture::load_from_image_buffer(
                    wgpu_device, 
                    queue,
                    texture_usage,
                    &colour_image_buffer));      
                    
                //println!("colour format {:#?}", &self.colour_texture.as_ref().unwrap().descriptor());
            }

            let depth_image = capture.get_color_image();
            let width = depth_image.get_width_pixels();
            let height = depth_image.get_height_pixels();

            if width > 0 && height > 0 {
                let slice = unsafe { std::slice::from_raw_parts(depth_image.get_buffer(), depth_image.get_size()) };
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

                let depth_texture = wgpu::Texture::load_from_image_buffer(
                    wgpu_device, 
                    queue,
                    texture_usage,
                    &colour_image_buffer);  

                //println!("depth format {:#?}", &depth_texture.descriptor());
                    
                let uniforms = create_uniforms(self.depth_dimension, self.depth_range, true);  
                let depth_texture_view = depth_texture.view().build();
                self.depth_bind_group = Some(create_bind_group(
                    wgpu_device, 
                    &self.pipeline.bind_group_layout, 
                    &depth_texture_view,
                    &sampler,
                    uniforms));

                //println!("depth_bind_group {:#?}", &self.depth_bind_group.as_ref().unwrap());
            } 


            let ir_image = capture.get_color_image();
            let width = ir_image.get_width_pixels();
            let height = ir_image.get_height_pixels();

            if width > 0 && height > 0 {
                let slice = unsafe { std::slice::from_raw_parts(ir_image.get_buffer(), ir_image.get_size()) };
                let mut v = Vec::new();
                for chunk in slice.chunks(2) {
                    let data = [chunk[0], chunk[1]];
                    v.push(LittleEndian::read_u16(&data));
                }
                let ir_image_buffer: ImageBuffer<Luma<u16>, _> = ImageBuffer::from_raw(width as u32, height as u32, v)
                    .expect("can't create Image Buffer from raw pixels");
                let ir_texture = wgpu::Texture::load_from_image_buffer(
                    wgpu_device, 
                    queue,
                    texture_usage,
                    &colour_image_buffer);      
                    
                let uniforms = create_uniforms(self.depth_dimension, self.depth_range, false);    
                let ir_texture_view = ir_texture.view().build();
                self.ir_bind_group = Some(create_bind_group(
                    wgpu_device, 
                    &self.pipeline.bind_group_layout, 
                    &ir_texture_view, 
                    &sampler,
                    uniforms));

                //println!("ir_bind_group {:#?}", &self.ir_bind_group.as_ref().unwrap());
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


// The vertex type that we will use to represent a point on our triangle.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Vertex {
    position: [f32; 2],
}

// The vertices that make up the rectangle to which the image will be drawn.
const VERTICES: [Vertex; 4] = [
    Vertex {
        position: [-1.0, 1.0],
    },
    Vertex {
        position: [-1.0, -1.0],
    },
    Vertex {
        position: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0],
    },
];

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    resolution: [f32; 2],
    min_max_range: [f32; 2],
    draw_colour: u32,
}

fn create_uniforms(resolution: Vec2, range: Vec2, draw_colour: bool) -> Uniforms {
    let draw_colour = if draw_colour {
        1 
    } else {
        0
    };
    Uniforms {
        resolution: [resolution.x, resolution.y],
        min_max_range: [range.x, range.y],
        draw_colour,
    }
}

// See the `nannou::wgpu::bytes` documentation for why this is necessary.
fn vertices_as_bytes(data: &[Vertex]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}

fn create_bind_group_layout(
    device: &wgpu::Device,
    texture_sample_type: wgpu::TextureSampleType,
) -> wgpu::BindGroupLayout {
    let uniform_dynamic = false;
    wgpu::BindGroupLayoutBuilder::new()
        .texture(
            wgpu::ShaderStages::FRAGMENT,
            false,
            wgpu::TextureViewDimension::D2,
            texture_sample_type,
        )
        .sampler(wgpu::ShaderStages::FRAGMENT, true)
        .uniform_buffer(wgpu::ShaderStages::FRAGMENT, uniform_dynamic)
        .build(device)
}

fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    texture: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    uniforms: Uniforms,
) -> wgpu::BindGroup {
    let usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
    let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("uniform-buffer"),
        contents: bytemuck::cast_slice(&[uniforms]),
        usage,
    });

    wgpu::BindGroupBuilder::new()
        .texture_view(texture)
        .sampler(sampler)
        .buffer::<Uniforms>(&uniform_buffer, 0..1)
        .build(device, layout)
}

fn create_pipeline_layout(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    let desc = wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    };
    device.create_pipeline_layout(&desc)
}

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    vs_mod: &wgpu::ShaderModule,
    fs_mod: &wgpu::ShaderModule,
    dst_format: wgpu::TextureFormat,
    sample_count: u32,
) -> wgpu::RenderPipeline {
    wgpu::RenderPipelineBuilder::from_layout(layout, vs_mod)
        .fragment_shader(fs_mod)
        .color_format(dst_format)
        .add_vertex_buffer::<Vertex>(&wgpu::vertex_attr_array![0 => Float32x2])
        .sample_count(sample_count)
        .primitive_topology(wgpu::PrimitiveTopology::TriangleStrip)
        .build(device)
}

