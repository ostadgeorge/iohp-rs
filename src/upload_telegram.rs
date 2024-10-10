use std::fs;

use ioh_scrap::Item;
use teloxide::{
    prelude::{Request, Requester},
    types::{InputFile, InputMedia, InputMediaAudio},
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
            Some(urls) => urls
                .iter()
                .map(|url| format!("./audio/{}", ioh_scrap::file_name_by_url(url)))
                .collect::<Vec<String>>(),
            None => vec![],
        };

        let mut metadata: Vec<(String, String)> = item
            .metadata
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        metadata.sort_by(|a, b| a.0.cmp(&b.0));

        let caption = format!(
            "{} {}\n\n{}",
            item.document_counter,
            item.title,
            metadata.iter().fold(String::new(), |acc, (k, v)| {
                format!("{}{}: {}\n", acc, k, v)
            })
        );

        let cnt_files = file_names.len();
        let media = file_names
            .iter()
            .enumerate()
            .map(|(i, file_name)| {
                let file = InputFile::file(file_name);

                let audio = if i == cnt_files - 1 {
                    InputMediaAudio::new(file)
                        .title(format!("{} - Part {}", item.title, i + 1))
                        .caption(caption.to_string())
                } else {
                    InputMediaAudio::new(file).title(format!("{} - Part {}", item.title, i + 1))
                };

                InputMedia::Audio(audio)
            })
            .collect::<Vec<InputMedia>>();

        loop {
            match bot
                .send_media_group(chat_id.clone(), media.clone())
                .send()
                .await
            {
                Ok(_) => break,
                Err(e) => {
                    println!("Error: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }

        break;
    }

    Ok(())
}
