use std::fs;

use ioh_scrap::Item;
use teloxide::{
    prelude::{Request, Requester}, types::{InputFile, InputMedia, InputMediaAudio}
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Upload Telegram!");

    let token = std::env::var("BOT_TOKEN")?;
    let chat_id = std::env::var("CHAT_ID")?;

    let bot = teloxide::Bot::new(token);

    let mut data: Vec<Item> = serde_json::from_str(&fs::read_to_string("data_with_sound.json")?)?;
    data.sort_by(|a, b| {
        let asize: usize = a.document_counter.trim_end_matches(".").parse().unwrap();
        let bsize: usize = b.document_counter.trim_end_matches(".").parse().unwrap();

        asize.cmp(&bsize)
    });

    for item in data.iter() {
        println!("{}: {}", item.document_counter, item.title);

        let file_names = match &item.sound_urls {
            Some(urls) => urls.iter().map(|url| format!("./audio/{}", ioh_scrap::file_name_by_url(url))).collect::<Vec<String>>(),
            None => vec![]
        };

        let cnt_files = file_names.len();
        let media = file_names.iter().enumerate().map(|(i, file_name)| {
            let file = InputFile::file(file_name);

            let audio = if i == cnt_files - 1 {
                InputMediaAudio::new(file).title(format!("{} - Part {}", item.title, i + 1)).caption("caption")
            } else {
                InputMediaAudio::new(file).title(format!("{} - Part {}", item.title, i + 1))
            };

            InputMedia::Audio(audio)
        }).collect::<Vec<InputMedia>>();

        bot.send_media_group(chat_id, media)
            .send()
            .await?;

        break;
    }

    Ok(())
}
