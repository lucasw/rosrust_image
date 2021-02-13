/*
 * Lucas Walter
 * February 2021
 *
 * Load images from a directory and publish them in sequence, then loop
 */

// mod utility;

// use byteorder::{WriteBytesExt, BigEndian};
use image::GenericImageView;
use rosrust_msg::sensor_msgs::Image;
// use rosrust::api::raii as ros;
// use std::sync::{Arc, Mutex};
// use utility::from_rgb;
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
            if !rosrust::is_ok() {
                break;
            }
            if let Ok(entry) = entry_result {
                let path = entry.path();
                let metadata_result = fs::metadata(&path);
                if let Ok(metadata) = metadata_result {
                    if metadata.is_file() {
                        let _extension = path.extension().unwrap_or_else(|| OsStr::new("none"));
                        // println!("{:?} extension -> {:?}", &path, extension);
                        // skip if not 'png' or 'jpg' or 'jpeg'?
                        let img_result = image::open(&path);
                        if let Ok(img) = img_result {
                            let img_sz = img.dimensions();
                            println!("{:?} {:?}", path, img_sz);
                            // TODO(lucasw) build in one step instead of as mut
                            let mut msg = Image::default();
                            msg.header.stamp = rosrust::now();
                            msg.header.frame_id = "todo".to_string();
                            msg.width = img_sz.0;
                            msg.height = img_sz.1;
                            msg.encoding = "rgb8".to_string();
                            msg.step = msg.width * 3;
                            let size = (msg.step * msg.height) as usize;
                            msg.data.resize(size, 0);
                            // TODO(lucasw) something with zip to iterate through msg.data and img
                            // together?
                            for (i, pixel) in img.pixels().enumerate() {
                                if i as u32 >= msg.width * msg.height {
                                    eprintln!("size mismatch {} {} {}", i, msg.width, msg.height);
                                    break;
                                }
                                msg.data[i * 3] = pixel.2[0];
                                msg.data[i * 3 + 1] = pixel.2[1];
                                msg.data[i * 3 + 2] = pixel.2[2];
                            }
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
