use std::sync::Arc;
use nannou::image::{Bgra, Luma, ImageBuffer};
use nannou::prelude::*;

use azure_kinect::*;

pub struct Kinect {
    device: Device,
    tracker: Tracker,
}

impl Kinect {
    pub fn new(azure_api: Arc<Api>, tracker_api: Arc<ApiTracker>, device_idx: u32) -> Result<Kinect, Box<dyn std::error::Error>> {
        let device = Device::new(azure_api, device_idx)?;
        let camera_config = k4a_device_configuration_t {
            depth_mode: k4a_depth_mode_t::K4A_DEPTH_MODE_NFOV_UNBINNED,
            ..k4a_device_configuration_t::default()
        };
        device.start_cameras(&camera_config)?;

        let sensor_calibration = device
        .get_calibration(
            camera_config.depth_mode,
            k4a_color_resolution_t::K4A_COLOR_RESOLUTION_OFF,
        )
        .expect("Get depth camera calibration failed!");

        let tracker_config = k4abt_tracker_configuration_t::default();
        let tracker = Tracker::new(
            tracker_api,
            &sensor_calibration.calibration,
            tracker_config,
        )
        .expect("Body tracker initialization failed!");

        Ok( Kinect {
            device,
            tracker,
        })
    }

    pub fn draw_skeleton(&self, draw: &Draw, rect: Rect) {
        if let Ok(capture) = self.device.get_capture(0) {
            match self.tracker.enqueue_capture(capture.handle, K4A_WAIT_INFINITE) {
                Ok(_) => (),
                Err(err) => match azure_kinect::Error::from(err) {
                    Error::Timeout => {
                        println!("Error! Add capture to tracker process queue timeout!")
                    }
                    Error::Failed => {
                        println!("Error! Add capture to tracker process queue failed!")
                    }
                    _ => println!("an unexpected error occured"),
                },
            }

            if let Ok(body_frame) = self.tracker.pop_result(K4A_WAIT_INFINITE) {
                let num_bodies = self.tracker.get_num_bodies(&body_frame);
                //println!("{} bodies detected!", num_bodies);

                let cube_size = 100.0;
                let dist = 1.0;

                for i in 0..num_bodies {
                    if let Ok(body) = self.tracker.get_body(&body_frame, i) {
                        // print_body_information(body);
                        for i in 0..k4abt_joint_id_t::K4ABT_JOINT_COUNT as usize {
                            let position = body.skeleton.joints[i].position;
                            unsafe {
                                let mut y = position.xyz.y;
                                y *= -1.0;
                                let position = pt3(position.xyz.x, y + 500.0, position.xyz.z);

                                let orientation = body.skeleton.joints[i].orientation;
                                let confidence_level = body.skeleton.joints[i].confidence_level;
                        
                                let hsva = hsva(dist, 1.0, 0.5, 0.2);
                                draw_cube(&draw, position, cube_size, dist, hsva);
                            }
                            
                        }
                    }
                }
            }
        }
    }
}


fn print_body_information(body: k4abt_body_t) {
    println!("Body ID: {}", body.id);
    for i in 0..k4abt_joint_id_t::K4ABT_JOINT_COUNT as usize {
        let position = body.skeleton.joints[i].position;
        let orientation = body.skeleton.joints[i].orientation;
        let confidence_level = body.skeleton.joints[i].confidence_level;

        unsafe {
            println!("Joint[{}]: Position[mm] ({}, {}, {}); Orientation ({}, {}, {}, {}); Confidence Level {:?}", i, position.xyz.x, position.xyz.y, position.xyz.z, orientation.wxyz.w, orientation.wxyz.x, orientation.wxyz.y, orientation.wxyz.z, confidence_level);
        }
    }
}


fn draw_cube(draw: &Draw, pos: Point3, size: f32, dist: f32, hsva: Hsva) {
    let cuboid = geom::Cuboid::from_xyz_whd(pos, vec3(size, size, size) * dist);
    let wireframe = create_wireframe(&cuboid, dist * 3.0);
    
    // draw the center
    let cpoints = cuboid.triangles_iter().flat_map(geom::Tri::vertices);
    draw.mesh()
        .points(cpoints)
        .color(hsva);

    // draw the wireframe
    for w in &wireframe {
        let wpoints = w.triangles_iter().flat_map(geom::Tri::vertices);
        draw.mesh()
            .points(wpoints)
            .color(BLACK);
    }    
}

fn create_wireframe(cuboid: &Cuboid, wire_width: f32) -> Vec<Cuboid> {
    let x = cuboid.x();
    let y = cuboid.y();
    let z = cuboid.z();
    let ww = cuboid.w();
    let xx = ww * 0.5;
    let hh = cuboid.h();
    let yy = hh * 0.5;
    let dd = cuboid.d();
    let zz = dd * 0.5;
    let w = wire_width; 
    vec![
        //top
        Cuboid::from_x_y_z_w_h_d(x + -xx, y + yy, z + 0.0, w, w, dd),
        Cuboid::from_x_y_z_w_h_d(x + 0.0, y + yy, z + zz, ww, w, w),
        Cuboid::from_x_y_z_w_h_d(x + xx, y + yy, z + 0.0, w, w, dd),
        Cuboid::from_x_y_z_w_h_d(x + 0.0, y + yy, z + -zz, ww, w, w),
        //bottom
        Cuboid::from_x_y_z_w_h_d(x + -xx, y + -yy, z + 0.0, w, w, dd),
        Cuboid::from_x_y_z_w_h_d(x + 0.0, y + -yy, z + zz, ww, w, w),
        Cuboid::from_x_y_z_w_h_d(x + xx, y + -yy, z + 0.0, w, w, dd),
        Cuboid::from_x_y_z_w_h_d(x + 0.0, y + -yy, z + -zz, ww, w, w),
        //sides
        Cuboid::from_x_y_z_w_h_d(x + -xx, y + 0.0, z + -zz, w, hh, w),
        Cuboid::from_x_y_z_w_h_d(x + -xx, y + 0.0, z + zz, w, hh, w),
        Cuboid::from_x_y_z_w_h_d(x + xx, y + 0.0, z + zz, w, hh, w),
        Cuboid::from_x_y_z_w_h_d(x + xx, y + 0.0, z + -zz, w, hh, w),
    ]
}
