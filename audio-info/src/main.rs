use structopt::StructOpt;
use std::fs;
use audio_info::AudioInfo;

/// Generate SVG waveform from audio file
#[derive(StructOpt, Clone)]
struct AudioToSvgArgs {
    /// The audio file to process
    #[structopt(long, short)]
    file: String,
    /// Output SVG file path
    #[structopt(long, short = "o")]
    output: String,
}

fn main() {
    let args = AudioToSvgArgs::from_args();
    
    // Create AudioInfo instance and process file
    let mut audio_info = AudioInfo::default();
    match audio_info.update_file(args.file.clone()) {
        Ok(()) => {
            // Write SVG to output file
            match fs::write(&args.output, &audio_info.image) {
                Ok(()) => println!("Successfully wrote SVG to {}", args.output),
                Err(e) => eprintln!("Error writing SVG file: {}", e),
            }
        },
        Err(e) => eprintln!("Error processing audio file: {}", e),
    }
} 