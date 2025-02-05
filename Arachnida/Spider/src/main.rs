use clap::Parser;
use scraper::{Html, Selector};
use url::Url;
use std::path::Path;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use futures::future::join_all;
use std::time::Duration;

type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
  /// URL of the website to scrap
  url: String,

  /// Optional recursivity setting
  #[arg(short = 'r', long)]
  r: bool,

  /// Optional depth of recursivity, default = 5
  #[arg(short = 'l', long, value_name = "NUMBER")]
  l: Option<u32>,

  /// Optional save path, default = ./data/
  #[arg(short = 'p', long, value_name = "PATH")]
  p: Option<String>,
}

#[derive(Clone)]
struct Settings {
  recursive: bool,
  depth: u32,
  path: String,
}

#[tokio::main]
async fn main() -> Result<(), BoxedError> {

  let args = Args::parse();

  let settings = Settings {
    recursive: args.r,
    depth: args.l.unwrap_or(5),
    path: args.p.unwrap_or_else(|| "./data".to_string()),
  };

  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(10))
    .build()?;
  
  let visited_urls = Arc::new(Mutex::new(HashSet::new()));

  match get_page_content(&client, args.url, settings, 0, visited_urls).await {
    Ok(_) => println!("Scraping completed successfully!"),
    Err(e) => eprintln!("Error during scraping: {}", e),
  }

  Ok(())
}

async fn get_page_content(
    client: &reqwest::Client,
    url: String,
    settings: Settings,
    current_depth: u32,
    visited_urls: Arc<Mutex<HashSet<String>>>
  ) -> Result<(), BoxedError> {


  if (settings.recursive && current_depth > settings.depth) || 
    (!settings.recursive && current_depth >= 1) {
    return Ok(());
  }

  // Check if URL was already visited
  if !should_process_url(&url, &visited_urls).await? {
      return Ok(());
  }

  println!("Processing {} (depth: {})", url, current_depth);

  // Fetch and parse page content
  let (imgs, links) = match fetch_and_parse_page(client, &url).await {
    Ok((i, l)) => (i, l),
    Err(e) => {
      eprintln!("Error processing {}: {}", url, e);
      return Ok(());  // Continue with other URLs even if one fails
    }
  };

  // Download images
  for img_url in &imgs {
    if let Err(e) = download_image(client, img_url, &settings.path).await {
      eprintln!("Failed to download {}: {}", img_url, e);
      continue;  // Continue with other images even if one fails
    }
  }

  // Handle recursion
  if settings.recursive && current_depth < settings.depth {
    process_links(client, links, settings, current_depth, visited_urls).await?;
  }

  Ok(())
}

async fn should_process_url(url: &str, visited_urls: &Arc<Mutex<HashSet<String>>>) -> Result<bool, BoxedError> {
  let mut visited = visited_urls.lock().await;
  if visited.contains(url) {
    return Ok(false);
  }
  visited.insert(url.to_string());
  Ok(true)
}

async fn fetch_and_parse_page(client: &reqwest::Client, url: &str) -> Result<(Vec<String>, Vec<String>), BoxedError> {
  let base_url = Url::parse(url)?;
  let page_content = client.get(url)
      .send()
      .await?
      .text()
      .await?;

  let document = Html::parse_document(&page_content);
  let img_selector = Selector::parse("img, .img-responsive").unwrap();
  let link_selector = Selector::parse("a").unwrap();
  let valid_extensions = ["jpg", "jpeg", "png", "gif", "bmp"];

  let mut imgs = Vec::new();
  let mut links = Vec::new();

  // Extract images
  for element in document.select(&img_selector) {
    if let Some(src) = element.value().attr("src") {
      // Skip data URLs and base64 encoded images
      if !src.starts_with("data:") {
        if has_valid_extension(src, &valid_extensions) {
          if let Ok(absolute_url) = base_url.join(src) {
            imgs.push(absolute_url.to_string());
          }
        }
      }
    }
  }

  // Extract links
  for element in document.select(&link_selector) {
    if let Some(href) = element.value().attr("href") {
      if let Ok(absolute_url) = base_url.join(href) {
        if absolute_url.host() == base_url.host() {
          links.push(absolute_url.to_string());
        }
      }
    }
  }

  Ok((imgs, links))
}


async fn download_image(client: &reqwest::Client, img_url: &str, base_path: &str) -> Result<(), BoxedError> {
  let parsed_url = Url::parse(img_url)?;
  let filename = format!("{}/{}",
    base_path.trim_end_matches('/'),
    parsed_url.path().split('/').last().unwrap_or("unknown.jpg")
  );

  // Create directories if they don't exist
  if let Some(parent) = Path::new(&filename).parent() {
    tokio::fs::create_dir_all(parent).await?;
  }

  let response = client.get(img_url)
    .send()
    .await?;
  
  let bytes = response.bytes().await?;
  let mut file = File::create(&filename).await?;
  file.write_all(&bytes).await?;
  
  println!("Downloaded: {}", filename);
  Ok(())
}

async fn process_links(
  client: &reqwest::Client,
  links: Vec<String>,
  settings: Settings,
  current_depth: u32,
  visited_urls: Arc<Mutex<HashSet<String>>>
) -> Result<(), BoxedError> {

  let futures: Vec<_> = links
    .into_iter()
    .map(|link| {
      let settings = settings.clone();
      let visited_urls = Arc::clone(&visited_urls);
      get_page_content(client, link, settings, current_depth + 1, visited_urls)
    })
    .collect();

  let results = join_all(futures).await;
  for result in results {
    if let Err(e) = result {
      eprintln!("Error in recursive fetch: {}", e);
    }
  }
  Ok(())
}

fn has_valid_extension(url: &str, valid_extensions: &[&str]) -> bool {
  if let Some(extension) = url.split('.').last() {
    return valid_extensions.contains(&extension.to_lowercase().as_str());
  }
  false
}
