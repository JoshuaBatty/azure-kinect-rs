extern crate nannou;

mod kinect;

use nannou::prelude::*;
use azure_kinect::Api;
use kinect::Kinect;

fn main() {
    nannou::app(model)
        .update(update)
        .backends(wgpu::Backends::DX12)
        .run();
}

struct Model {
    kinect: Kinect,
}

fn model(app: &App) -> Model {
    env_logger::init();

    app.set_loop_mode(LoopMode::rate_fps(30.0));
    
    let w_id = app
        .new_window()
        .size(710, 1200)
        .view(view)
        .build()
        .unwrap();

    let window = app.window(w_id).unwrap();
    let device = window.device();
    
    let azure_api = Api::new().expect("Can't load kinect azure library, make sure k4a.dll & depthengine_2_0.dll are next to the projects executable");
    let kinect = Kinect::new(azure_api.clone(), 0, &device).expect("Can't open kinect azure device");

    Model {
        kinect,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let window = app.main_window();
    let device = window.device();
    let queue = window.queue();

    model.kinect.update(&device, &queue);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let window = app.main_window();
    let rect = window.rect();
    let device = window.device();
    let draw = app.draw();
    draw.background().color(BLACK);

    model.kinect.draw_colour_image(
        &draw, 
        0.0, 
        rect.top() - rect.top() / 3.0, 
        rect.w(), 
        rect.h() / 3.0);

    model.kinect.draw_depth_image(
        &draw, 
        &frame,
        &device,
        0.0, 
        0.0,
        rect.w(), 
        rect.h() / 3.0);

    model.kinect.draw_ir_image(
        &draw, 
        &frame,
        &device,
        0.0, 
        rect.bottom() + rect.top() / 3.0, 
        rect.w(), 
        rect.h() / 3.0);

    let x_count = 20;
    let y_count = 14;
    let mut i = 0;
    for x in 0..x_count {
        for y in 0..y_count {
            let w = rect.w() / x_count as f32;
            let h = (rect.h() / 3.0) / y_count as f32;

            let x = map_range(x,0,x_count, rect.left() + (w/2.0), rect.right() + (w/2.0));
            let y = map_range(y,0,y_count, rect.top() - (h / 2.0), rect.h() / 6.0 - (h/2.0));

            // model.kinect.draw_colour_image(
            //     &draw, 
            //     x, 
            //     y - (rect.h() / 3.0), 
            //     w, 
            //     h);

            // if i%2 == 0 {
            //     model.kinect.draw_depth_image(
            //         &draw, 
            //         &frame,
            //         &device,
            //         x, 
            //         y - (rect.h() / 3.0),
            //         w, 
            //         h);
            // }
            // else {
            //     model.kinect.draw_ir_image(
            //         &draw, 
            //         &frame,
            //         &device,
            //         x, 
            //         y - (rect.h() / 3.0),
            //         w, 
            //         h);
            // }
            

            i += 1;
        }
    }
    

    // // Write to the window frame.
    draw.to_frame(app, &frame).unwrap();
}

