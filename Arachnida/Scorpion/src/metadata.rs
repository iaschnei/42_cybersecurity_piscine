use std::path::PathBuf;
use chrono::{DateTime, Utc};
use image::GenericImageView;

#[derive(Debug)]
pub struct Metadata {

  pub size_bytes: u64,
  pub created: Option<DateTime<Utc>>,

  pub width: u32,
  pub height: u32,
  pub color_type: String,

  pub camera_model: Option<String>
}

impl Metadata {
  // Create a new instance with a file at a given path
  pub fn from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {

    let fs_metadata = std::fs::metadata(path)?;

    let img = image::open(path)?;
    let (width, height) = img.dimensions();

    let mut metadata = Metadata {
      size_bytes: fs_metadata.len(),
      created: fs_metadata.created().ok().map(DateTime::from),
      width,
      height,
      color_type: format!("{:?}", img.color()),
      camera_model: None,
    };

    //Try to get EXIF data (most likely for jpg/jpeg)
    if let Ok(file) = std::fs::File::open(path) {
      if let Ok(exif) = exif::Reader::new()
        .read_from_container(&mut std::io::BufReader::new(file)) 
      {
        if let Some(model) = exif.get_field(exif::Tag::Model, exif::In::PRIMARY) {
          metadata.camera_model = Some(model.display_value().to_string());
        }
      }
    }

    Ok(metadata)
  }
}