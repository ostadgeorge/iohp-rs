use std::{collections::HashMap, error::Error, fs, io};

use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Start Extracting");

    let files = fs::read_dir("data")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    let mut document_json = Vec::new();
    for file in files {
        let content = fs::read_to_string(&file)?;
        println!("Extracting file: {:?}", file);

        let document = scraper::Html::parse_document(&content);
        let selector = &scraper::Selector::parse(".document").unwrap();

        let mut nth = 0;
        while let Some(element) = document.select(selector).nth(nth) {
            let document_counter_selector = &scraper::Selector::parse(".document-counter").unwrap();
            let document_counter = element
                .select(document_counter_selector)
                .next()
                .unwrap()
                .text()
                .collect::<String>();

            let title_selector = &scraper::Selector::parse(".index_title > a").unwrap();
            let title = element
                .select(title_selector)
                .next()
                .unwrap()
                .text()
                .collect::<String>();

            let metadata_selector = &scraper::Selector::parse(".document-metadata").unwrap();
            let metadata_element = element.select(metadata_selector).next();

            let mut metadata: HashMap<String, String> = HashMap::new();
            match metadata_element {
                Some(metadata_element) => {
                    let mut idx = 0;
                    while idx < metadata_element.children().count() / 4 {
                        let key_selector = &scraper::Selector::parse("dt").unwrap();
                        let key = metadata_element
                            .select(key_selector)
                            .nth(idx)
                            .unwrap()
                            .text()
                            .collect::<String>();

                        let value_selector = &scraper::Selector::parse("dd").unwrap();
                        let value = metadata_element
                            .select(value_selector)
                            .nth(idx)
                            .unwrap()
                            .text()
                            .collect::<String>();

                        metadata.insert(key, value);
                        idx += 1;
                    }
                }
                None => {}
            }

            document_json.push(json!({
                "document_counter": document_counter.trim_ascii(),
                "title": title,
                "metadata": metadata,
            }));
            
            nth += 1;
        }
    }

    let json_data = fs::File::create("data.json")?;
    let mut writer = io::BufWriter::new(json_data);

    serde_json::to_writer(&mut writer, &document_json)?;
    
    Ok(())
}
