/*
 * Lucas Walter
 * February 2021
 *
 * Load images from a directory and publish them in sequence, then loop
 */

mod utility;

use byteorder::{WriteBytesExt, BigEndian};
use env_logger;
use image;
use image::GenericImageView;  // ::dimensions;
use rosrust;
use rosrust_msg::sensor_msgs::Image;
// use rosrust::api::raii as ros;
// use std::sync::{Arc, Mutex};
use utility::from_rgb;
use std::ffi::OsStr;
use std::{env, fs};

fn main() {
    env_logger::init();
    rosrust::init("image_dir_pub");

    let update_period = rosrust::param("~update_period").unwrap().get().unwrap_or(0.2);
    rosrust::ros_info!("update period: {}", update_period);
    let rate = rosrust::rate(1.0 / update_period);
    let image_pub = rosrust::publish("image", 4).unwrap();

    while rosrust::is_ok() {
        let mut num_pubs = 0;
        let current_dir = env::current_dir().unwrap();
        for entry_result in fs::read_dir(current_dir).unwrap() {
            if let Ok(entry) = entry_result {
                let path = entry.path();
                let metadata_result = fs::metadata(&path);
                if let Ok(metadata) = metadata_result {
                    if metadata.is_file() {
                        println!("{:?} extension -> {:?}", &path, path.extension().unwrap_or(OsStr::new("none")));
                        let img_result = image::open(&path);
                        if let Ok(img) = img_result {
                            println!("{:?} {:?}", path, img.dimensions());
                            // TODO(lucasw) build in one step instead of as mut
                            let mut msg = Image::default();
                            msg.header.stamp = rosrust::now();
                            msg.header.frame_id = "todo".to_string();
                            msg.width = 10;
                            msg.height = 10;
                            msg.encoding = "rgb8".to_string();
                            msg.step = msg.width * 3;
                            let size = (msg.step * msg.height) as usize;
                            msg.data.resize(size, 0);
                            image_pub.send(msg).unwrap();
                            num_pubs += 1;
                            rate.sleep();
                        }
                    }
                }
            }
        }
        if num_pubs == 0 {
            // TODO(lucasw) maybe sleep longer to wait for something to get added to directory
            rate.sleep();
        }
    }
}
