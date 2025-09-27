use crate::display::update_status_display;
use crate::words::create_word_variants;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::time::{Duration, Instant};
use itertools::Itertools;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    pub min_len: usize,
    pub max_len: usize,
    pub limit: usize,
    pub output_file: String,
    pub chunk_size: usize,
    pub quiet: bool,
    pub append: bool,
    pub max_words: usize,
    pub no_special_chars: bool,
}

pub fn generate_combinations_streaming(
    words: &[String],
    config: &GeneratorConfig,
) -> Result<usize, Box<dyn std::error::Error>> {
    // Remove duplicates
    let unique_words: Vec<String> = words.iter().cloned().collect::<std::collections::HashSet<_>>().into_iter().collect();
    let n = unique_words.len();

    // Use atomic file operations with temporary file for safety
    let (file, temp_path) = if config.append {
        (OpenOptions::new().create(true).append(true).open(&config.output_file)?, None)
    } else {
        let temp_path = format!("{}.tmp.{}", config.output_file, std::process::id());
        let file = File::create(&temp_path)?;
        (file, Some(temp_path))
    };
    let mut writer = BufWriter::new(file);
    let mut total_count = 0;
    let mut chunk_buffer = Vec::with_capacity(config.chunk_size);

    let start_time = Instant::now();
    let mut last_update = Instant::now();
    let mut first_display = true;

    // Special characters for padding
    let special_chars = ['!', '@', '#', '$', '%'];

    // Generate all permutations for each word count from 1 to max_words
    for k in 1..=config.max_words.min(n) {
        // Get all permutations of k distinct words
        for word_indices in (0..n).permutations(k) {
            // Get the actual words for this permutation
            let perm_words: Vec<&String> = word_indices.iter().map(|&i| &unique_words[i]).collect();

            // Generate all combinations for this word permutation
            generate_word_combinations(
                &perm_words,
                &special_chars,
                config,
                &mut chunk_buffer,
                &mut total_count,
                &mut writer,
                &start_time,
                &mut last_update,
                &mut first_display,
                &unique_words,
                k,
            )?;

            // Check limit
            if config.limit > 0 && total_count >= config.limit {
                break;
            }
        }

        if config.limit > 0 && total_count >= config.limit {
            break;
        }
    }

    // Write remaining combinations
    if !chunk_buffer.is_empty() {
        write_chunk(&mut writer, &chunk_buffer)?;
        total_count += chunk_buffer.len();
    }

    writer.flush()?;
    drop(writer);

    // Atomic rename
    if let Some(temp_path) = temp_path {
        std::fs::rename(&temp_path, &config.output_file)?;
    }

    Ok(total_count)
}

fn generate_word_combinations(
    words: &[&String],
    special_chars: &[char],
    config: &GeneratorConfig,
    chunk_buffer: &mut Vec<String>,
    total_count: &mut usize,
    writer: &mut BufWriter<File>,
    start_time: &Instant,
    last_update: &mut Instant,
    first_display: &mut bool,
    all_words: &[String],
    current_word_count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // Generate all leet + case variants for each word
    let word_variants: Vec<Vec<String>> = words
        .par_iter()
        .map(|word| create_word_variants(word))
        .collect();

    // Generate cartesian product of all word variants
    let base_combinations = generate_cartesian_product(&word_variants);

    // Apply length filtering and special character padding
    for base_combo in base_combinations {
        // Check length constraints
        if base_combo.len() < config.min_len || base_combo.len() > config.max_len {
            continue;
        }

        // Add the base combination (no special chars)
        add_to_buffer(base_combo.clone(), chunk_buffer, total_count, writer, config, start_time, last_update, first_display, all_words, current_word_count)?;

        // Add special character variations if enabled
        if !config.no_special_chars {
            add_special_char_variations(&base_combo, special_chars, config, chunk_buffer, total_count, writer, start_time, last_update, first_display, all_words, current_word_count)?;
        }

        if config.limit > 0 && *total_count >= config.limit {
            break;
        }
    }

    Ok(())
}

fn generate_cartesian_product(word_variants: &[Vec<String>]) -> Vec<String> {
    if word_variants.is_empty() {
        return vec![];
    }

    let mut result = vec![String::new()];
    for variants in word_variants {
        let mut new_result = Vec::new();
        for base in result {
            for variant in variants {
                new_result.push(format!("{}{}", base, variant));
            }
        }
        result = new_result;
    }
    result
}

fn add_special_char_variations(
    base_combo: &str,
    special_chars: &[char],
    config: &GeneratorConfig,
    chunk_buffer: &mut Vec<String>,
    total_count: &mut usize,
    writer: &mut BufWriter<File>,
    start_time: &Instant,
    last_update: &mut Instant,
    first_display: &mut bool,
    all_words: &[String],
    current_word_count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let n = special_chars.len();

    // Single prefix
    for &special in special_chars {
        let padded = format!("{}{}", special, base_combo);
        if padded.len() >= config.min_len && padded.len() <= config.max_len {
            add_to_buffer(padded, chunk_buffer, total_count, writer, config, start_time, last_update, first_display, all_words, current_word_count)?;
        }
    }

    // Single suffix
    for &special in special_chars {
        let padded = format!("{}{}", base_combo, special);
        if padded.len() >= config.min_len && padded.len() <= config.max_len {
            add_to_buffer(padded, chunk_buffer, total_count, writer, config, start_time, last_update, first_display, all_words, current_word_count)?;
        }
    }

    // Multiple special chars (2 to n)
    for k in 2..=n {
        for special_combo in special_chars.iter().combinations(k) {
            for perm_special in special_combo.iter().permutations(k) {
                // Prefix
                let mut padded = String::new();
                for &&special in &perm_special {
                    padded.push(*special);
                }
                padded.push_str(base_combo);
                if padded.len() >= config.min_len && padded.len() <= config.max_len {
                    add_to_buffer(padded, chunk_buffer, total_count, writer, config, start_time, last_update, first_display, all_words, current_word_count)?;
                }

                // Suffix
                let mut padded = base_combo.to_string();
                for &&special in &perm_special {
                    padded.push(*special);
                }
                if padded.len() >= config.min_len && padded.len() <= config.max_len {
                    add_to_buffer(padded, chunk_buffer, total_count, writer, config, start_time, last_update, first_display, all_words, current_word_count)?;
                }
            }
        }
    }

    Ok(())
}

fn add_to_buffer(
    password: String,
    chunk_buffer: &mut Vec<String>,
    total_count: &mut usize,
    writer: &mut BufWriter<File>,
    config: &GeneratorConfig,
    start_time: &Instant,
    last_update: &mut Instant,
    first_display: &mut bool,
    all_words: &[String],
    current_word_count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    chunk_buffer.push(password);

    if chunk_buffer.len() >= config.chunk_size {
        write_chunk(writer, chunk_buffer)?;
        *total_count += chunk_buffer.len();
        chunk_buffer.clear();

        // Update status display
        if !config.quiet && (*first_display || last_update.elapsed() >= Duration::from_secs(2)) {
            update_status_display(*total_count, start_time, &config.output_file, all_words, current_word_count, *first_display, 0);
            *last_update = Instant::now();
            *first_display = false;
        }
    }

    Ok(())
}

fn write_chunk(writer: &mut BufWriter<File>, combinations: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    for combination in combinations {
        writeln!(writer, "{}", combination)?;
    }
    Ok(())
}