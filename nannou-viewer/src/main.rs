extern crate nannou;

mod azure;

use nannou::prelude::*;
use nannou::image;
use nannou::image::GenericImageView;
use nannou::image::{ImageBuffer,ImageEncoder};
use std::ops::DerefMut;
use std::slice;

use azure_kinect::*;

fn main() {
    nannou::app(model)
        .update(update)
        .exit(exit)
        .run();
}

struct Model {
    //kinect: azure::Kinect,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    azure_rx: azure::Receivers,
    azure_thread: azure::Handle,
}

// The vertex type that we will use to represent a point on our triangle.
#[repr(C)]
#[derive(Clone, Copy)]
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

fn assets_path(app: &App) -> std::path::PathBuf {
    app.project_path()
        .expect("couldn't find project path")
        .join("nannou-viewer")
        .join("assets")        
}

fn model(app: &App) -> Model {
    let (azure_thread, azure_receiver) =  azure::live_cam();

    /*
    let factory = Factory::new().expect("no factory");
    let device = factory.device_open(0).expect("can't open device");
    let camera_config = k4a_device_configuration_t {
        depth_mode: k4a_depth_mode_t::K4A_DEPTH_MODE_NFOV_2X2BINNED,
        color_format: k4a_image_format_t::K4A_IMAGE_FORMAT_COLOR_MJPG,
        ..k4a_device_configuration_t::default()
    };
    let camera = device.start_cameras(&camera_config).expect("can't start camera");
    let image_dimension = camera_config.depth_mode.get_dimension();

    // let mut buffer: &mut [u8] = &mut[];
    //let mut buffer: Vec<u8> = vec![0; 368640];
    //let pitch = 1280;

    if let Ok(capture) = camera.get_capture(1000) {
        // let width = depth_image.get_width_pixels();
        unsafe {
            let mut depth_image = capture.get_depth_image();
            let ptr = depth_image.get_mut_buffer();
            let len = depth_image.get_size();
            let width = depth_image.get_width_pixels();
            let height = depth_image.get_height_pixels();

            println!("k4a_image_get_width_pixels = {}", width);
            println!("k4a_image_get_height_pixels = {}", height);
            println!("k4a_image_get_stride_bytes = {}", depth_image.get_stride_bytes());
            println!("k4a_image_get_size = {}", len);
            println!("width * height pixels = {}", width * height);
            println!("k4a_image_format_t = {:?}", depth_image.get_format());
            
            let depth_data = slice::from_raw_parts(ptr, (height * width) as usize * std::mem::size_of::<u16>());
            //println!("depth_data = {:?}", depth_data);
            let image = image::load_from_memory(&depth_data).unwrap();

            //let img = Vec::from_raw_parts(ptr, len, len);
            //let img = cpu_accessible_buffer_from_slices(Some(&depth_data[..]).into_iter());
            //let converted: ImageBuffer<image::LumaA<u8>, _> = ImageBuffer::from_raw(width as u32, height as u32, depth_data).expect("can't create Image Buffer from raw pixels");
            println!("end of unsafe block");



            // match image::load_from_memory_with_format(&img, image::ImageFormat::Png) {
            //     Ok(i) => {
            //         println!("success");
            //     },
            //     Err(e) => {
            //         println!("error {}",&e.to_string());
            //     }
            // }            

            // encode it into a memory buffer
            // let mut encoded_img = Vec::new();
            // {
            //     let encoder = image::jpeg::JpegEncoder::new_with_quality(&mut encoded_img, 100);
            //     encoder
            //         .write_image(&img, 1, 1, image::ColorType::La16)
            //         .expect("Could not encode image");
            // }
            
        }  
        println!("back to safety");      
    }
    println!("back in app land");

    //println!("buffer = {:?}", &buffer);
    */

    ///////////////////////////////////
    // Load the image.
    let logo_path = assets_path(app).join("nannou.png");
    let image = image::open(logo_path).unwrap();
    //let image = image::load_from_memory(&buffer).unwrap();

    let (img_w, img_h) = image.dimensions();

    let w_id = app
        .new_window()
        .size(1280, 720)
        .view(view)
        .build()
        .unwrap();
    let window = app.window(w_id).unwrap();
    let device = window.swap_chain_device();
    let format = Frame::TEXTURE_FORMAT;
    let msaa_samples = window.msaa_samples();

    let vs_mod = wgpu::shader_from_spirv_bytes(device, include_bytes!("shaders/vert.spv"));
    let fs_mod = wgpu::shader_from_spirv_bytes(device, include_bytes!("shaders/frag.spv"));

    // Load the image as a texture.
    let texture = wgpu::Texture::from_image(&window, &image);
    let texture_view = texture.view().build();

    // Create the sampler for sampling from the source texture.
    let sampler_desc = wgpu::SamplerBuilder::new().into_descriptor();
    let sampler_filtering = wgpu::sampler_filtering(&sampler_desc);
    let sampler = device.create_sampler(&sampler_desc);

    let bind_group_layout =
        create_bind_group_layout(device, texture_view.sample_type(), sampler_filtering);
    let bind_group = create_bind_group(device, &bind_group_layout, &texture_view, &sampler);
    let pipeline_layout = create_pipeline_layout(device, &bind_group_layout);
    let render_pipeline = create_render_pipeline(
        device,
        &pipeline_layout,
        &vs_mod,
        &fs_mod,
        format,
        msaa_samples,
    );

    let vertices_bytes = vertices_as_bytes(&VERTICES[..]);
    let usage = wgpu::BufferUsage::VERTEX;
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: vertices_bytes,
        usage,
    });

    Model {
        bind_group,
        vertex_buffer,
        render_pipeline,
        azure_rx: azure_receiver,
        azure_thread,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    
    
}

fn view(app: &App, model: &Model, frame: Frame) {
    // let mut encoder = frame.command_encoder();
    // let mut render_pass = wgpu::RenderPassBuilder::new()
    //     .color_attachment(frame.texture_view(), |color| color)
    //     .begin(&mut encoder);
    // render_pass.set_bind_group(0, &model.bind_group, &[]);
    // render_pass.set_pipeline(&model.render_pipeline);
    // render_pass.set_vertex_buffer(0, model.vertex_buffer.slice(..));
    // let vertex_range = 0..VERTICES.len() as u32;
    // let instance_range = 0..1;
    // render_pass.draw(vertex_range, instance_range);


    if let Some(colour_image_buffer) = model.azure_rx.color.try_iter().last() {
        let window = app.main_window();
        let device = window.swap_chain_device();
        let queue = window.swap_chain_queue();
        let texture_usage = wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::SAMPLED;

        let colour_texture = wgpu::Texture::load_from_image_buffer(
            &device, 
            &queue,
            texture_usage,
            &colour_image_buffer);
        
        // let flat_samples = colour_image_buffer.as_flat_samples();
        // model.texture.upload_data(
        //     app.main_window().swap_chain_device(),
        //     &mut *frame.command_encoder(),
        //     &flat_samples.as_slice(),
        // );

        let draw = app.draw();
        draw.texture(&colour_texture);

        // Write to the window frame.
        draw.to_frame(app, &frame).unwrap();
    }
}

fn exit(app: &App, model: Model) {
    // TODO: Re-add when using live feed.
    model.azure_thread.close();
}


fn create_bind_group_layout(
    device: &wgpu::Device,
    texture_sample_type: wgpu::TextureSampleType,
    sampler_filtering: bool,
) -> wgpu::BindGroupLayout {
    wgpu::BindGroupLayoutBuilder::new()
        .texture(
            wgpu::ShaderStage::FRAGMENT,
            false,
            wgpu::TextureViewDimension::D2,
            texture_sample_type,
        )
        .sampler(wgpu::ShaderStage::FRAGMENT, sampler_filtering)
        .build(device)
}

fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    texture: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    wgpu::BindGroupBuilder::new()
        .texture_view(texture)
        .sampler(sampler)
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

// See the `nannou::wgpu::bytes` documentation for why this is necessary.
fn vertices_as_bytes(data: &[Vertex]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}


fn get_depth_color(depth: u16, minmax: Range<u16>) -> u32 {
    if depth == 0 {
        return 0xff000000;
    }

    let clamped_value = std::cmp::min(minmax.max, std::cmp::max(depth, minmax.min));

    const RANGE: f32 = 2.0 / 3.0;
    let hue =
        RANGE - (clamped_value - minmax.min) as f32 / (minmax.max - minmax.min) as f32 * RANGE;

    let i = (hue * 6.0) as i32;
    let f = hue * 6.0 - i as f32;

    let rgb = match i {
        0 => (1.0f32, f, 0.0f32),
        1 => (1.0 - f, 1.0f32, 0.0f32),
        2 => (0.0f32, 1.0f32, f),
        3 => (0.0f32, 1.0 - f, 1.0f32),
        4 => (f, 0.0f32, 1.0f32),
        _ => (1.0f32, 0.0f32, 1.0 - f),
    };

    0xff000000
        | (((255.0 * rgb.0) as u32) << 16)
        | (((255.0 * rgb.1) as u32) << 8)
        | ((255.0 * rgb.2) as u32)
}