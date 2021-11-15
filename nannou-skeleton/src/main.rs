mod kinect;

use nannou::prelude::*;
use nannou::draw::camera::{Camera, CameraController};
use nannou::event::ElementState;
use azure_kinect::{Api, ApiTracker};
use kinect::Kinect;

fn main() {
    nannou::app(model)
        .event(event)
        .update(update)
        .view(view)
        .run();
}

struct Model {
    grid: Grid,
    camera: Camera,
    cam_controller: CameraController,
    kinect: Kinect,
}

fn model(app: &App) -> Model {
    env_logger::init();
    
    app.new_window()
        .size(1280, 720)
        .mouse_wheel(mouse_wheel)
        .key_pressed(key_pressed)
        .key_released(key_released)
        .view(view)
        .build()
        .unwrap();
    
    let azure_api = Api::new().expect("Can't load kinect azure library, make sure k4a.dll & depthengine_2_0.dll are next to the projects executable");
    let tracker_api = ApiTracker::new().expect("Can't load kinect body tracking library, make sure k4abt.dll & onnxruntime.dll & dnn_model_2_0_op11.onnx are next to the projects executable");
    let kinect = Kinect::new(azure_api.clone(), tracker_api.clone(), 0).expect("Can't open kinect azure device");

    let camera = Camera::new()
        .fov(std::f32::consts::PI / 4.0)
        .position(vec3(0.0, 100.0, -600.0));
    let speed = 1000.0;
    let sensitivity = 1.0;
    let cam_controller = CameraController::new(speed, sensitivity);

    Model {
        grid: Grid::new(),  
        camera,
        cam_controller,  
        kinect,
    }
}

fn event(app: &App, model: &mut Model, event: Event) {
    match event {
        Event::DeviceEvent(_device_id, device_event) => {
            if let nannou::winit::event::DeviceEvent::MouseMotion { delta } = device_event {
                if app.mouse.buttons.pressed().next().is_some() {
                    model.cam_controller.process_mouse(delta.0 as f32, delta.1 as f32);
                }
            }
        }
        _ => ()
    }
}


fn update(app: &App, model: &mut Model, update: Update) {
    model.cam_controller.update_camera(&mut model.camera, update.since_last);

    let title = format!("`FPS - `{:.2}`", app.fps());
    app.main_window().set_title(&title);
}

fn mouse_wheel(_app: &App, model: &mut Model, dt: MouseScrollDelta, _phase: TouchPhase) {
    model.cam_controller.process_scroll(&dt);
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    model.cam_controller.process_keyboard(key, ElementState::Pressed);
}

fn key_released(_app: &App, model: &mut Model, key: Key) {
    model.cam_controller.process_keyboard(key, ElementState::Released);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw().camera(model.camera);
    draw.background().rgb(0.07, 0.07, 0.07);

    model.kinect.draw_skeleton(
        &draw, 
        app.window_rect());
    
    // draw the grid
    model.grid.draw(&draw);

    // Draw to the frame!
    draw.to_frame(app, &frame).unwrap();
}

struct Grid {
    lines: Vec<Cuboid>,
}

impl Grid {
    pub fn new() -> Self {
        let num_lines = 2000;
        let world_size = 100000.0;
        let grid_thickness = 2.0;
        let mut lines = Vec::new();
        for i in 0..num_lines {
            let pos = map_range(i, 0, num_lines, -world_size, world_size);
            // lines X
            let centre = pt3(pos, 0.0, 0.0);
            let size = vec3(grid_thickness, grid_thickness, world_size);
            lines.push(geom::Cuboid::from_xyz_whd(centre, size));

            // lines Z
            let centre = pt3(0.0, 0.0, pos);
            let size = vec3(world_size, grid_thickness, grid_thickness);
            lines.push(geom::Cuboid::from_xyz_whd(centre, size));
        }

        Grid {
            lines
        }
    }

    pub fn draw(&self, draw: &Draw) {
        for (i,c) in self.lines.iter().enumerate() {
            let cpoints = c.triangles_iter().flat_map(geom::Tri::vertices);
            let a = if i % 5 == 0 {
                0.1
            } else {
                0.01
            };
            draw.mesh()
                .points(cpoints)
                .color(rgba(1.0, 1.0, 1.0, a));
        }
    }
}