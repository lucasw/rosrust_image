// use byteorder::{WriteBytesExt, BigEndian};
use pixels::{Pixels, SurfaceTexture};
use rosrust_msg::sensor_msgs::Image;
// use rosrust::api::raii as ros;
use std::sync::{Arc, Mutex};
// use std::{fmt, mem, thread, time};
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalSize};
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
    });  // event_loop.run
}

// TODO(lucasw) put into library
/// Create a window for the game.
///
/// Automatically scales the window to cover about 2/3 of the monitor height.
///
/// # Returns
///
/// Tuple of `(window, surface, width, height, hidpi_factor)`
/// `width` and `height` are in `PhysicalSize` units.
fn create_window(
    title: &str,
    screen_width: u32,
    screen_height: u32,
    event_loop: &EventLoop<()>,
) -> (winit::window::Window, u32, u32, f64) {
    // Create a hidden window so we can estimate a good default window size
    let window = winit::window::WindowBuilder::new()
        .with_visible(false)
        .with_title(title)
        .build(&event_loop)
        .unwrap();
    let hidpi_factor = window.scale_factor();

    // Get dimensions
    let width = screen_width as f64;
    let height = screen_height as f64;
    let (monitor_width, monitor_height) = {
        if let Some(monitor) = window.current_monitor() {
            let size = monitor.size().to_logical(hidpi_factor);
            (size.width, size.height)
        } else {
            (width, height)
        }
    };
    let scale = (monitor_height / height * 2.0 / 3.0).round().max(1.0);

    // Resize, center, and display the window
    let min_size: winit::dpi::LogicalSize<f64> =
        PhysicalSize::new(width, height).to_logical(hidpi_factor);
    let default_size = LogicalSize::new(width * scale, height * scale);
    let center = LogicalPosition::new(
        (monitor_width - width * scale) / 2.0,
        (monitor_height - height * scale) / 2.0,
    );
    window.set_inner_size(default_size);
    window.set_min_inner_size(Some(min_size));
    window.set_outer_position(center);
    window.set_visible(true);

    let size = default_size.to_physical::<f64>(hidpi_factor);

    (
        window,
        size.width.round() as u32,
        size.height.round() as u32,
        hidpi_factor,
    )
}

