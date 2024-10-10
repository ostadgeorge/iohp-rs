use ioh_scrap::Item;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs,
    io::{self, Write},
    sync::Arc,
};
use tokio::sync::Mutex;

use indicatif::ProgressIterator;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Context {
    cnt_ended: usize,
    cnt_started: usize,
    items: Vec<Item>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Start Scraping Audio Items");

    let data: Vec<Item> = serde_json::from_str(&fs::read_to_string("data.json")?)?;
    let thread_pool = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(16)
        .enable_io()
        .enable_time()
        .build()?;

    let ctx = Arc::new(Mutex::new(Context {
        cnt_ended: 0,
        cnt_started: 0,
        items: vec![],
    }));

    let chunks = data.chunks(50);

    for chunk in chunks.progress() {
        let mut futs = vec![];

        for item in chunk {
            let ctx = ctx.clone();
            let fut = thread_pool.spawn(process_item(item.clone(), ctx));
            futs.push(fut);
        }

        for fut in futs.into_iter() {
            let _ = fut.await?;
        }
    }

    thread_pool.shutdown_background();

    let json_data = fs::File::create("data_with_sound.json")?;
    let mut writer = io::BufWriter::new(json_data);

    writer.write_all(
        serde_json::to_string_pretty(&ctx.lock().await.items.to_vec())
            .unwrap()
            .as_bytes(),
    )?;
    Ok(())
}

async fn process_item(
    item: Item,
    ctx: Arc<Mutex<Context>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("processing {} {} ...", item.document_counter, item.title);

    let url = match item.metadata.get("Audio") {
        Some(url) => url,
        None => {
            println!("Audio not found for {}", item.title);
            return Ok(());
        }
    };

    let mut client = get_client(3).await?;
    let mut req = client.get(url).header("User-Agent", "Mozilla/5.0");

    let (cookie, location) = loop {
        let res = req.try_clone().unwrap().send().await;
        match res {
            Ok(res) => {
                if res.status().is_redirection() || res.status().is_success() {
                    let cookie = match res.headers().get("set-cookie") {
                        Some(cookie) => cookie.to_str().unwrap(),
                        None => "",
                    };
                    let location = res.url().clone();
                    break (cookie.to_string(), location);
                } else {
                    println!("Error: {:?}", res.status());
                    std::thread::sleep(std::time::Duration::from_secs(10));

                    client = get_client(3).await?;
                    req = client.get(url).header("User-Agent", "Mozilla/5.0");
                    continue;
                }
            }
            Err(err) => {
                println!("Error: {:?}", err);
                std::thread::sleep(std::time::Duration::from_secs(10));

                client = get_client(3).await?;
                req = client.get(url).header("User-Agent", "Mozilla/5.0");
                continue;
            }
        };
    };

    let mut req = client
        .get(location.clone())
        .header("User-Agent", "Mozilla/5.0")
        .header("cookie", cookie.clone());

    let res = loop {
        let res = req.try_clone().unwrap().send().await;
        match res {
            Ok(res) => {
                if res.status().is_success() {
                    break res;
                } else {
                    println!("Error: {:?}", res.status());
                    std::thread::sleep(std::time::Duration::from_secs(10));

                    client = get_client(3).await?;
                    req = client
                        .get(location.clone())
                        .header("User-Agent", "Mozilla/5.0")
                        .header("cookie", cookie.clone());
                    continue;
                }
            }
            Err(err) => {
                println!("Error: {:?}", err);
                std::thread::sleep(std::time::Duration::from_secs(10));

                client = get_client(3).await?;
                req = client
                    .get(location.clone())
                    .header("User-Agent", "Mozilla/5.0")
                    .header("cookie", cookie.clone());
                continue;
            }
        };
    };

    let mut sound_urls = vec![];
    let content = &res.text().await?;
    content.split("\n").for_each(|line| {
        if line.contains(
            r#"file: "https://mps.lib.harvard.edu/vod/_definst_/smil:s3/drs-delivery-prod/"#,
        ) {
            let url = line.split(r#"file: ""#).collect::<Vec<&str>>()[1].trim_end_matches("\",");
            println!("URL: {:?}", url);
            sound_urls.push(url.to_string());
        }
    });

    ctx.lock().await.items.push(Item {
        document_counter: item.document_counter,
        title: item.title,
        metadata: item.metadata,
        sound_urls: Some(sound_urls),
    });

    ctx.lock().await.cnt_ended += 1;
    println!("Done till now: {:?}", ctx.lock().await.cnt_ended);

    std::thread::sleep(std::time::Duration::from_secs(5));

    Ok(())
}

async fn get_client(num_redirect: usize) -> Result<reqwest::Client, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::custom(move |attempt| {
            if attempt.previous().len() > num_redirect {
                attempt.stop()
            } else {
                attempt.follow()
            }
        }))
        .tls_sni(false)
        .danger_accept_invalid_certs(true)
        .http2_keep_alive_timeout(std::time::Duration::from_secs(20))
        .timeout(std::time::Duration::from_secs(120))
        .connection_verbose(true)
        .build()?;

    Ok(client)
}
