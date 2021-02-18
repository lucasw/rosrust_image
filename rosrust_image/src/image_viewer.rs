mod utility;

use byteorder::{BigEndian, WriteBytesExt};
use pixels::{Pixels, SurfaceTexture};
use rosrust_msg::sensor_msgs::Image;
// use rosrust::api::raii as ros;
use utility::{create_window, from_rgb};
// use std::{fmt, mem, thread, time};
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit_input_helper::WinitInputHelper;

// rosrust::ros_info!("msg {} {} {} {}", msg.width, msg.height, msg.encoding, msg.data.len());
fn image_msg_to_pixels(
    msg: Image,
    screen: &mut [u8],
    screen_width: u32,
    _screen_height: u32
) {
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
            rosrust::ros_warn!(
                "{} {} -> {} {}, {} >= {}, msg {} x {}",
                count,
                screen_width,
                x,
                y,
                msg_byte_base_ind,
                msg.data.len(),
                msg.width,
                msg.height
            );
            break;
        }
        let msg_byte_base_ind = msg_byte_base_ind as usize;
        let val = from_rgb(
            msg.data[msg_byte_base_ind + 2],
            msg.data[msg_byte_base_ind + 1],
            msg.data[msg_byte_base_ind],
        );
        pix.write_u32::<BigEndian>(val).unwrap();
    }
    /*
    rosrust::ros_debug!(
        "msg {} {}, frame {} {}",
        msg.width,
        msg.height,
        screen_width,
        screen_height,
    );
    */
}

fn main() {
    rosrust::init("image_viewer");
    let screen_width: u32 = 2000;
    let screen_height: u32 = 1000;
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let (window, p_width, p_height, mut _hidpi_factor) =
        create_window("Image Viewer", screen_width, screen_height, &event_loop);
    let surface_texture = SurfaceTexture::new(p_width, p_height, &window);
    rosrust::ros_info!(
        "{} {}, {} {}",
        screen_width,
        screen_height,
        p_width,
        p_height
    );

    let mut pixels = Pixels::new(screen_width, screen_height, surface_texture).unwrap();

    let (tx, rx) = crossbeam_channel::unbounded();
    let image_callback = {
        move |msg: Image| {
            if let Ok(_) = tx.send(msg) {
            }
        }
    };
    let _image_sub = rosrust::subscribe("image_in", 4, image_callback).unwrap();

    event_loop.run(move |event, _, control_flow| {
        let mut do_request = false;
        {
            let mut some_msg = None;
            // empty every message from the channel, keep the latest
            loop {
                let possible_msg = rx.try_recv();
                if let Ok(new_msg) = possible_msg {
                    some_msg = Some(new_msg);
                } else {
                    break;
                }
            }
            if let Some(msg) = some_msg {
                let screen = pixels.get_frame();
                image_msg_to_pixels(msg, screen, screen_width, screen_height);
                do_request = true;
            }
        }
        // The one and only event that winit_input_helper doesn't have for us...
        if let Event::RedrawRequested(_) = event {
            if pixels
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
                pixels.resize(size.width, size.height);
                println!("new width {} height {}", size.width, size.height);
            }

            if input.key_held(VirtualKeyCode::Left) || input.key_held(VirtualKeyCode::A) {
                println!("left pressed");
            }

            do_request = true;
        }

        if !rosrust::is_ok() {
            *control_flow = ControlFlow::Exit;
            return;
        }

        if do_request {
            window.request_redraw();
        }
        // rosrust::sleep(rosrust::Duration::from_nanos(1_000_000));
    }); // event_loop.run
}
