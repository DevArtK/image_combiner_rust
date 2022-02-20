mod args;

use args::Args;
use image::{io::Reader, ImageFormat, DynamicImage, GenericImageView, imageops::FilterType::Triangle, ImageError};
use std::convert::TryInto;


#[derive(Debug)]
enum ImageDataErrors {
    DifferentImageFormats,
    BufferTooSmall,
    UnableToReadImageFromPath(std::io::Error),
    UnableToFormatImage(String),
    UnableToDecodeImage(ImageError),
    UnableToSaveImage(ImageError)

}

/// Acts as a temporary storage for Image meta data before being saved
struct FloatingImage {
    width: u32,
    height: u32,
    data: Vec<u8>, // Data will reserve memory for the pixel rgba values, u8 type because rgba stores values from 0 - 255 which is exact size that rgba goes up to
    name: String
}

impl FloatingImage {
    fn new(width: u32, height: u32, name: String) -> Self {
        let buffer_capacity = height * width * 4; // 4: number of pixels' rgba values
        let buffer = Vec::with_capacity(buffer_capacity.try_into().unwrap());

        FloatingImage {
            width,
            height,
            data: buffer,
            name
        }
    }
    // Methods on a struct take in self as first argument

    fn set_data(&mut self, data: Vec<u8>) -> Result<(), ImageDataErrors> {
        // If the data passed in is bigger than the capacity, means buffer is not big enough to hold onto input data
        if data.len() > self.data.capacity() {
            return Err(ImageDataErrors::BufferTooSmall)
        }
        self.data = data;
        Ok(())
    }
}


fn main() -> Result<(), ImageDataErrors> {
    let args = Args::new();
    let (image_1, image_1_format) = find_image_from_path(args.image_1)?;
    let (image_2, image_2_format) = find_image_from_path(args.image_2)?;

    // if images aren't the same formats
    if image_1_format != image_2_format {
        return Err(ImageDataErrors::DifferentImageFormats);
    }

    // Redeclare(shadow) image_1 and image_2 from resizing result
    let (image_1, image_2) = standardize_size(image_1, image_2);


    let mut output = FloatingImage::new(image_1.width(), image_1.height(), args.output);

    let combined_data = combine_images(image_1, image_2);
    output.set_data(combined_data)?;

    if let Err(e) = image::save_buffer_with_format(
        output.name, &output.data,
        output.width, output.height,
        image::ColorType::Rgba8, image_1_format) {
            Err(ImageDataErrors::UnableToSaveImage(e))
        } else {
            Ok(())
        }
}

/// Takes in path as a string, returns 2 DynamicImages from image crate
fn find_image_from_path(path: String) -> Result<(DynamicImage, ImageFormat), ImageDataErrors> {
    // Reader struct implements an open function which takes a path to an image file
    // Returning the Result, unwrap the result (get result)
    // let image_reader: Reader<BufReader<File>> = Reader::open(path).unwrap();
    match Reader::open(&path) {
        Ok(image_reader) => {

            // Get's the image format from the unwrapped value (result) of image reader
            // if let Some() : when dealing with Option
            if let Some(image_format) = image_reader.format() {

                match image_reader.decode() {
                    // Return both values in a tuple (image and it's format)
                    Ok(image) => Ok((image, image_format)),
                    Err(e) => Err(ImageDataErrors::UnableToDecodeImage(e))
                }
            } else {
                    return Err(ImageDataErrors::UnableToFormatImage(path))
                }
        },
            Err(e) => Err(ImageDataErrors::UnableToReadImageFromPath(e))
    }
}

/// Get's the smaller of the two images provided, returns height and width of type u32 of it
fn get_smallest_dimensions(dim_1: (u32, u32), dim_2: (u32, u32)) -> (u32, u32) {
    // Number of pixel in image_1 and image_2 to get size
    let pix_1 = dim_1.0 * dim_1.1;
    let pix_2 = dim_1.0 * dim_1.1;

    // Return the smaller of the two dimensions provided
    return if pix_1 < pix_2 { dim_1 } else { dim_2 }
}

/// Resizes either of the two pictures to the smaller, returns both images, with the bigger one resized to the smaller
fn standardize_size(image_1: DynamicImage, image_2: DynamicImage) -> (DynamicImage, DynamicImage) {

    // Dimensions method comes from image crate
    let (width, height) = get_smallest_dimensions(image_1.dimensions(), image_2.dimensions());

    println!("width: {}, height {}", width, height);

    // If image_2 dimensions are the same as the smallest of the two provided images found, resize image_1
    if image_2.dimensions() == (width, height) {
        (image_1.resize_exact(width, height, Triangle), image_2)
    } else {
        // else if image_1 is equal to the smallest of the two provided images, resize image_2
        (image_1, image_2.resize_exact(width, height, Triangle))
    }
}

// Takes in two images, returns the pixel values in a vector
fn combine_images(image_1: DynamicImage, image_2: DynamicImage) -> Vec<u8> {
    // DynamicImage struct implements to_rgb8 method, which returns
    // an ImageBuffer which contains a Vec<u8>, and implements into_vec method which returns the vec itself
    let vec_1 = image_1.to_rgba8().into_vec();
    let vec_2 = image_2.to_rgba8().into_vec();

    alternate_pixels(vec_1, vec_2)
}

fn alternate_pixels(vec_1: Vec<u8>, vec_2: Vec<u8>) -> Vec<u8> {
    // holds the data after the two images have been alternated,
    // pushing pixels from one vector then the other; alternating
    // To size the combined data, it is matched using vec macro of u8's,
    // and the number of u8's being the same number as vec_1

    let mut combined_data = vec![0u8; vec_1.len()];

    let mut i = 0;

    // Go over each pixel in the size of the vector
    while i < vec_1.len() {
        if i % 8 == 0 {
            // vec_1 is referenced so that it doesn't get passed into the set_rgba function, but a reference does
            combined_data.splice(i ..=i + 3, set_rgba(&vec_1, i, i + 3));
        } else {
            combined_data.splice(i..=i + 3, set_rgba(&vec_2, i, i + 3));
        }
        // 4 because rgba, as accessing the indicies in the vector using modulus
        i += 4;
    }
    combined_data
}

// Takes in a vector, a starting index, and and ending an index and returns the rgba set from those indices
fn set_rgba(vec: &Vec<u8>, start: usize, end: usize) -> Vec<u8> {
    let mut rgba = Vec::new();

    // for every index from start to end inclusive
    for i in start..=end {
        // new variable
        // match in case the value doesn't exist
        let val: u8 = match vec.get(i) {
            Some(v) => *v, // de-referene so that it's not pushing reference values
            None => panic!("Index out of bounds")
        };
        rgba.push(val);
    }

    rgba
}
