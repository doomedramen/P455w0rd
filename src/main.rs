mod args;
mod words;
mod generator;
mod display;

use clap::Parser;
use args::Args;
use words::get_words;
use generator::{generate_combinations_streaming, GeneratorConfig};

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

    // Create generator configuration
    let config = GeneratorConfig {
        min_len,
        max_len,
        limit: args.limit,
        output_file: args.output.clone(),
        chunk_size: args.chunk_size,
        quiet: args.quiet,
        append: args.append,
    };

    // Generate and write combinations incrementally
    let count = generate_combinations_streaming(&words, &config)?;

    println!("Generated {} passwords to {}", count, args.output);

    Ok(())
}