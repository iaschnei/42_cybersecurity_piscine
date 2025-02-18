use std::path::PathBuf;
use font8x8::{BASIC_FONTS, UnicodeFonts};
use minifb::{Key, Window, WindowOptions};
use clap::Parser;
use image::GenericImageView;

mod metadata;
use metadata::Metadata;

const WIDTH: usize = 840;
const HEIGHT: usize = 460;

const IMAGE_DISPLAY_WIDTH: usize = 600;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about)]
struct Args {
  /// Paths of images to handle
  path: Vec<String>,
}

struct DisplayState {
  window: Window,
  buffer: Vec<u32>,
  images:Vec<ImageData>,
  current_image: usize,
}

struct ImageData {
  buffer: Vec<u32>,
  width: usize,
  height: usize,
  metadata: Metadata,
}

impl ImageData {
  fn from_path(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
    let img = image::open(path)?;
    let dimensions = img.dimensions();
    let path_buf = PathBuf::from(path);
    let metadata = Metadata::from_file(&path_buf)?;

    // Minifb (gui lib) only takes u32 RGBA as argument to display 
    let rgb_img = img.to_rgba8();
    let buffer: Vec<u32> = rgb_img.pixels()
      .map(|p| {
        let r: u32 = p[0] as u32;
        let g: u32 = p[1] as u32;
        let b: u32 = p[2] as u32;
        let a: u32 = p[3] as u32;
        (a << 24) | (r << 16) | (g << 8) | b
      })
      .collect();

      Ok(ImageData {
        buffer,
        width: dimensions.0 as usize,
        height: dimensions.1 as usize,
        metadata,
      })
  }

  // Scale the image to the reserved space
  fn get_scaled_buffer(&self, target_width: usize, target_height: usize) -> Vec<u32>  {
    let mut scaled_buffer = vec![0; target_width * target_height];

    for y in 0..target_height {
      for x in 0..target_width {
        let src_x = (x * self.width) / target_width;
        let src_y = (y * self.height) / target_height;
        let src_idx = src_y * self.width + src_x;
        let dst_idx = y * target_width + x;

        if src_idx < self.buffer.len() {
          scaled_buffer[dst_idx] = self.buffer[src_idx];
        }
      }
    }

    scaled_buffer
  }
}

fn main() {

  let args = Args::parse();

  if check_args(&args) == false {
    eprintln!("Error parsing arguments, supported extensions are : jpg / jpeg / png / gif / bmp");
    return;
  }

  match init_display(&args.path) {
    Ok(display_state) => display_loop(display_state),
    Err(e) => eprintln!("Error initialising display: {}", e),
  }
}

fn check_args(args: &Args) -> bool {

  let valid_extensions = ["jpg", "jpeg", "png", "gif", "bmp"];

  for path in &args.path {
    if let Some(extension) = path.split('.').last() {
      if !valid_extensions.contains(&extension.to_lowercase().as_str()) {
        return false
      }
    }
  }

  true
}

fn init_display(paths: &[String]) -> Result<DisplayState, Box<dyn std::error::Error>> {
  let window = Window::new(
    "Scorpion",
    WIDTH,
    HEIGHT,
    WindowOptions::default(),
  )?;

  let mut images = Vec::new();
  for path in paths {
    images.push(ImageData::from_path(path)?);
  }

  Ok(DisplayState {
    window,
    buffer: vec![0; WIDTH * HEIGHT],
    images,
    current_image: 0,
  })
}

fn display_loop(mut state: DisplayState) {
  state.window.set_target_fps(30);

  while state.window.is_open() && !state.window.is_key_down(Key::Escape) {
    //Clear buffer
    for i in state.buffer.iter_mut() {
      *i = 0;
    }

    if !state.images.is_empty() {
      let image = &state.images[state.current_image];

      let target_height = HEIGHT;
      let target_width = IMAGE_DISPLAY_WIDTH;

      let scaled_image = image.get_scaled_buffer(target_width, target_height);

      for y in 0..target_height {
        for x in 0..target_width {
          let src_idx = y * target_width + x;
          let dst_idx = y * WIDTH + x;
          if src_idx < scaled_image.len() && dst_idx < state.buffer.len() {
            state.buffer[dst_idx] = scaled_image[src_idx];
          }
        }
      }

      let metadata = &image.metadata;
      let metadata_x = IMAGE_DISPLAY_WIDTH + 10;
      let text_color = 0xFFFFFFFF;

      let metadata_texts = [
        format!("Size: {} bytes", metadata.size_bytes),
        format!("Created: {}", metadata.created.map_or("Unknown".to_string(), |dt| dt.to_string())),
        format!("Dimensions: {}x{}", metadata.width, metadata.height),
        format!("Color Type: {}", metadata.color_type),
        format!("Camera Model: {}", metadata.camera_model.as_deref().unwrap_or("Unknown")),
      ];

      for (i, text) in metadata_texts.iter().enumerate() {
        draw_text(&mut state.buffer, metadata_x, 10 + i * 20, text, text_color);
      }

      if state.window.is_key_pressed(Key::Right, minifb::KeyRepeat::No) {
        state.current_image = (state.current_image + 1) % state.images.len();
      }
      if state.window.is_key_pressed(Key::Left, minifb::KeyRepeat::No) {
        state.current_image = if state.current_image == 0 {
          state.images.len() - 1
        } else {
          state.current_image - 1
        };
      }
    }


    state.window
      .update_with_buffer(&state.buffer, WIDTH, HEIGHT)
      .unwrap();
  }
}

fn draw_text(buffer: &mut [u32], x: usize, y: usize, text: &str, color: u32) {
  for (char_idx, c) in text.chars().enumerate() {
    let char_x = x + (char_idx * 8);
    if char_x >= WIDTH {
      break;
    }

    if let Some(glyph) = BASIC_FONTS.get(c) {
      for (row_idx, row) in glyph.iter().enumerate() {
        let buffer_y = y + row_idx;
        if buffer_y >= HEIGHT {
            break;
        }

        for bit_idx in 0..8 {
          let buffer_x = char_x + bit_idx;
          if buffer_x >= WIDTH {
              break;
          }

          if *row & (1 << bit_idx) != 0 {
            let idx = buffer_y * WIDTH + buffer_x;
            if idx < buffer.len() {
              buffer[idx] = color;
            }
          }
        }
      }
    }
  }
}