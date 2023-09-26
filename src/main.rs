mod rgb16;
mod rgb8;

use anyhow::{anyhow, Result};
use image::io::Reader as ImageReader;
use image::{self, DynamicImage, ImageBuffer};

// Return the list of image files in the given directory
fn get_image_files(root: &str) -> Vec<String> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            files.append(&mut get_image_files(path.to_str().unwrap()));
        } else {
            let path = path.to_str().unwrap();
            if path.ends_with(".jpg") || path.ends_with(".png") {
                // There is no "_sdr" in the path
                if !path.contains("_sdr") {
                    files.push(path.to_string());
                }
            }
        }
    }
    files
}

fn add_postfix_to_file_name(file_name: &str, postfix: &str) -> String {
    let mut file_name = file_name.to_string();
    let index = file_name.rfind(".").unwrap();
    file_name.insert_str(index, postfix);
    file_name
}

fn main() -> Result<()> {
    // List all of the images "jpg, png" in "./data/"
    let img_files = get_image_files("./data/");
    for img_file in img_files {
        let sdr_img_file = add_postfix_to_file_name(&img_file, "_sdr");
        let sdr_img = hdr_to_sdr(&img_file)?;
        sdr_img.save(sdr_img_file)?;
        println!("{:?}", sdr_img.color());
    }
    Ok(())
}

fn hdr_to_sdr(file_name: &str) -> Result<DynamicImage> {
    let img = ImageReader::open(file_name).unwrap().decode().unwrap();
    match img {
        image::DynamicImage::ImageRgb8(img) => {
            let width = img.width();
            let height = img.height();
            let raw = img.into_raw();
            let sdr = rgb8::to_sdr(raw, height, width)?;
            let sdr_img = DynamicImage::ImageRgb8(
                ImageBuffer::from_raw(width, height, sdr).ok_or_else(|| anyhow!(""))?,
            );
            Ok(sdr_img)
        }
        image::DynamicImage::ImageRgb16(img) => {
            let width = img.width();
            let height = img.height();
            let raw = img.into_raw();
            let sdr = rgb16::to_sdr(raw, height, width)?;
            let sdr_img = DynamicImage::ImageRgb8(
                ImageBuffer::from_raw(width, height, sdr).ok_or_else(|| anyhow!(""))?,
            );
            Ok(sdr_img)
        }
        _ => panic!("Color {:?} type not supported", img.color()),
    }
}
