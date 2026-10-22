use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use error_trace::toplevel;
use humanlog::{DebugMode, HumanLogger};
use image::{GrayAlphaImage, ImageFormat, ImageReader, RgbaImage};
use log::error;


#[derive(Parser)]
struct Args {
    #[clap(name = "INPUT")]
    input:     PathBuf,
    #[clap(short, long, default_value = "0.5")]
    threshold: f64,
    #[clap(short, long, default_value = "./image.png")]
    output:    PathBuf,
}


fn main() -> ExitCode {
    let args = Args::parse();
    if let Err(err) = HumanLogger::terminal(DebugMode::HumanFriendly).init() {
        eprintln!("WARNING: Failed to setup logger: {err}")
    }

    // Load the file with image
    let image = match ImageReader::open(&args.input) {
        Ok(image) => match image.decode() {
            Ok(image) => image,
            Err(err) => {
                error!("{}", toplevel!(("Failed to decode image {:?}", args.input), err));
                return ExitCode::FAILURE;
            },
        },
        Err(err) => {
            error!("{}", toplevel!(("Failed to open image {:?}", args.input), err));
            return ExitCode::FAILURE;
        },
    };

    // Turn it into black-and-white
    let image: GrayAlphaImage = image.into_luma_alpha8();

    // Keep all pixels on the threshold, alpha all the rest
    let dims = image.dimensions();
    let image: RgbaImage = RgbaImage::from_vec(
        dims.0,
        dims.1,
        image
            .pixels()
            .map(|p| if p.0[0] >= (255.0 * args.threshold) as u8 { [0x03, 0x56, 0x9D, 0xFF].into_iter() } else { [0, 0, 0, 0].into_iter() })
            .flatten()
            .collect::<Vec<u8>>(),
    )
    .unwrap();

    // Write it
    if let Err(err) = image.save_with_format(&args.output, ImageFormat::Png) {
        error!("{}", toplevel!(("Failed to save image to {:?}", args.output), err));
        return ExitCode::FAILURE;
    }

    //
    println!("Done.");
    ExitCode::SUCCESS
}
