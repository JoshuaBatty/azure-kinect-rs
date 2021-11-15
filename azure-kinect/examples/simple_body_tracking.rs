use azure_kinect::*;

pub fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let api = Api::new()?;
    let device = Device::new(api.clone(), 0).expect("Open K4A Device failed!");

    let camera_config = k4a_device_configuration_t {
        depth_mode: k4a_depth_mode_t::K4A_DEPTH_MODE_NFOV_UNBINNED,
        ..k4a_device_configuration_t::default()
    };

    device
        .start_cameras(&camera_config)
        .expect("Start K4A cameras failed!");

    let sensor_calibration = device
        .get_calibration(
            camera_config.depth_mode,
            k4a_color_resolution_t::K4A_COLOR_RESOLUTION_OFF,
        )
        .expect("Get depth camera calibration failed!");

    let api_tracker = ApiTracker::new()?;
    let tracker_config = k4abt_tracker_configuration_t::default();
    let tracker = Tracker::new(
        api_tracker.clone(),
        &sensor_calibration.calibration,
        tracker_config,
    )
    .expect("Body tracker initialization failed!");

    for frame_count in 0..10000 {
        if let Ok(capture) = device.get_capture_wait_infinite() {
            //println!("Start processing frame {}", frame_count);

            match tracker.enqueue_capture(capture.handle, K4A_WAIT_INFINITE) {
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

            if let Ok(body_frame) = tracker.pop_result(K4A_WAIT_INFINITE) {
                let num_bodies = tracker.get_num_bodies(&body_frame);
                println!("{} bodies detected!", num_bodies);

                for i in 0..num_bodies {
                    if let Ok(body) = tracker.get_body(&body_frame, i) {
                        print_body_information(body);
                    }
                }
            }
        }
    }

    Ok(())
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

// void print_body_information(k4abt_body_t body)
// {
//     printf("Body ID: %u\n", body.id);
//     for (int i = 0; i < (int)K4ABT_JOINT_COUNT; i++)
//     {
//         k4a_float3_t position = body.skeleton.joints[i].position;
//         k4a_quaternion_t orientation = body.skeleton.joints[i].orientation;
//         k4abt_joint_confidence_level_t confidence_level = body.skeleton.joints[i].confidence_level;
//         printf("Joint[%d]: Position[mm] ( %f, %f, %f ); Orientation ( %f, %f, %f, %f); Confidence Level (%d) \n",
//             i, position.v[0], position.v[1], position.v[2], orientation.v[0], orientation.v[1], orientation.v[2], orientation.v[3], confidence_level);
//     }
// }
