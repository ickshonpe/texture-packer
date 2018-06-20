extern crate image;
extern crate serde;
extern crate serde_json;

use image::{GenericImage, DynamicImage};
use std::collections::HashMap;
use std::fs::{File, ReadDir};
use std::io::Write;
use std::path::Path;
use std::ffi::OsStr;

type OutputManifest = HashMap<String, (u32, u32, u32, u32)>;

fn load_images() -> Vec<(String, DynamicImage)> {
    let args = std::env::args().collect::<Vec<_>>();
    let path_name = if args.len() > 1 { &args[1] } else { "." };
    let dir: ReadDir = std::fs::read_dir(path_name).unwrap();
    let mut source_images: Vec<(String, DynamicImage)> = Vec::new();
    for dir_entry in dir {
        if let Ok(entry) = dir_entry {
            if entry.file_type().unwrap().is_file() {
                let path = entry.path();
                if  path.extension() == Some(OsStr::new("png")) {                    
                        let input_buffer = image::open(&path).unwrap();
                        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
                        source_images.push((name, input_buffer));                    
                }
            }
        }
    }
    source_images    
}

fn pack_by_decreasing_height(source_images: &mut Vec<(String, DynamicImage)>) -> ( image::RgbaImage, OutputManifest) {    
    source_images.sort_unstable_by(
        |&(_, ref a), &(_, ref b)| {
            let (_, height_a) = a.dimensions();
            let (_, height_b) = b.dimensions();
            height_a.cmp(&height_b).reverse()
        });
    let output_size = 2048;
    let mut output_buffer: image::RgbaImage = image::ImageBuffer::new(output_size, output_size);
    let mut output_manifest: OutputManifest = HashMap::with_capacity(source_images.len());
    let mut out_x = 0;
    let mut out_y = 0;
    let mut row_height = 0;
    for (name, image) in source_images {
        let (image_width, image_height) = image.dimensions();
        if out_y + image_height <= output_size
            && image_width <= output_size {
            if output_size <= out_x + image_width {
                out_x = 0;
                out_y += row_height;
                if output_size <= out_y + image_height {
                    row_height = 0;
                    continue;
                }
                row_height = image_height;
            } else {
                row_height = std::cmp::max(row_height, image_height);
            }
            for x in 0..image_width {
                for y in 0..image_height {
                    let in_pixel = image.get_pixel(x, y);
                    output_buffer.put_pixel(x + out_x, y + out_y, in_pixel);
                }
            }
            output_manifest.insert(name.to_string(), (out_x, out_y, image_width, image_height));
            out_x += image_width;
        }
    }
    (output_buffer, output_manifest)
}

fn write_images(output_buffer: image::RgbaImage, output_manifest: OutputManifest) {    
    let ref mut output_file = File::create(&Path::new("tileset.png")).unwrap();
    let _ = image::ImageRgba8(output_buffer).save(output_file, image::PNG);
    let serialized_manifest = serde_json::to_string(&output_manifest).unwrap();
    let ref mut output_file = File::create(&Path::new("tileset.json")).unwrap();
    let _ = output_file.write_all(serialized_manifest.as_bytes());
}

fn main() {
    let mut source_images = load_images();
    let (output_buffer, output_manifest) = pack_by_decreasing_height(&mut source_images);
    write_images(output_buffer, output_manifest);
}
