mod utility;

// use byteorder::{WriteBytesExt, BigEndian};
use pixels::{Pixels, SurfaceTexture};
use rosrust_msg::sensor_msgs::Image;
// use rosrust::api::raii as ros;
use std::sync::{Arc, Mutex};
use utility::create_window;
// use std::{fmt, mem, thread, time};
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit_input_helper::WinitInputHelper;

fn main() {
    let screen_width: u32 = 800;
    let screen_height: u32 = 600;
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let (window, p_width, p_height, mut _hidpi_factor) =
        create_window("Image Viewer", screen_width, screen_height, &event_loop);
    let surface_texture = SurfaceTexture::new(p_width, p_height, &window);

    // The compiler says this shouldn't be mut, but why not?  It's mut later
    // when it is extracted from the mutex.
    let pixels = Pixels::new(screen_width, screen_height, surface_texture).unwrap();
    let pixels_mutex = Mutex::new(pixels);
    let pixels_arc1 = Arc::new(pixels_mutex);
    let pixels_arc2 = pixels_arc1.clone();

    // let mut paused = false;

    rosrust::init("image_viewer");

    let image_callback = {
        move |msg: Image| {
            // rosrust::ros_info!("msg {} {} {} {}", msg.width, msg.height, msg.encoding, msg.data.len());
            let mut viewer_pixels = Mutex::lock(&pixels_arc2).unwrap();
            let screen = viewer_pixels.get_frame();

            // TODO(lucasw) check encoding
            let channels = 3;
            for (count, pixel) in msg.data.iter().enumerate() {
                let ind = count as u32 / channels;
                let byte_ind = count as u32 - ind * channels;
                let x = (ind / msg.width) as u32;
                let y = (ind % msg.width) as u32;

                if x >= screen_width {
                    continue;
                }
                if y >= screen_height {
                    continue;
                }

                let remapped_byte_ind;
                // change bgr to rgb
                // TODO(lucasw) only if bgr
                if byte_ind == 0 {
                    remapped_byte_ind = 2;
                } else if byte_ind == 2 {
                    remapped_byte_ind = 0;
                } else {
                    remapped_byte_ind = 1;
                }

                let viewer_ind = ((y * screen_width + x) * 4 + remapped_byte_ind) as usize;
                screen[viewer_ind] = *pixel;
            }
        }
    };

    let _image_sub = rosrust::subscribe("image_in", 4, image_callback).unwrap();

    let rate = rosrust::rate(30.0);

    event_loop.run(move |event, _, control_flow| {
        // The one and only event that winit_input_helper doesn't have for us...
        if let Event::RedrawRequested(_) = event {
            // scene.update();
            // scene.view.render(pixels.get_frame());

            let mut pixels_locked = Mutex::lock(&pixels_arc1).unwrap();
            if pixels_locked
                .render()
                .map_err(|e| eprintln!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // For everything else, for let winit_input_helper collect events to build its state.
        // It returns `true` when it is time to update our game state and request a redraw.
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Adjust high DPI factor
            if let Some(factor) = input.scale_factor_changed() {
                _hidpi_factor = factor;
            }
            // Resize the window
            if let Some(size) = input.window_resized() {
                let mut pixels_locked = Mutex::lock(&pixels_arc1).unwrap();
                pixels_locked.resize(size.width, size.height);
            }

            window.request_redraw();
        }

        if !rosrust::is_ok() {
            *control_flow = ControlFlow::Exit;
            return;
        }

        // thread::sleep(time::Duration::from_millis(33));
        rate.sleep();
    }); // event_loop.run
}
