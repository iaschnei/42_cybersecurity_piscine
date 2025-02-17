use minifb::{Key, Window, WindowOptions};
use clap::Parser;
use image_meta::MetaData;

const WIDTH: usize = 840;
const HEIGHT: usize = 460;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
  /// Paths of images to handle
  path: Vec<String>,

  /// Optional modify settings
  #[arg(short = 'x', long)]
  x: bool,
}

fn main() {

  let args = Args::parse();

  let modify = args.x;

  if check_args(args) == false {
    eprintln!("Error parsing arguments, supported extensions are : jpg / jpeg / png / gif / bmp");
    return;
  }



  display_loop();

}

fn check_args(args: Args) -> bool {

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

// TODO : separate init from actual loop
fn display_loop() {

  let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

  let mut window = Window::new(
    "Scorpion",
    WIDTH,
    HEIGHT,
    WindowOptions::default(),
  )
  .unwrap_or_else(|e| {
    panic!("{}", e);
  });

  window.set_target_fps(30);

  while window.is_open() && !window.is_key_down(Key::Escape) {

    for i in buffer.iter_mut() {
      //TODO : Handle mouse inputs and all
      *i = 0;
    }

    window
      .update_with_buffer(&buffer, WIDTH, HEIGHT)
      .unwrap();
  }
}