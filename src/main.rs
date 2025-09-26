use clap::Parser;
use itertools::Itertools;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write, stdout};
use std::time::{Duration, Instant};
use indicatif::{ProgressBar, ProgressStyle};
use crossterm::{
    cursor,
    terminal::{self, ClearType},
    ExecutableCommand,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file containing words (one per line) or comma-separated words
    #[arg(short, long)]
    input: Option<String>,

    /// Words provided directly as arguments
    words: Vec<String>,

    /// Output file path
    #[arg(short, long, default_value = "passwords.txt")]
    output: String,

    /// Generate passwords for WPA2 (8-63 characters)
    #[arg(long)]
    wpa2: bool,

    /// Minimum password length
    #[arg(long, default_value = "4")]
    min_length: usize,

    /// Maximum password length
    #[arg(long, default_value = "20")]
    max_length: usize,

    /// Maximum number of combinations to generate (0 = unlimited)
    #[arg(long, default_value = "0")]
    limit: usize,

    /// Number of passwords to buffer before writing to file
    #[arg(long, default_value = "100000")]
    chunk_size: usize,

    /// Disable interactive status display
    #[arg(long)]
    quiet: bool,

    /// Quick mode: fewer l33t variants, focus on 8-24 character passwords
    #[arg(long)]
    quick: bool,

    /// Append to output file instead of overwriting
    #[arg(long)]
    append: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Set length constraints for WPA2 or quick mode
    let (min_len, max_len) = if args.wpa2 {
        (8, 63)
    } else if args.quick {
        (8, 24)
    } else {
        (args.min_length, args.max_length)
    };

    // Get words from input
    let words = get_words(&args)?;

    if words.is_empty() {
        eprintln!("No words provided. Use --input file or provide words as arguments.");
        std::process::exit(1);
    }

    println!("Processing {} words...", words.len());

    // Generate and write combinations incrementally
    let count = generate_combinations_streaming(&words, min_len, max_len, args.limit, &args.output, args.chunk_size, args.quiet, args.quick, args.append)?;

    println!("Generated {} passwords to {}", count, args.output);

    Ok(())
}

fn get_words(args: &Args) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut words = Vec::new();

    // Add words from arguments
    words.extend(args.words.clone());

    // Add words from input file if provided
    if let Some(input_file) = &args.input {
        let content = std::fs::read_to_string(input_file)?;
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() {
                // Handle comma-separated words in a line
                if line.contains(',') {
                    words.extend(line.split(',').map(|w| w.trim().to_string()));
                } else {
                    words.push(line.to_string());
                }
            }
        }
    }

    // Remove duplicates and empty strings
    words.sort();
    words.dedup();
    words.retain(|w| !w.is_empty());

    Ok(words)
}

fn generate_combinations_streaming(
    words: &[String],
    min_len: usize,
    max_len: usize,
    limit: usize,
    output_file: &str,
    chunk_size: usize,
    quiet: bool,
    quick_mode: bool,
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
        .map(|word| if quick_mode { create_quick_variants(word) } else { create_word_variants(word) })
        .collect();

    // Estimate total combinations for progress calculation
    let estimated_total = estimate_total_combinations(&word_variants, words.len().min(6));

    // Generate combinations for different lengths (1 to n words)
    'outer: for combo_size in 1..=words.len().min(6) {

        // Get all combinations of indices
        for combo_indices in (0..words.len()).combinations(combo_size) {
            // Generate all permutations of these word groups
            for perm in combo_indices.iter().permutations(combo_size) {
                let perm_variants: Vec<&Vec<String>> = perm
                    .iter()
                    .map(|&&i| &word_variants[i])
                    .collect();

                // Generate cartesian product and write in chunks
                let mut temp_combinations = Vec::new();
                generate_cartesian_product(&perm_variants, &mut temp_combinations, min_len, max_len);

                // Apply padding and add to chunk buffer
                let padded = apply_padding_and_filtering(temp_combinations, min_len, max_len);

                for combo in padded {
                    chunk_buffer.push(combo);

                    // Write chunk when buffer is full
                    if chunk_buffer.len() >= chunk_size {
                        write_chunk(&mut writer, &chunk_buffer)?;
                        total_count += chunk_buffer.len();
                        chunk_buffer.clear();

                        // Update status display (only every 2 seconds)
                        if !quiet && (first_display || last_update.elapsed() >= Duration::from_secs(2)) {
                            update_status_display(total_count, &start_time, output_file, words, combo_size, first_display, estimated_total);
                            last_update = Instant::now();
                            first_display = false;
                        }

                        if limit > 0 && total_count >= limit {
                            break 'outer;
                        }
                    }
                }
            }
        }
    }

    // Write remaining combinations in buffer
    if !chunk_buffer.is_empty() {
        // Sort and deduplicate final chunk
        chunk_buffer.sort();
        chunk_buffer.dedup();
        write_chunk(&mut writer, &chunk_buffer)?;
        total_count += chunk_buffer.len();
    }

    writer.flush()?;
    Ok(total_count)
}

fn create_quick_variants(word: &str) -> Vec<String> {
    let mut variants = Vec::new();

    // Original word (lowercase)
    variants.push(word.to_lowercase());

    // Capitalize first letter
    let capitalized = capitalize_first(word);
    variants.push(capitalized);

    // All uppercase
    variants.push(word.to_uppercase());

    // Simple l33t replacements (only common ones)
    let simple_leet = create_simple_leet_variants(word);
    variants.extend(simple_leet);

    // Remove duplicates
    variants.sort();
    variants.dedup();
    variants
}

fn capitalize_first(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
    }
}

fn create_simple_leet_variants(word: &str) -> Vec<String> {
    let mut variants = Vec::new();
    let lower = word.to_lowercase();

    // Only the most common l33t replacements
    let simple_replacements = [
        ('a', '4'),
        ('e', '3'),
        ('i', '1'),
        ('l', '1'),
        ('o', '0'),
        ('s', '5'),
    ];

    for &(from, to) in &simple_replacements {
        if lower.contains(from) {
            let replaced = lower.replace(from, &to.to_string());
            variants.push(replaced.clone());

            // Also add capitalized version
            variants.push(capitalize_first(&replaced));
        }
    }

    variants
}

fn create_word_variants(word: &str) -> Vec<String> {
    let mut variants = Vec::new();

    // Get all l33t variants first (these are lowercase)
    let leet_variants = create_leet_variants(word);

    // For each l33t variant, create all capitalization patterns
    for leet_word in &leet_variants {
        // Add all capitalization variants of this l33t version
        variants.extend(create_capitalization_variants(leet_word));
    }

    // Remove duplicates
    variants.sort();
    variants.dedup();
    variants
}

fn create_capitalization_variants(word: &str) -> Vec<String> {
    let mut variants = Vec::new();
    let chars: Vec<char> = word.chars().collect();

    if chars.is_empty() {
        return variants;
    }

    // All combinations of case changes for alphabetic characters
    let alpha_positions: Vec<usize> = chars
        .iter()
        .enumerate()
        .filter(|(_, &ch)| ch.is_alphabetic())
        .map(|(i, _)| i)
        .collect();

    if alpha_positions.is_empty() {
        variants.push(word.to_string());
        return variants;
    }

    // Generate all possible case combinations using bit patterns
    let max_combinations = 1 << alpha_positions.len();

    for combination in 0..max_combinations {
        let mut variant_chars = chars.clone();

        for (bit_pos, &char_pos) in alpha_positions.iter().enumerate() {
            if (combination >> bit_pos) & 1 == 1 {
                variant_chars[char_pos] = variant_chars[char_pos].to_uppercase().next().unwrap_or(variant_chars[char_pos]);
            } else {
                variant_chars[char_pos] = variant_chars[char_pos].to_lowercase().next().unwrap_or(variant_chars[char_pos]);
            }
        }

        variants.push(variant_chars.iter().collect());
    }

    variants
}

fn capitalize_word(word: &str) -> String {
    let mut chars: Vec<char> = word.chars().collect();
    if !chars.is_empty() {
        chars[0] = chars[0].to_uppercase().next().unwrap_or(chars[0]);
    }
    chars.into_iter().collect()
}

fn create_leet_variants(word: &str) -> Vec<String> {
    let leet_map: HashMap<char, Vec<char>> = [
        ('a', vec!['@', '4']),
        ('e', vec!['3']),
        ('i', vec!['1', '!']),
        ('l', vec!['1', '!']),
        ('o', vec!['0']),
        ('s', vec!['5', '$']),
        ('t', vec!['7']),
        ('g', vec!['9']),
        ('b', vec!['6']),
        ('z', vec!['2']),
    ].iter().cloned().collect();

    let mut all_variants = Vec::new();
    let chars: Vec<char> = word.to_lowercase().chars().collect();

    // Generate all possible combinations recursively
    generate_all_leet_combinations(&chars, 0, String::new(), &leet_map, &mut all_variants);

    // Remove duplicates and empty strings
    all_variants.retain(|s| !s.is_empty());
    all_variants.sort();
    all_variants.dedup();
    all_variants
}

fn generate_all_leet_combinations(
    chars: &[char],
    pos: usize,
    current: String,
    leet_map: &HashMap<char, Vec<char>>,
    results: &mut Vec<String>,
) {
    if pos >= chars.len() {
        results.push(current);
        return;
    }

    let ch = chars[pos];

    // Always try the original character
    generate_all_leet_combinations(chars, pos + 1, current.clone() + &ch.to_string(), leet_map, results);

    // Try all l33t replacements for this character
    if let Some(replacements) = leet_map.get(&ch) {
        for &replacement in replacements {
            generate_all_leet_combinations(
                chars,
                pos + 1,
                current.clone() + &replacement.to_string(),
                leet_map,
                results,
            );
        }
    }
}

fn generate_cartesian_product(
    variant_groups: &[&Vec<String>],
    results: &mut Vec<String>,
    min_len: usize,
    max_len: usize,
) {
    fn cartesian_recursive(
        groups: &[&Vec<String>],
        pos: usize,
        current: String,
        results: &mut Vec<String>,
        min_len: usize,
        max_len: usize,
    ) {
        if pos >= groups.len() {
            if current.len() >= min_len && current.len() <= max_len {
                results.push(current);
            }
            return;
        }

        for variant in groups[pos] {
            let new_combination = current.clone() + variant;
            if new_combination.len() <= max_len {
                cartesian_recursive(groups, pos + 1, new_combination, results, min_len, max_len);
            }
        }
    }

    cartesian_recursive(variant_groups, 0, String::new(), results, min_len, max_len);
}

fn apply_padding_and_filtering(mut combinations: Vec<String>, min_len: usize, max_len: usize) -> Vec<String> {
    let special_chars = vec!['!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '_', '+', '='];
    let mut padded_combinations = Vec::new();

    for combo in combinations.drain(..) {
        // Add original if it meets length requirements
        if combo.len() >= min_len && combo.len() <= max_len {
            padded_combinations.push(combo.clone());
        }

        // Add padding variants if needed
        if combo.len() < max_len {
            // Add special chars at the end
            for &special in &special_chars {
                let padded = format!("{}{}", combo, special);
                if padded.len() >= min_len && padded.len() <= max_len {
                    padded_combinations.push(padded);
                }

                // Add multiple special chars
                for count in 2..=(max_len - combo.len()).min(3) {
                    let padding: String = special.to_string().repeat(count);
                    let padded = format!("{}{}", combo, padding);
                    if padded.len() >= min_len && padded.len() <= max_len {
                        padded_combinations.push(padded);
                    }
                }
            }

            // Add special chars at the beginning
            for &special in special_chars.iter().take(5) {
                let padded = format!("{}{}", special, combo);
                if padded.len() >= min_len && padded.len() <= max_len {
                    padded_combinations.push(padded);
                }
            }
        }
    }

    // Remove duplicates and sort
    padded_combinations.sort();
    padded_combinations.dedup();
    padded_combinations
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

fn update_status_display(
    total_count: usize,
    start_time: &Instant,
    output_file: &str,
    words: &[String],
    current_combo_size: usize,
    is_first: bool,
    estimated_total: usize,
) {
    let elapsed = start_time.elapsed();
    let rate = if elapsed.as_secs() > 0 {
        total_count as f64 / elapsed.as_secs() as f64
    } else {
        0.0
    };

    // Calculate progress and ETA
    let (progress_pct, show_progress) = if estimated_total > 0 && total_count <= estimated_total {
        ((total_count as f64 / estimated_total as f64 * 100.0).min(100.0), true)
    } else if estimated_total > 0 && total_count > estimated_total {
        // If we've exceeded estimate significantly, don't show percentage
        let excess_factor = total_count as f64 / estimated_total as f64;
        if excess_factor > 3.0 {
            (0.0, false) // Don't show progress when estimate is clearly wrong
        } else {
            // Small excess, show capped progress
            (95.0, true)
        }
    } else {
        (0.0, false)
    };

    let eta_secs = if rate > 0.0 && estimated_total > total_count {
        ((estimated_total - total_count) as f64 / rate).min(86400.0) // Cap at 24 hours
    } else if total_count > estimated_total {
        // When exceeded estimate, show "Unknown" ETA
        -1.0 // Special value for unknown
    } else {
        0.0
    };

    let eta_formatted = if eta_secs < 0.0 {
        "Unknown".to_string()
    } else if eta_secs > 3600.0 {
        format!("{:.1}h", eta_secs / 3600.0)
    } else if eta_secs > 60.0 {
        format!("{:.1}m", eta_secs / 60.0)
    } else {
        format!("{:.0}s", eta_secs)
    };

    // Move cursor up to overwrite previous display (only if not first time)
    if !is_first {
        print!("\x1B[12A"); // Move cursor up 12 lines
        print!("\x1B[0J"); // Clear from cursor to end of screen
    }

    println!("Session..........: p455w0rd");
    println!("Status...........: Running");
    println!("Mode.............: Password Generator");
    println!("Target...........: {}", output_file);
    println!("Time.Elapsed.....: {:.0}s", elapsed.as_secs_f64());
    println!("Time.ETA.........: {}", eta_formatted);
    println!("Words............: {} words", words.len());
    println!("Combo.Size.......: {}-word combinations", current_combo_size);
    println!("Speed............: {:.0} H/s", rate);
    if show_progress {
        println!("Progress.........: {}/{} ({:.2}%)", total_count, estimated_total, progress_pct);
    } else {
        println!("Progress.........: {} passwords (estimate exceeded)", total_count);
    }
    println!("Generated........: {} passwords", total_count);
    println!();
}