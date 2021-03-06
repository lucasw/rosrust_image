/*
 * Lucas Walter
 * February 2021
 *
 * Load images from a directory and publish them in sequence, then loop
 */

// mod utility;

// use byteorder::{WriteBytesExt, BigEndian};
use image::GenericImageView;
use rosrust_msg::sensor_msgs;
// use rosrust::api::raii as ros;
// use std::sync::{Arc, Mutex};
// use utility::from_rgb;
use std::ffi::OsStr;
use std::io::{Error, ErrorKind};
use std::fs;

fn to_image_msg(img: image::DynamicImage) -> sensor_msgs::Image {
    let img_sz = img.dimensions();
    // println!("{:?}", img_sz);
    // TODO(lucasw) build in one step instead of as mut
    let mut msg = sensor_msgs::Image::default();
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
    msg
}

fn publish_if_image(
    entry: fs::DirEntry,
    image_pub: &rosrust::Publisher<sensor_msgs::Image>,
) -> Result<String, Error> {
    // TODO(lucasw) some kinds of errors are normal and should be ignored,
    // for example if not a file
    // if let Ok(entry) = entry_result {
    // let entry = entry_result?;
    {
        let path = entry.path();
        let metadata = fs::metadata(&path)?;
        {
            if metadata.is_file() {
                let extension = path.extension().unwrap_or_else(|| OsStr::new("none"));
                // TODO(lucasw) publish the path in an adjacent string message
                rosrust::ros_info!("{:?} extension -> {:?}", &path, extension);
                // skip if not 'png' or 'jpg' or 'jpeg'?
                // TODO(lucasw) handle gifs and iterate through frames
                // TODO(lucasw) load cached msg if no changes
                let img_result = image::open(&path);
                if let Ok(img) = img_result {
                    let msg = to_image_msg(img);
                    // the trait `From<rosrust::error::Error>` is not implemented for `std::io::Error`
                    // image_pub.send(msg)?;
                    image_pub.send(msg).unwrap();
                    Ok("publish".to_string())
                } else {
                    // TODO(lucasw) but possible some images are failing to load for other reasons?
                    Err(Error::new(ErrorKind::Other, "not a supported image file"))
                }
            } else {
                Err(Error::new(ErrorKind::Other, "not a file"))
            }
        }
    }
    // unreachable
    // Err(Error::new(ErrorKind::Other, "shouldn't reach here"))
}

/*
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}
*/

fn main() {
    env_logger::init();
    rosrust::init("image_dir_pub");

    // let current_dir = env::current_dir().unwrap().to_str();

    // &str doesn't work as a param, it will compile but the param
    // won't get loaded.
    let image_dir = rosrust::param("~image_dir")
        .unwrap()
        .get()
        // TODO(lucasw) clipppy says this is better, because the string is only generated
        // when the or condition needs it to.
        .unwrap_or_else(|_| ".".to_string());
        // .unwrap_or(".".to_string());
    let publish_rate: f64 = rosrust::param("~publish_rate")
        .unwrap()
        .get()
        .unwrap_or(5.0);
    rosrust::ros_info!("publish rate: {}", publish_rate);
    let rate = rosrust::rate(publish_rate);
    let image_pub = rosrust::publish("image", 4).unwrap();

    rosrust::ros_info!("image dir: '{}'", image_dir);
    let image_path = std::path::Path::new(&image_dir);
    while rosrust::is_ok() {
        let mut num_pubs = 0;
        let mut paths: Vec<_> = fs::read_dir(&image_path)
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        paths.sort_by_key(|dir| dir.path());
        for path in paths {
            if !rosrust::is_ok() {
                break;
            }
            // print_type_of(&entry_result);
            let rv = publish_if_image(path, &image_pub);
            if rv.is_ok() {
                num_pubs += 1;
                rate.sleep();
            }
        }
        if num_pubs == 0 {
            // TODO(lucasw) maybe sleep longer to wait for something to get added to directory
            rate.sleep();
        }
    }
}
