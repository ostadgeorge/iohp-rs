use std::{error::Error, fs};

use indicatif::ProgressIterator;
use ioh_scrap::Item;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Compress Audio!");

    let data: Vec<Item> = serde_json::from_str(&fs::read_to_string("data_with_sound.json")?)?;
    let urls: Vec<String> = data
        .iter()
        .filter_map(|item| item.sound_urls.clone())
        .flatten()
        .collect();

    let thread_pool = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(16)
        .enable_io()
        .enable_time()
        .build()?;

    let chunks = urls.chunks(16);
    for chunk in chunks.progress() {
        let mut futs = vec![];

        for url in chunk {
            let fut = thread_pool.spawn(compress_audio(url.clone()));
            futs.push(fut);
        }

        for fut in futs.into_iter() {
            let _ = fut.await?;
        }
    }

    thread_pool.shutdown_background();

    Ok(())
}

async fn compress_audio(url: String) -> Result<(), Box<dyn Error + Send + Sync>> {
    let file_name = ioh_scrap::file_name_by_url(&url, "mp3");
    let file_path = format!("audio/{}", file_name);

    println!("Compressing: {}", file_name);

    // ffmpeg -i file.mp3 -ab 32k -f ogg file.ogg
    let _ = std::process::Command::new("ffmpeg")
        .args(&[
            "-i",
            &file_path,
            "-ab",
            "32k",
            "-f",
            "ogg",
            format!("audio/{}", ioh_scrap::file_name_by_url(&url, "ogg")).as_str(),
        ]).output()?;

    print!("Compressed: {}", file_name);

    Ok(())
}
