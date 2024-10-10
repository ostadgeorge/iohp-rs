use std::{
    error::Error,
    fs,
    io::{self, Write},
};
use thirtyfour::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // https://curiosity.lib.harvard.edu/iranian-oral-history-project

    println!("Start Scraping");

    let mut caps = DesiredCapabilities::firefox();
    // caps.add_arg("--headless")?;
    // caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;

    let driver = WebDriver::new("http://127.0.0.1:4444", caps).await?;

    let mut page = 1;
    loop {
        println!("Scraping page: {}", page);
        let url = format!("https://curiosity.lib.harvard.edu/iranian-oral-history-project/catalog?page={}&per_page=96&search_field=all_fields", page);
        driver.goto(&url).await?;
        
        let file = fs::File::create(format!("data/page-{}.html", page))?;
        let mut writer = io::BufWriter::new(file);
        writer.write_all(driver.source().await?.as_bytes())?;

        page += 1;
        if driver.find_all(By::ClassName("document")).await?.len() == 0 {
            break;
        };
    }

    driver.quit().await?;

    Ok(())
}
