use std::{path::Path, fs, fmt};
use rten::Model;

use lambda_http::{http::HeaderMap, Error};
use rten_tensor::NdTensor;

use crate::{utils::{read_image, read_buffer}, output::{OutputFormat, format_text_output}};
use ocrs::{DecodeMethod, OcrEngine, OcrEngineParams};

const DETECTION_MODEL: &str = "/models/text-detection.rten";
const RECOGNITION_MODEL: &str = "/models/text-recognition.rten";
use tracing::info;

/// Adds context to an error reading or parsing a file.
trait FileErrorContext<T> {
    /// If `self` represents a failed operation to read a file, convert the
    /// error to a message of the form "{context} from {path}: {original_error}".
    fn file_error_context<P: fmt::Display>(self, context: &str, path: P) -> Result<T, String>;
}

impl<T, E: std::fmt::Display> FileErrorContext<T> for Result<T, E> {
    fn file_error_context<P: fmt::Display>(self, context: &str, path: P) -> Result<T, String> {
        self.map_err(|err| format!("{} from {}: {}", context, path, err))
    }
}

fn get_exec_dir() -> String {
    let mut rsrc_dir = std::env::current_exe()
        .expect("Can't find path to executable");
    // Directory containing binary
    rsrc_dir.pop();
    rsrc_dir.to_str().unwrap().to_string()
}

async fn transcribe_tensor_image(buffer: NdTensor<f32, 3>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let exec_dir = get_exec_dir();
    info!("Running in the directory: {:?}", exec_dir);
    let detection_model_path = exec_dir.clone() + DETECTION_MODEL;
    let recognition_model_path = exec_dir.clone() + RECOGNITION_MODEL;

    let detection_model = load_model(&detection_model_path)
        .file_error_context("Failed to load text detection model", detection_model_path)?;

    let recognition_model = load_model(&recognition_model_path)
        .file_error_context("Failed to load text recognition model", recognition_model_path)?;

    let color_img = buffer;

    let engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        debug: false,
        decode_method: if true {
            DecodeMethod::BeamSearch { width: 100 }
        } else {
            DecodeMethod::Greedy
        },
    })?;

    let ocr_input = engine.prepare_input(color_img.view())?;
    let word_rects = engine.detect_words(&ocr_input)?;
    let line_rects = engine.find_text_lines(&ocr_input, &word_rects);
    // TODO: join text lines by rectangle distances 
    let line_texts = engine.recognize_text(&ocr_input, &line_rects)?;

    let lines: Vec<String> = line_texts
        .iter()
        .flatten()
        .map(|line| line.to_string())
        .collect();

    Ok(lines)
}

pub async fn transcribe_image(buffer: Vec<u8>) -> Result<Vec<String>, String> {
    let image_content = read_buffer(&buffer).expect("Can't convert image buff into NdTensor<f32, 3>");
    match transcribe_tensor_image(image_content).await {
        Ok(res) => {
            info!("Success {:?}", res);
            Ok(res)
        },
        Err(e) => {
            info!("Failed to process text with err: {:?}", e);
            Err("Can't transribe image with error: {e}".into())
        }
    }
}

pub fn load_model(source_path: &str) -> Result<Model, Box<dyn std::error::Error>> {
    let model_bytes = fs::read(source_path)?;
    let model = Model::load(&model_bytes)?;
    Ok(model)
}

#[cfg(test)]
mod test_aiutils {
    use crate::{aiutils::transcribe_tensor_image, utils::{read_buffer, read_image}};

    #[tokio::test]
    async fn test_generate_annotated_png() {
        let image_path = "/home/ydederkal/develop/rust/menu-lens/data/de_hofbrau.png";

        transcribe_tensor_image(read_image(&image_path).unwrap()).await.ok().unwrap();
        println!("LOGME: Done!!!");
        //assert_eq!(annotated.shape(), img.shape());
    }
}
