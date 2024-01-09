use lambda_http::{Body, Error, Request, Response};

use teloxide::{
    net::Download,
    payloads::SendMessageSetters,
    requests::Requester,
    types::UpdateKind,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    Bot,
};
use tracing::info;
use reqwest;

use crate::aiutils;
use crate::utils;

pub async fn handle_telegram_request(req: Request) -> Result<Response<Body>, Error> {
    let bot = Bot::new(env::var("TELEGRAM_BOT_TOKEN").unwrap());

    info!("Received request {:?}", req);

    let update = utils::convert_input_to_json(req).await?;

    // Match the update type
    match update.kind {
        // If the update is a message
        UpdateKind::Message(message) => {
            // Check if the message is a voice message
            if message.photo().is_none() {
                info!("Not a photo message");
                bot.send_message(message.chat.id, "I only accept photo messages.")
                    .reply_to_message_id(message.id)
                    .disable_web_page_preview(true)
                    .disable_notification(true)
                    .allow_sending_without_reply(true)
                    .await?;

                return Ok(Response::builder()
                    .status(200)
                    .body(Body::Text("Not a photo message".into()))
                    .unwrap());
            }

            let photo = message.photo()
                .expect("Photo is not present in the message.")
                .last().expect("Last photo object is not valid.");

            let file = bot.get_file(&photo.file.id).await?;
            let file_path = file.path.clone();
            let mut buffer:Vec<u8> = Vec::new();
            info!("Downloading file by id: {:?} to buffer", &photo.file.id);
            bot.download_file(&file_path, &mut buffer).await?;

            // Send file to OpenAI Whisper for transcription
            info!("Transcribe image using OCR using English lang");
            match aiutils::transcribe_image(buffer).await {
                Ok(lines) => {
                    let keyboard:Vec<InlineKeyboardButton> = lines.into_iter()
                        .filter(|l| l.len() > 2)
                        .map(|line| InlineKeyboardButton::url(
                                line.clone(),
                                reqwest::Url::parse(
                                    format!("https://www.google.com/search?q={}&tbm=isch", line.replace(" ", "+")).to_string().as_ref()
                                    ).ok().unwrap()
                                )
                            ).collect();
                    let chunks:Vec<Vec<InlineKeyboardButton>> = keyboard.chunks(3).map(|k|k.to_vec()).collect();
                    if chunks.len() > 0 {
                        let mut markup = InlineKeyboardMarkup::new([chunks[0].clone()]);
                        for (_, chunk) in chunks.into_iter().skip(1).enumerate() {
                            markup = markup.append_row(chunk);
                        }

                        bot.send_message(message.chat.id, "Please pick the dish.".to_string())
                            .reply_markup(markup)
                            .reply_to_message_id(message.id)
                            //.disable_web_page_preview(true)
                            .disable_notification(true)
                            .allow_sending_without_reply(true)
                            .await?;
                    } else {
                        bot.send_message(message.chat.id, "Sorry, can't parse your image.".to_string())
                            .reply_to_message_id(message.id)
                            .disable_web_page_preview(true)
                            .disable_notification(true)
                            .allow_sending_without_reply(true)
                            .await?;
                    }
                                
                },
                Err(_) => {
                    bot.send_message(message.chat.id, "Sorry, can't parse your image.".to_string())
                        .reply_to_message_id(message.id)
                        .disable_web_page_preview(true)
                        .disable_notification(true)
                        .allow_sending_without_reply(true)
                        .await?;
                },
            };
        }
        _ => { }
    }

    Ok(Response::builder()
        .status(200)
        .body(Body::Text("OK".into()))
        .unwrap())
}
