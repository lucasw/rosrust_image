mod utility;

use byteorder::{WriteBytesExt, BigEndian};
use pixels::{Pixels, SurfaceTexture};
use rosrust_msg::sensor_msgs::Image;
// use rosrust::api::raii as ros;
use std::sync::{Arc, Mutex};
use utility::create_window;
// use std::{fmt, mem, thread, time};
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit_input_helper::WinitInputHelper;

pub fn from_rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) << 24 | (g as u32) << 16 | (b as u32) << 8
}

fn main() {
    rosrust::init("image_viewer");
    let screen_width: u32 = 800;
    let screen_height: u32 = 600;
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let (window, p_width, p_height, mut _hidpi_factor) =
        create_window("Image Viewer", screen_width, screen_height, &event_loop);
    let surface_texture = SurfaceTexture::new(p_width, p_height, &window);
    rosrust::ros_info!("{} {}, {} {}", screen_width, screen_height, p_width, p_height);

    // TODO(lucasw) The compiler says this shouldn't be mut, but why not?  It's mut later
    // when it is extracted from the mutex.
    let pixels = Pixels::new(screen_width, screen_height, surface_texture).unwrap();
    let pixels_mutex = Mutex::new(pixels);
    let pixels_arc1 = Arc::new(pixels_mutex);
    let pixels_arc2 = pixels_arc1.clone();

    // let mut paused = false;

    let image_callback = {
        move |msg: Image| {
            // rosrust::ros_info!("msg {} {} {} {}", msg.width, msg.height, msg.encoding, msg.data.len());
            let mut viewer_pixels = Mutex::lock(&pixels_arc2).unwrap();
            let screen = viewer_pixels.get_frame();

            // TODO(lucasw) check encoding
            let channels = 3;
            for (count, mut pix) in screen.chunks_exact_mut(4).enumerate() {
                let x = count as u32 % screen_width;
                let y = count as u32 / screen_width;
                if x >= msg.width {
                    continue;
                }
                if y >= msg.height {
                    break;
                }
                let msg_byte_base_ind = (y * msg.width + x) as u32 * channels;
                if (msg_byte_base_ind + 2) as usize >= msg.data.len() {
                    eprintln!("{} {} -> {} {}, {} >= {}, msg {} x {}", count, screen_width, x, y,
                              msg_byte_base_ind, msg.data.len(), msg.width, msg.height);
                    break;
                }
                let msg_byte_base_ind = msg_byte_base_ind as usize;
                let val = from_rgb(
                    msg.data[msg_byte_base_ind + 2],
                    msg.data[msg_byte_base_ind + 1],
                    msg.data[msg_byte_base_ind + 0],
                );
                pix.write_u32::<BigEndian>(val).unwrap();
            }
            // This skips the pixel buffer and write to the screen,
            // which won't be scaled after a resize
            // screen[viewer_ind] = *pixel;
            rosrust::ros_info!("msg {} {}, frame {} {}", msg.width, msg.height, screen_width, screen_height);
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
                println!("scale factor changed {}", _hidpi_factor);
            }
            // Resize the window
            if let Some(size) = input.window_resized() {
                let mut pixels_locked = Mutex::lock(&pixels_arc1).unwrap();
                pixels_locked.resize(size.width, size.height);
                println!("new width {} height {}", size.width, size.height);
            }

            if input.key_held(VirtualKeyCode::Left) || input.key_held(VirtualKeyCode::A) {
                println!("left pressed");
            }

            window.request_redraw();
        }

        if !rosrust::is_ok() {
            *control_flow = ControlFlow::Exit;
            return;
        }

        // thread::sleep(time::Duration::from_millis(33));
        // This blocks winit inputs
        // rate.sleep();
    }); // event_loop.run
}
