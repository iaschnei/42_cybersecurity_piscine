use clap::Parser;
use scraper::{Html, Selector};
use ureq::Agent;
use std::collections::HashSet;
use std::vec::Vec;
use url::Url;
use std::fs;
use std::path::Path;

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

fn main() {

  let args = Args::parse();

  let settings = Settings {
    recursive: args.r,
    depth: args.l.unwrap_or(5),
    path: args.p.unwrap_or("./data".to_string()),
  };

  let client: Agent = Agent::new();

  let already_visited = HashSet::<String>::new();

  if let Err(e) = get_page_images(&client, args.url, settings, 0, already_visited) {
    eprintln!("Error getting images from page: {}", e);
  }

}

fn get_page_images(
  client: &ureq::Agent,
  url: String,
  settings: Settings,
  current_depth: u32,
  mut already_visited: HashSet<String>
) -> Result<(), BoxedError>{

  if !settings.recursive && current_depth >= 1 ||
    settings.recursive && current_depth > settings.depth {
    return Ok(()) 
  }

  if already_visited.contains(&url) {
    return Ok(())
  }
  already_visited.insert(url.clone());

  println!("Processing {} (depth: {})", url, current_depth);

  let (imgs, links) = match get_imgs_and_links(client, &url) {
    Ok((i, l)) => (i, l),
    Err(e) => {
      eprintln!("Error processing {url}: {e}");
      return Ok(());
    }
  };

  for img_url in &imgs {
    if let Err(e) = download_image(client, img_url, &settings.path) {
      eprintln!("Failed to download {}: {}", img_url, e);
      continue;
    }
  }

  if settings.recursive && current_depth < settings.depth {
    process_links(client, links, settings, current_depth, already_visited)?;
  }

  Ok(())
}

fn get_imgs_and_links(client: &ureq::Agent, url: &str) -> Result<(Vec<String>, Vec<String>), BoxedError> {

  let base_url = Url::parse(url)?;
  let page_body = match client.get(url).call() {
    Ok(response) => response,
    Err(ureq::Error::Status(code,_)) => {
      eprintln!("Server returned unexpected status: {}", code);
      return Ok((Vec::new(), Vec::new()));
    }
    Err(_) => {
      eprintln!("Unknown error"); 
      return Ok((Vec::new(), Vec::new()));
    }
  };

  let body = page_body.into_string()?;
  let parsed_body = Html::parse_document(&body);

  let img_selector = Selector::parse("img").unwrap();
  let link_selector = Selector::parse("a").unwrap();
  let valid_extensions = ["jpg", "jpeg", "png", "gif", "bmp"];

  let mut imgs = Vec::new();
  let mut links = Vec::new();

  for element in parsed_body.select(&img_selector) {
    if let Some(src) = element.value().attr("src") {
      if has_valid_extension(src, &valid_extensions) {
        if let Ok(absolute_url) = base_url.join(src) {
          imgs.push(absolute_url.to_string());
        }
      }
    }
  }

  for element in parsed_body.select(&link_selector) {
    if let Some(href) = element.value().attr("href") {
      links.push(href.to_string());
    }
  }

  Ok((imgs, links))
}

fn has_valid_extension(url: &str, valid_extensions: &[&str]) -> bool {
  if let Some(extension) = url.split('.').last() {
    return valid_extensions.contains(&extension.to_lowercase().as_str());
  }
  false
}

fn download_image(client: &ureq::Agent, img_url: &str, base_path: &str) -> Result<(), BoxedError> {

  let parsed_url = Url::parse(img_url)?;
  let filename = format!(
    "{}/{}",
    base_path.trim_end_matches('/'),
    parsed_url.path().split('/').last().unwrap_or("unknown.jpg")
  );

  // Create dir
  if let Some(parent) = Path::new(&filename).parent() {
    fs::create_dir_all(parent)?;
  }

  let mut bytes = Vec::new();
  client.get(img_url)
    .call()?
    .into_reader()
    .read_to_end(&mut bytes)?;

  std::fs::write(&filename, bytes)?;

  println!("Downloaded: {}", filename);
  Ok(())
}

fn process_links (
  client: &ureq::Agent,
  links: Vec<String>,
  settings: Settings,
  current_depth: u32,
  already_visited: HashSet<String>
) ->Result<(), BoxedError> {

  for link in links {
    let settings = settings.clone();
    let already_visited = already_visited.clone();
    if let Err(e) = get_page_images(client, link, settings, current_depth + 1, already_visited) {
      eprintln!("Error in recursive fetch: {}", e);
    }
  }

  Ok(())
}