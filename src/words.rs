use crate::args::Args;

pub fn get_words(args: &Args) -> Result<Vec<String>, Box<dyn std::error::Error>> {
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

pub fn create_word_variants(word: &str) -> Vec<String> {
    let mut variants = Vec::new();
    let lower = word.to_lowercase();

    // Generate all possible l33t combinations for this word
    let leet_variants = generate_all_leet_for_word(&lower);

    // For each l33t variant, add different capitalizations
    for leet_word in &leet_variants {
        variants.push(leet_word.clone());                    // lowercase
        variants.push(capitalize_word(leet_word));           // Capitalized
        variants.push(leet_word.to_uppercase());            // UPPERCASE
    }

    // Remove duplicates
    variants.sort();
    variants.dedup();
    variants
}

fn generate_all_leet_for_word(word: &str) -> Vec<String> {
    let replacements = [
        ('a', '4'),
        ('e', '3'),
        ('i', '1'),
        ('l', '1'),
        ('o', '0'),
        ('s', '5'),
    ];

    let chars: Vec<char> = word.chars().collect();
    let mut results = Vec::new();

    // Find all positions that can be replaced
    let replaceable_positions: Vec<(usize, char, char)> = chars
        .iter()
        .enumerate()
        .filter_map(|(i, &ch)| {
            replacements.iter()
                .find(|&&(from, _)| from == ch)
                .map(|&(_, to)| (i, ch, to))
        })
        .collect();

    if replaceable_positions.is_empty() {
        return vec![word.to_string()];
    }

    // Generate all combinations using bit patterns
    let max_combinations = 1 << replaceable_positions.len();

    for combination in 0..max_combinations {
        let mut result_chars = chars.clone();

        for (bit_pos, &(char_pos, _original, replacement)) in replaceable_positions.iter().enumerate() {
            if (combination >> bit_pos) & 1 == 1 {
                result_chars[char_pos] = replacement;
            }
        }

        results.push(result_chars.iter().collect());
    }

    results
}

fn capitalize_word(word: &str) -> String {
    let mut chars: Vec<char> = word.chars().collect();
    if !chars.is_empty() {
        chars[0] = chars[0].to_uppercase().next().unwrap_or(chars[0]);
    }
    chars.into_iter().collect()
}