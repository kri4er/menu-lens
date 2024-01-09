use lambda_http::{Body, Error, Request};
use teloxide::types::Update;
use rten_tensor::prelude::*;
use rten_tensor::{NdTensor, NdTensorView};

pub async fn convert_input_to_json(input: Request) -> Result<Update, lambda_http::Error> {
    let body = input.body();
    let body_str = match body {
        Body::Text(text) => text,
        not => panic!("expected Body::Text(...) got {not:?}"),
    };
    let body_json: Update = serde_json::from_str(body_str)?;
    Ok(body_json)
}

pub fn read_image(path: &str) -> Result<NdTensor<f32, 3>, Box<dyn std::error::Error>> {
    let input_img = image::open(path)?;
    let input_img = input_img.into_rgb8();

    let (width, height) = input_img.dimensions();

    let in_chans = 3;
    let mut float_img = NdTensor::zeros([in_chans, height as usize, width as usize]);
    for c in 0..in_chans {
        let mut chan_img = float_img.slice_mut([c]);
        for y in 0..height {
            for x in 0..width {
                chan_img[[y as usize, x as usize]] = input_img.get_pixel(x, y)[c] as f32 / 255.0
            }
        }
    }
    Ok(float_img)
}

pub fn read_buffer(buffer: &Vec<u8>) -> Result<NdTensor<f32, 3>, Box<dyn std::error::Error>> {
    let input_img = image::load_from_memory(buffer)?;
    let input_img = input_img.into_rgb8();

    let (width, height) = input_img.dimensions();

    let in_chans = 3;
    let mut float_img = NdTensor::zeros([in_chans, height as usize, width as usize]);
    for c in 0..in_chans {
        let mut chan_img = float_img.slice_mut([c]);
        for y in 0..height {
            for x in 0..width {
                chan_img[[y as usize, x as usize]] = input_img.get_pixel(x, y)[c] as f32 / 255.0
            }
        }
    }
    Ok(float_img)
}
