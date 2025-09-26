use crate::display::update_status_display;
use crate::words::create_word_variants;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::time::{Duration, Instant};
use itertools::Itertools;

pub fn generate_combinations_streaming(
    words: &[String],
    min_len: usize,
    max_len: usize,
    limit: usize,
    output_file: &str,
    chunk_size: usize,
    quiet: bool,
    append: bool,
) -> Result<usize, Box<dyn std::error::Error>> {

    let file = if append {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(output_file)?
    } else {
        File::create(output_file)?
    };
    let mut writer = BufWriter::new(file);
    let mut total_count = 0;
    let mut chunk_buffer = Vec::with_capacity(chunk_size);

    let start_time = Instant::now();
    let mut last_update = Instant::now();
    let mut first_display = true;

    // Create l33t variants for each word
    let word_variants: Vec<Vec<String>> = words
        .iter()
        .map(|word| create_word_variants(word))
        .collect();

    // Estimate total combinations for progress calculation
    let estimated_total = estimate_total_combinations(&word_variants, words.len().min(6));

    // Generate combinations by target length (length-first approach)
    'outer: for target_length in min_len..=max_len {
        // Generate combinations that result in exactly target_length
        let combinations_for_length = generate_combinations_for_length(
            &word_variants,
            words.len().min(6),
            target_length
        );

        for combo in combinations_for_length {
            chunk_buffer.push(combo);

            // Write chunk when buffer is full
            if chunk_buffer.len() >= chunk_size {
                write_chunk(&mut writer, &chunk_buffer)?;
                total_count += chunk_buffer.len();
                chunk_buffer.clear();

                // Update status display (only every 2 seconds)
                if !quiet && (first_display || last_update.elapsed() >= Duration::from_secs(2)) {
                    update_status_display(total_count, &start_time, output_file, words, target_length, first_display, estimated_total);
                    last_update = Instant::now();
                    first_display = false;
                }

                if limit > 0 && total_count >= limit {
                    break 'outer;
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
    Ok(total_count)
}

fn generate_combinations_for_length(
    word_variants: &[Vec<String>],
    max_combo_size: usize,
    target_length: usize,
) -> Vec<String> {
    let mut results = Vec::new();
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
                        results.push(combo.clone());
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
                                                results.push(padded);
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
                                    results.push(padded);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Remove duplicates and sort
    results.sort();
    results.dedup();
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
            let new_combination = current.clone() + variant;
            // Prune early if we've already exceeded target length
            if new_combination.len() <= target_length {
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
            .map(|variants| variants.len().min(50)) // Cap variants per word to 50
            .sum::<usize>() / word_variants.len().max(1);

        // Total combinations for this size
        let mut size_total = combinations.saturating_mul(permutations);

        // Apply variants with overflow protection
        for _ in 0..combo_size {
            if size_total > 10_000_000 / avg_variants {
                size_total = 10_000_000; // Cap to prevent overflow
                break;
            }
            size_total = size_total.saturating_mul(avg_variants);
        }

        total = total.saturating_add(size_total);

        // Cap total to reasonable limit based on combo size
        let max_for_size = match combo_size {
            1..=2 => 10_000_000,
            3..=4 => 100_000_000,
            _ => 1_000_000_000,
        };

        if total > max_for_size {
            return max_for_size;
        }
    }

    total.max(1000000) // Ensure minimum reasonable value
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