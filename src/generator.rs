use crate::display::update_status_display;
use crate::words::create_word_variants;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::time::{Duration, Instant};
use std::collections::HashSet;
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
}

// Resource limits to prevent overflow and excessive memory usage
const MAX_WORD_VARIANTS: usize = 1000;
const MAX_COMBINATION_SIZE: usize = 4;  // Reduced from 6 to prevent explosion
const MAX_MEMORY_USAGE_MB: usize = 500;
const MAX_HASHSET_SIZE: usize = 1_000_000;

pub fn generate_combinations_streaming(
    words: &[String],
    config: &GeneratorConfig,
) -> Result<usize, Box<dyn std::error::Error>> {

    // Use atomic file operations with temporary file for safety
    let (file, temp_path) = if config.append {
        // For append mode, write directly to the file
        (OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config.output_file)?, None)
    } else {
        // For overwrite mode, use temporary file for atomic operation
        let temp_path = format!("{}.tmp.{}", config.output_file, std::process::id());
        let file = File::create(&temp_path)?;
        (file, Some(temp_path))
    };
    let mut writer = BufWriter::new(file);
    let mut total_count = 0;
    let mut chunk_buffer = Vec::with_capacity(config.chunk_size);
    let mut seen_passwords = HashSet::with_capacity(config.chunk_size * 2);

    // Reserve string capacity to reduce allocations
    chunk_buffer.reserve(config.chunk_size);

    let start_time = Instant::now();
    let mut last_update = Instant::now();
    let mut first_display = true;

    // Create l33t variants for each word with limits using parallel processing
    let word_variants: Vec<Vec<String>> = words
        .par_iter() // Use parallel iterator
        .map(|word| {
            let variants = create_word_variants(word);
            // Limit variants per word to prevent explosion
            if variants.len() > MAX_WORD_VARIANTS {
                variants.into_iter().take(MAX_WORD_VARIANTS).collect()
            } else {
                variants
            }
        })
        .collect();

    // Early termination if word list is too large
    if words.len() > 50 {
        eprintln!("Warning: Large word list ({} words) may cause excessive memory usage", words.len());
        if words.len() > 100 {
            return Err("Word list too large (>100 words). Please reduce input size.".into());
        }
    }

    // Estimate total combinations for progress calculation
    let estimated_total = estimate_total_combinations(&word_variants, words.len().min(MAX_COMBINATION_SIZE));

    // Generate combinations by target length (length-first approach)
    'outer: for target_length in config.min_len..=config.max_len {
        // Generate combinations that result in exactly target_length
        let combinations_for_length = generate_combinations_for_length(
            &word_variants,
            words.len().min(MAX_COMBINATION_SIZE),
            target_length
        );

        for combo in combinations_for_length {
            // Only add if not seen before (deduplication)
            if seen_passwords.insert(combo.clone()) {
                chunk_buffer.push(combo);

                // Write chunk when buffer is full
                if chunk_buffer.len() >= config.chunk_size {
                    write_chunk(&mut writer, &chunk_buffer)?;
                    total_count += chunk_buffer.len();
                    chunk_buffer.clear();

                    // Clear seen_passwords periodically to manage memory with bounds check
                    if seen_passwords.len() > MAX_HASHSET_SIZE.min(config.chunk_size * 10) {
                        seen_passwords.clear();
                    }

                    // Update status display (only every 2 seconds)
                    if !config.quiet && (first_display || last_update.elapsed() >= Duration::from_secs(2)) {
                        update_status_display(total_count, &start_time, &config.output_file, words, target_length, first_display, estimated_total);
                        last_update = Instant::now();
                        first_display = false;
                    }

                    if config.limit > 0 && total_count >= config.limit {
                        break 'outer;
                    }
                }
            }
        }
    }

    // Write remaining combinations in buffer
    if !chunk_buffer.is_empty() {
        write_chunk(&mut writer, &chunk_buffer)?;
        total_count += chunk_buffer.len();
    }

    writer.flush()?;
    drop(writer); // Ensure file is closed before renaming

    // Atomically move temporary file to final location
    if let Some(temp_path) = temp_path {
        std::fs::rename(&temp_path, &config.output_file)?;
    }

    Ok(total_count)
}

fn generate_combinations_for_length(
    word_variants: &[Vec<String>],
    max_combo_size: usize,
    target_length: usize,
) -> Vec<String> {
    let mut results_set = HashSet::new();
    let special_chars = ['!', '@', '#', '$', '%'];

    // Try different combo sizes (1 to max_combo_size words)
    for combo_size in 1..=max_combo_size.min(word_variants.len()) {
        // Get all combinations of word indices
        for combo_indices in (0..word_variants.len()).combinations(combo_size) {
            // Generate all permutations of these word groups
            for perm in combo_indices.iter().permutations(combo_size) {
                let perm_variants: Vec<&Vec<String>> = perm
                    .iter()
                    .map(|&&i| &word_variants[i])
                    .collect();

                // Generate cartesian product for this permutation
                let mut temp_combinations = Vec::new();
                generate_cartesian_product_for_length(&perm_variants, &mut temp_combinations, target_length);

                // Add combinations that are exactly target_length
                for combo in temp_combinations {
                    if combo.len() == target_length {
                        results_set.insert(combo.clone());
                    }

                    // Try adding special characters to reach target_length
                    if combo.len() < target_length {
                        let needed_chars = target_length - combo.len();

                        // Add special chars at end (up to the needed amount)
                        if needed_chars <= special_chars.len() {
                            for chars_to_add in 1..=needed_chars {
                                if chars_to_add <= special_chars.len() {
                                    for special_combo in special_chars.iter().combinations(chars_to_add) {
                                        for perm_special in special_combo.iter().permutations(chars_to_add) {
                                            let mut padded = combo.clone();
                                            for &&special in perm_special {
                                                padded.push(special);
                                            }
                                            if padded.len() == target_length {
                                                results_set.insert(padded);
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Add single special char at beginning
                        if needed_chars == 1 {
                            for &special in &special_chars {
                                let padded = format!("{}{}", special, combo);
                                if padded.len() == target_length {
                                    results_set.insert(padded);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Convert HashSet to sorted Vec for consistent output
    let mut results: Vec<String> = results_set.into_iter().collect();
    results.sort();
    results
}

fn generate_cartesian_product_for_length(
    variant_groups: &[&Vec<String>],
    results: &mut Vec<String>,
    target_length: usize,
) {
    fn cartesian_recursive(
        groups: &[&Vec<String>],
        pos: usize,
        current: String,
        results: &mut Vec<String>,
        target_length: usize,
    ) {
        if pos >= groups.len() {
            // Only add if length is close to target (for padding consideration)
            if current.len() <= target_length {
                results.push(current);
            }
            return;
        }

        for variant in groups[pos] {
            // Pre-check length before string allocation
            if current.len() + variant.len() <= target_length {
                let mut new_combination = String::with_capacity(current.len() + variant.len() + 10); // Extra capacity for efficiency
                new_combination.push_str(&current);
                new_combination.push_str(variant);
                cartesian_recursive(groups, pos + 1, new_combination, results, target_length);
            }
        }
    }

    cartesian_recursive(variant_groups, 0, String::new(), results, target_length);
}

fn write_chunk(writer: &mut BufWriter<File>, combinations: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    for combination in combinations {
        writeln!(writer, "{}", combination)?;
    }
    Ok(())
}

fn estimate_total_combinations(word_variants: &[Vec<String>], max_combo_size: usize) -> usize {
    let mut total: usize = 0;

    // For each combination size (1 to max_combo_size words)
    for combo_size in 1..=max_combo_size.min(word_variants.len()) {
        // Number of ways to choose combo_size words from total words
        let combinations = binomial_coefficient(word_variants.len(), combo_size);

        // Number of permutations of those chosen words
        let permutations = factorial(combo_size);

        // Average number of variants per word (more conservative estimate)
        let avg_variants = word_variants.iter()
            .map(|variants| variants.len().min(MAX_WORD_VARIANTS / 10)) // More conservative cap
            .sum::<usize>() / word_variants.len().max(1);

        // Total combinations for this size with enhanced overflow protection
        let mut size_total = combinations.saturating_mul(permutations);

        // Apply variants with stricter overflow protection
        for _ in 0..combo_size {
            if size_total > 1_000_000 / avg_variants.max(1) {
                size_total = 1_000_000; // Much more conservative cap
                break;
            }
            size_total = size_total.saturating_mul(avg_variants);
        }

        total = total.saturating_add(size_total);

        // Much more conservative caps to prevent memory exhaustion
        let max_for_size = match combo_size {
            1 => 100_000,
            2 => 1_000_000,
            3 => 5_000_000,
            _ => 10_000_000, // Hard cap for 4+ word combinations
        };

        if total > max_for_size {
            return max_for_size;
        }
    }

    total.clamp(10_000, 50_000_000) // Reasonable bounds
}

fn binomial_coefficient(n: usize, k: usize) -> usize {
    if k > n || k == 0 {
        return if k == 0 { 1 } else { 0 };
    }

    let mut result = 1;
    for i in 0..k.min(n - k) {
        result = result * (n - i) / (i + 1);
        if result > 10_000_000 { // Cap to avoid huge numbers
            return result;
        }
    }
    result
}

fn factorial(n: usize) -> usize {
    if n <= 1 { return 1; }
    let mut result = 1;
    for i in 2..=n {
        result *= i;
        if result > 10_000 { // Cap factorial to avoid huge numbers
            return result;
        }
    }
    result
}