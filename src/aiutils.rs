use std::{path::Path, fs, fmt};
use rten::Model;

use lambda_http::{http::HeaderMap, Error};

use crate::{utils::read_image, output::{OutputFormat, format_text_output}};
use ocrs::{DecodeMethod, OcrEngine, OcrEngineParams};

const DETECTION_MODEL: &str = "models/text-detection.rten";
const RECOGNITION_MODEL: &str = "models/text-recognition.rten";

/// Adds context to an error reading or parsing a file.
trait FileErrorContext<T> {
    /// If `self` represents a failed operation to read a file, convert the
    /// error to a message of the form "{context} from {path}: {original_error}".
    fn file_error_context<P: fmt::Display>(self, context: &str, path: P) -> Result<T, String>;
}

impl<T, E: std::fmt::Display> FileErrorContext<T> for Result<T, E> {
    fn file_error_context<P: fmt::Display>(self, context: &str, path: P) -> Result<T, String> {
        self.map_err(|err| format!("{} from \"{}\": {}", context, path, err))
    }
}

async fn transcribe_image_low() -> Result<(), Box<dyn std::error::Error>> {
    let model_path:&str = "/home/ydederkal/develop/rust/menu-lens/models/text-detection.rten";

    let detection_model = load_model(model_path.into())
        .file_error_context("Failed to load text detection model", model_path)?;

    let recognition_model_src:&str = "/home/ydederkal/develop/rust/menu-lens/models/text-recognition.rten";
    let recognition_model = load_model(recognition_model_src)
        .file_error_context("Failed to load text recognition model", recognition_model_src)?;

    let image_path = "/home/ydederkal/develop/rust/menu-lens/data/de_hofbrau.png";
    let color_img = read_image(&image_path)?;

    let engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        debug: true,
        decode_method: if false {
            DecodeMethod::BeamSearch { width: 100 }
        } else {
            DecodeMethod::Greedy
        },
    }).unwrap();

    let ocr_input = engine.prepare_input(color_img.view())?;
    let word_rects = engine.detect_words(&ocr_input)?;
    let line_rects = engine.find_text_lines(&ocr_input, &word_rects);
    let line_texts = engine.recognize_text(&ocr_input, &line_rects)?;

    let output_path:Option<String> = None;

    let write_output_str = |content: String| -> Result<(), Box<dyn std::error::Error>> {
        if let Some(output_path) = &output_path {
            std::fs::write(output_path, content.into_bytes())?;
        } else {
            println!("{}", content);
        }
        Ok(())
    };


    let output_format = OutputFormat::Text;
    match output_format {
        OutputFormat::Text => {
            let content = format_text_output(&line_texts);
            write_output_str(content)?;
        }
        _ => panic!("Do not know what to do with png")
        /*
        OutputFormat::Json => {
            let content = format_json_output(FormatJsonArgs {
                input_path: &args.image,
                input_hw: color_img.shape()[1..].try_into()?,
                text_lines: &line_texts,
            });
            write_output_str(content)?;
        }
        */
        /*
        OutputFormat::Png => {
            let png_args = GeneratePngArgs {
                img: color_img.view(),
                line_rects: &line_rects,
                text_lines: &line_texts,
            };
            let annotated_img = generate_annotated_png(png_args);
            let Some(output_path) = args.output_path else {
                return Err("Output path must be specified when generating annotated PNG".into());
            };
            write_image(&output_path, annotated_img.view())?;
        }
        */
    }

    
    Ok(())
}

pub async fn transcribe_image(buffer: Vec<u8>) -> Result<Vec<String>, Error> {
    transcribe_image_low().await.ok().unwrap();
    let result = vec!["Hello from lambda image to text using OCRs".to_string()];
    Ok(result)
}



pub fn load_model(source_path: &str) -> Result<Model, Box<dyn std::error::Error>> {
    let model_bytes = fs::read(source_path)?;
    let model = Model::load(&model_bytes)?;
    Ok(model)
}
