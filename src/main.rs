mod args;
mod words;
mod generator;
mod display;
mod combinatorics;

use clap::Parser;
use args::Args;
use words::get_words;
use generator::{generate_combinations_streaming, GeneratorConfig};
use combinatorics::{calculate_total_combinations, CombinatorialConfig, format_file_size, format_combination_count};

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

    // Calculate combinatorial analysis
    let combinatorial_config = CombinatorialConfig {
        max_words: args.get_max_words(),
        include_special_chars: !args.no_special_chars,
    };

    let analysis = calculate_total_combinations(&words, &combinatorial_config)?;

    // Display analysis
    println!("\nCombinatorial Analysis:");
    println!("  Total combinations: {} (exact: {})", format_combination_count(analysis.total_combinations), analysis.total_combinations);
    println!("  Estimated file size: {}", format_file_size(analysis.estimated_file_size_bytes));
    println!("  Word permutations: {}", format_combination_count(analysis.breakdown.word_permutations));
    println!("  Leet variants: {}", format_combination_count(analysis.breakdown.leet_variants));
    println!("  Case variations: {}", analysis.breakdown.case_variants);
    println!("  Special char variants: {}", format_combination_count(analysis.breakdown.special_char_variants));

    println!("\nBreakdown by word count:");
    for breakdown in &analysis.breakdown.by_word_count {
        println!("  {} words: {} (exact: {}) (avg length: {:.1})",
                 breakdown.word_count,
                 format_combination_count(breakdown.combinations),
                 breakdown.combinations,
                 breakdown.average_length);
    }

    // Require confirmation unless --force is used
    if !args.force && analysis.total_combinations > 1_000_000 {
        println!("\n⚠️  Warning: This will generate {} passwords (estimated size: {})",
                 format_combination_count(analysis.total_combinations),
                 format_file_size(analysis.estimated_file_size_bytes));

        print!("Do you want to continue? [y/N]: ");
        use std::io::Write;
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        let input = input.trim().to_lowercase();
        if input != "y" && input != "yes" {
            println!("Operation cancelled.");
            std::process::exit(0);
        }
    }

    // Create generator configuration
    let config = GeneratorConfig {
        min_len,
        max_len,
        limit: args.limit,
        output_file: args.output.clone(),
        chunk_size: args.chunk_size,
        quiet: args.quiet,
        append: args.append,
        max_words: args.get_max_words(),
        no_special_chars: args.no_special_chars,
    };

    // Generate and write combinations incrementally
    let count = generate_combinations_streaming(&words, &config)?;

    println!("Generated {} passwords to {}", count, args.output);

    // Verify the count matches our calculation
    if count != analysis.total_combinations as usize && analysis.total_combinations != u64::MAX {
        println!("⚠️  Generated count ({}) differs from calculated count ({})",
                 count, format_combination_count(analysis.total_combinations));
    }

    Ok(())
}