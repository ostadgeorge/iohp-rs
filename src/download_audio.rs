use std::{error::Error, fs};

use indicatif::ProgressIterator;
use ioh_scrap::Item;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Start Downloading Audio");

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

    println!("Count Audio: {}", urls.len());

    let chunks = urls.chunks(16);
    for chunk in chunks.progress() {
        let mut futs = vec![];

        for url in chunk {
            let fut = thread_pool.spawn(download_audio(url.clone()));
            futs.push(fut);
        }

        for fut in futs.into_iter() {
            let _ = fut.await?;
        }
    }

    thread_pool.shutdown_background();

    Ok(())
}

async fn download_audio(url: String) -> Result<(), Box<dyn Error + Send + Sync>> {
    let file_name = ioh_scrap::file_name_by_url(&url);
    let file_path = format!("audio/{}", file_name);

    println!("Downloading: {} | Url: {}", file_name, url);

    if fs::metadata("audio").is_err() {
        fs::create_dir("audio")?;
    }

    if fs::metadata(&file_path).is_ok() {
        println!("Skip: {}", file_name);
        return Ok(());
    }

    // ffmpeg -protocol_whitelist file,http,https,tcp,tls,crypto -i "url" -c copy "file_name"
    let _ = std::process::Command::new("ffmpeg")
        .args(&[
            "-protocol_whitelist",
            "file,http,https,tcp,tls,crypto",
            "-i",
            &url,
            "-c",
            "copy",
            &file_path,
        ])
        .output()?;

    println!("Downloaded: {}", file_name);

    Ok(())
}
