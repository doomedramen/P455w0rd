mod args;
mod words;
mod generator;
mod display;

use clap::Parser;
use args::Args;
use words::get_words;
use generator::generate_combinations_streaming;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Set length constraints for WPA2
    let (min_len, max_len) = args.get_length_constraints();

    // Get words from input
    let words = get_words(&args)?;

    if words.is_empty() {
        eprintln!("No words provided. Use --input file or provide words as arguments.");
        std::process::exit(1);
    }

    println!("Processing {} words...", words.len());

    // Generate and write combinations incrementally
    let count = generate_combinations_streaming(&words, min_len, max_len, args.limit, &args.output, args.chunk_size, args.quiet, args.append)?;

    println!("Generated {} passwords to {}", count, args.output);

    Ok(())
}