extern crate nannou;

mod kinect;

use nannou::prelude::*;
use nannou::image;
use nannou::image::GenericImageView;
use nannou::image::{ImageBuffer,ImageEncoder};
use std::ops::DerefMut;
use std::slice;

use azure_kinect::*;
use kinect::Kinect;

fn main() {
    nannou::app(model)
        .update(update)
        .exit(exit)
        .run();
}

struct Model {
    kinect: Kinect,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
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
    env_logger::init();
    ///////////////////////////////////
    // Load the image.
    let logo_path = assets_path(app).join("nannou.png");
    let image = image::open(logo_path).unwrap();
    //let image = image::load_from_memory(&buffer).unwrap();

    let w_id = app
        .new_window()
        .size(1280, 720)
        .view(view)
        .build()
        .unwrap();

    let window = app.window(w_id).unwrap();
    let device = window.device();
    let format = Frame::TEXTURE_FORMAT;
    let msaa_samples = window.msaa_samples();

    let vs_desc = wgpu::include_wgsl!("shaders/vs.wgsl");
    let fs_desc = wgpu::include_wgsl!("shaders/fs.wgsl");
    let vs_mod = device.create_shader_module(&vs_desc);
    let fs_mod = device.create_shader_module(&fs_desc);

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
    let usage = wgpu::BufferUsages::VERTEX;
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: vertices_bytes,
        usage,
    });

    let azure_api = Api::new().expect("Can't load kinect azure library");
    let kinect = Kinect::new(azure_api.clone(), 0).expect("Can't open kinect azure device");
    

    Model {
        kinect,
        bind_group,
        vertex_buffer,
        render_pipeline,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let window = app.main_window();
    let device = window.device();
    let queue = window.queue();
    model.kinect.update(&device, &queue);
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

    let draw = app.draw();
    if let Some(colour_texture) = &model.kinect.colour_texture {
        draw.texture(&colour_texture);
    }

    // if let Some(depth_texture) = &model.kinect.depth_texture {
    //     draw.texture(&depth_texture);
    // }
    // Write to the window frame.
    draw.to_frame(app, &frame).unwrap();
}

fn exit(_app: &App, _model: Model) {

}


fn create_bind_group_layout(
    device: &wgpu::Device,
    texture_sample_type: wgpu::TextureSampleType,
    sampler_filtering: bool,
) -> wgpu::BindGroupLayout {
    wgpu::BindGroupLayoutBuilder::new()
        .texture(
            wgpu::ShaderStages::FRAGMENT,
            false,
            wgpu::TextureViewDimension::D2,
            texture_sample_type,
        )
        .sampler(wgpu::ShaderStages::FRAGMENT, sampler_filtering)
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