use itertools::Itertools;

pub struct CombinatorialConfig {
    pub max_words: usize,
    pub include_special_chars: bool,
}

#[derive(Debug, Clone)]
pub struct CombinatorialAnalysis {
    pub total_combinations: u64,
    pub estimated_file_size_bytes: u64,
    pub breakdown: CombinationBreakdown,
}

#[derive(Debug, Clone)]
pub struct CombinationBreakdown {
    pub word_permutations: u64,
    pub leet_variants: u64,
    pub case_variants: u64,
    pub special_char_variants: u64,
    pub by_word_count: Vec<WordCountBreakdown>,
}

#[derive(Debug, Clone)]
pub struct WordCountBreakdown {
    pub word_count: usize,
    pub combinations: u64,
    pub average_length: f64,
}

pub fn calculate_total_combinations(
    words: &[String],
    config: &CombinatorialConfig,
) -> Result<CombinatorialAnalysis, String> {
    if words.is_empty() {
        return Err("No words provided for combinatorial analysis".to_string());
    }

    // Remove duplicate words
    let unique_words: Vec<String> = words.iter().cloned().collect::<std::collections::HashSet<_>>().into_iter().collect();
    let n = unique_words.len();

    // 1. Calculate word permutations
    let word_permutations = calculate_word_permutations(n, config.max_words)?;

    // 2. Calculate leet variants for each word
    let leet_variants_per_word: Vec<u64> = unique_words
        .iter()
        .map(|word| calculate_leet_variants(word))
        .collect();

    let total_leet_variants: u64 = leet_variants_per_word.iter().product();

    // 3. Case variations (always 3 per variant)
    let _case_variants = 3;

    // 4. Special character padding
    let special_char_variants = if config.include_special_chars {
        calculate_special_char_variants()
    } else {
        1 // No padding
    };

    // Calculate breakdown by word count first (this gives us the accurate count)
    let by_word_count = calculate_breakdown_by_word_count(
        &unique_words,
        &leet_variants_per_word,
        config.max_words,
        config.include_special_chars,
    )?;

    // Calculate total combinations from breakdown (more accurate)
    let total_combinations = by_word_count
        .iter()
        .map(|b| b.combinations)
        .sum::<u64>()
        .min(1_000_000_000); // Cap at reasonable number

    // Estimate file size (average 15 characters per password + newline)
    let avg_password_length = estimate_average_password_length(&unique_words, config.include_special_chars);
    let estimated_file_size_bytes = total_combinations
        .checked_mul(avg_password_length as u64 + 1) // +1 for newline
        .unwrap_or(u64::MAX);

    Ok(CombinatorialAnalysis {
        total_combinations,
        estimated_file_size_bytes,
        breakdown: CombinationBreakdown {
            word_permutations,  // Still useful for reference
            leet_variants: total_leet_variants,  // Still useful for reference
            case_variants: 3,  // Theoretical maximum
            special_char_variants,
            by_word_count,
        },
    })
}

fn calculate_word_permutations(n: usize, max_words: usize) -> Result<u64, String> {
    let mut total = 0u64;

    for k in 1..=max_words.min(n) {
        // Calculate permutations: P(n, k) = n! / (n - k)!
        let permutations = permutation_count(n, k)?;
        total = total.checked_add(permutations)
            .ok_or_else(|| format!("Overflow calculating permutations for {} words", k))?;
    }

    Ok(total)
}

fn permutation_count(n: usize, k: usize) -> Result<u64, String> {
    if k > n {
        return Ok(0);
    }

    let mut result = 1u64;
    for i in 0..k {
        result = result.checked_mul((n - i) as u64)
            .ok_or_else(|| format!("Overflow in permutation calculation: P({}, {})", n, k))?;
    }

    Ok(result)
}

fn calculate_leet_variants(word: &str) -> u64 {
    let replacements = [
        ('a', '4'),
        ('e', '3'),
        ('i', '1'),
        ('l', '1'),
        ('o', '0'),
        ('s', '5'),
    ];

    let replaceable_count = word
        .to_lowercase()
        .chars()
        .filter(|&ch| replacements.iter().any(|&(from, _)| from == ch))
        .count();

    // 2^K possible leet combinations
    if replaceable_count >= 64 {
        return u64::MAX; // Would overflow, return max
    }

    1u64 << replaceable_count // 2^replaceable_count
}

fn calculate_special_char_variants() -> u64 {
    let special_chars = ['!', '@', '#', '$', '%'];
    let n = special_chars.len();

    // No padding: 1
    let mut total = 1u64;

    // Single prefix: n variants
    total = total.checked_add(n as u64).unwrap_or(u64::MAX);

    // Single suffix: n variants
    total = total.checked_add(n as u64).unwrap_or(u64::MAX);

    // Multiple padding: all permutations of 2-5 special chars (both prefix and suffix)
    for k in 2..=n {
        let permutations = permutation_count(n, k).unwrap_or(u64::MAX);
        let doubled = permutations.checked_mul(2).unwrap_or(u64::MAX); // ×2 for prefix/suffix
        total = total.checked_add(doubled).unwrap_or(u64::MAX);

        if total == u64::MAX {
            break;
        }
    }

    total
}

pub fn calculate_actual_word_variants(word: &str) -> u64 {
    let lower = word.to_lowercase();

    // Generate all possible l33t combinations for this word
    let leet_variants = generate_all_leet_for_word_combinatorics(&lower);

    // For each l33t variant, add different capitalizations
    let mut variants = Vec::new();
    for leet_word in leet_variants {
        variants.push(leet_word.clone());                    // lowercase
        variants.push(capitalize_word_combinatorics(&leet_word));         // Capitalized
        variants.push(leet_word.to_uppercase());            // UPPERCASE
    }

    // Remove duplicates (same as generator)
    variants.sort();
    variants.dedup();

    variants.len() as u64
}

fn generate_all_leet_for_word_combinatorics(word: &str) -> Vec<String> {
    let replacements = [
        ('a', '4'),
        ('e', '3'),
        ('i', '1'),
        ('l', '1'),
        ('o', '0'),
        ('s', '5'),
    ];

    let chars: Vec<char> = word.chars().collect();
    let replaceable_positions: Vec<usize> = chars
        .iter()
        .enumerate()
        .filter(|(_, &ch)| replacements.iter().any(|&(from, _)| from == ch))
        .map(|(i, _)| i)
        .collect();

    let mut variants = Vec::new();
    let max_combinations = 1 << replaceable_positions.len();

    for combination in 0..max_combinations {
        let mut result_chars = chars.clone();
        for (bit_pos, &char_pos) in replaceable_positions.iter().enumerate() {
            if (combination >> bit_pos) & 1 == 1 {
                if let Some(&(_, replacement)) = replacements.iter().find(|&&(from, _)| from == result_chars[char_pos]) {
                    result_chars[char_pos] = replacement;
                }
            }
        }
        variants.push(result_chars.iter().collect::<String>());
    }

    variants
}

fn capitalize_word_combinatorics(word: &str) -> String {
    if word.is_empty() {
        return String::new();
    }

    let mut chars = word.chars();
    if let Some(first) = chars.next() {
        let uppercase_first = first.to_uppercase().collect::<String>();
        if uppercase_first.len() == 1 && uppercase_first.starts_with(first) {
            // No change needed, use Cow to avoid allocation
            word.to_string()
        } else {
            // Capitalization needed
            uppercase_first + &chars.collect::<String>()
        }
    } else {
        String::new()
    }
}


fn calculate_breakdown_by_word_count(
    words: &[String],
    _leet_variants_per_word: &[u64],
    max_words: usize,
    include_special_chars: bool,
) -> Result<Vec<WordCountBreakdown>, String> {
    let mut breakdown = Vec::new();
    let n = words.len();

    for k in 1..=max_words.min(n) {
        let _permutations = permutation_count(n, k)?;

        // Calculate actual leet variants and their case variations for k-word combinations
        let total_combinations = if k == 1 {
            // For single words, sum up variants for each word and multiply by special variants
            let mut total_single_word_variants = 0u64;
            for word_idx in 0..words.len() {
                let word = &words[word_idx];
                let actual_variants = calculate_actual_word_variants(word);
                total_single_word_variants += actual_variants;
            }

            let special_variants = if include_special_chars { calculate_special_char_variants() } else { 1 };

            total_single_word_variants
                .checked_mul(special_variants)
                .unwrap_or(u64::MAX)
        } else {
            // For multi-word combinations, calculate for all permutations
            // Each permutation consists of k distinct words from the available n words
            let mut total_combinations = 0u64;

            // For each permutation of k distinct words
            for indices in (0..words.len()).permutations(k) {
                // Calculate cartesian product for this specific combination of words
                let mut cartesian_product = 1u64;
                for &idx in &indices {
                    let word = &words[idx];
                    let actual_variants = calculate_actual_word_variants(word);
                    cartesian_product = cartesian_product.checked_mul(actual_variants).unwrap_or(u64::MAX);
                }

                total_combinations = total_combinations.checked_add(cartesian_product).unwrap_or(u64::MAX);
                if total_combinations == u64::MAX {
                    break;
                }
            }

            let special_variants = if include_special_chars { calculate_special_char_variants() } else { 1 };
            total_combinations
                .checked_mul(special_variants)
                .unwrap_or(u64::MAX)
        };

        // Estimate average length for k-word combinations
        let avg_word_length = words.iter().take(k).map(|w| w.len()).sum::<usize>() as f64 / k as f64;
        let avg_length = avg_word_length * k as f64;

        breakdown.push(WordCountBreakdown {
            word_count: k,
            combinations: total_combinations,
            average_length: avg_length,
        });
    }

    Ok(breakdown)
}

fn estimate_average_password_length(words: &[String], include_special_chars: bool) -> usize {
    if words.is_empty() {
        return 0;
    }

    let avg_word_length = words.iter().map(|w| w.len()).sum::<usize>() / words.len();

    // Estimate based on typical usage patterns
    let multiplier = if include_special_chars { 1.5 } else { 1.2 };

    (avg_word_length as f64 * multiplier) as usize
}

pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let bytes_f64 = bytes as f64;
    let base = 1024_f64;

    let i = (bytes_f64.ln() / base.ln()) as usize;
    let i = i.min(UNITS.len() - 1);

    let size = bytes_f64 / base.powi(i as i32);

    if i == 0 {
        format!("{} {}", bytes, UNITS[i])
    } else {
        format!("{:.1} {}", size, UNITS[i])
    }
}

pub fn format_combination_count(count: u64) -> String {
    if count == u64::MAX {
        return "too many to count".to_string();
    }

    const UNITS: &[&str] = &["", "thousand", "million", "billion", "trillion"];

    let count_f64 = count as f64;
    let base = 1000_f64;

    let i = (count_f64.ln() / base.ln()) as usize;
    let i = i.min(UNITS.len() - 1);

    let size = count_f64 / base.powi(i as i32);

    if i == 0 {
        count.to_string()
    } else {
        format!("{:.1} {}", size, UNITS[i])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    fn generate_random_word(length: usize) -> String {
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
        let mut rng = rand::thread_rng();
        (0..length)
            .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
            .collect()
    }

    fn generate_random_word_list(min_words: usize, max_words: usize, min_len: usize, max_len: usize) -> Vec<String> {
        let mut rng = rand::thread_rng();
        let word_count = rng.gen_range(min_words..=max_words);
        (0..word_count)
            .map(|_| generate_random_word(rng.gen_range(min_len..=max_len)))
            .collect()
    }

    fn verify_calculation_is_reasonable(words: &[String], max_words: usize, include_special_chars: bool) -> Result<(), String> {
        let config = CombinatorialConfig {
            max_words,
            include_special_chars,
        };

        // Calculate expected count
        let analysis = calculate_total_combinations(words, &config)?;
        let expected_count = analysis.total_combinations;

        // Basic sanity checks
        if expected_count == 0 {
            return Err("Expected count should not be zero".to_string());
        }

        // Check that breakdown is consistent with total
        let breakdown_total: u64 = analysis.breakdown.by_word_count.iter().map(|b| b.combinations).sum();
        if breakdown_total != expected_count {
            return Err(format!("Breakdown total {} doesn't match expected count {}", breakdown_total, expected_count));
        }

        // Check that special chars increase the count when enabled
        if include_special_chars {
            let config_no_special = CombinatorialConfig {
                include_special_chars: false,
                ..config
            };
            let analysis_no_special = calculate_total_combinations(words, &config_no_special)?;
            if expected_count <= analysis_no_special.total_combinations {
                return Err("Special chars should increase count".to_string());
            }
        }

        // Check that more words allow more combinations (when we have multiple words)
        if words.len() > 1 && max_words > 1 {
            let config_single_word = CombinatorialConfig {
                max_words: 1,
                ..config
            };
            let analysis_single = calculate_total_combinations(words, &config_single_word)?;
            if expected_count < analysis_single.total_combinations {
                return Err("More max_words should not decrease total combinations".to_string());
            }
        }

        Ok(())
    }

    #[test]
    fn test_calculate_leet_variants() {
        // Word with no replaceable characters
        assert_eq!(calculate_leet_variants("xyz"), 1);

        // Word with one replaceable character
        assert_eq!(calculate_leet_variants("a"), 2);
        assert_eq!(calculate_leet_variants("e"), 2);
        assert_eq!(calculate_leet_variants("i"), 2);

        // Word with multiple replaceable characters
        assert_eq!(calculate_leet_variants("admin"), 4); // a and i -> 2^2
        assert_eq!(calculate_leet_variants("password"), 16); // a, s, s, o -> 2^4
        assert_eq!(calculate_leet_variants("hello"), 16); // e, l, l, o -> 2^4
        assert_eq!(calculate_leet_variants("aeiou"), 16); // a, e, i, o -> 2^4

        // Case insensitive
        assert_eq!(calculate_leet_variants("ADMIN"), 4);
        assert_eq!(calculate_leet_variants("Admin"), 4);
        assert_eq!(calculate_leet_variants("PASSWORD"), 16);
    }

    #[test]
    fn test_special_char_variants() {
        let variants = calculate_special_char_variants();

        // Should include:
        // - No padding: 1
        // - Single prefix: 5
        // - Single suffix: 5
        // - Multiple padding: permutations of 2-5 chars
        assert!(variants > 16); // At least the basic ones
    }

    #[test]
    fn test_word_permutations() {
        // 1 word
        assert_eq!(calculate_word_permutations(1, 5).unwrap(), 1);

        // 2 words
        assert_eq!(calculate_word_permutations(2, 2).unwrap(), 4); // 2 single words + 2 pairs = 4
        assert_eq!(calculate_word_permutations(2, 1).unwrap(), 2); // Just single words

        // 3 words
        let result3 = calculate_word_permutations(3, 3).unwrap();
        assert_eq!(result3, 15); // 3 singles + 6 pairs + 6 triplets = 15
    }

    #[test]
    fn test_permutation_count() {
        assert_eq!(permutation_count(5, 1).unwrap(), 5);
        assert_eq!(permutation_count(5, 2).unwrap(), 20);
        assert_eq!(permutation_count(5, 3).unwrap(), 60);
        assert_eq!(permutation_count(3, 3).unwrap(), 6);

        // Edge cases
        assert_eq!(permutation_count(5, 0).unwrap(), 1); // 1 way to choose nothing
        assert_eq!(permutation_count(0, 1).unwrap(), 0); // Can't choose 1 from 0
        assert_eq!(permutation_count(1, 0).unwrap(), 1); // 1 way to choose nothing
    }

    #[test]
    fn test_format_combination_count() {
        assert_eq!(format_combination_count(0), "0");
        assert_eq!(format_combination_count(500), "500");
        assert_eq!(format_combination_count(1500), "1.5 thousand");
        assert_eq!(format_combination_count(1_500_000), "1.5 million");
        assert_eq!(format_combination_count(2_000_000_000), "2.0 billion");
        assert_eq!(format_combination_count(u64::MAX), "too many to count");
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1_048_576), "1.0 MB");
        assert_eq!(format_file_size(1_073_741_824), "1.0 GB");
    }

    #[test]
    fn test_full_combinatorial_analysis() {
        let words = vec!["admin".to_string(), "pass".to_string()];
        let config = CombinatorialConfig {
            max_words: 2,
            include_special_chars: false,
        };

        let analysis = calculate_total_combinations(&words, &config).unwrap();

        // Should calculate a reasonable number of combinations
        assert!(analysis.total_combinations > 0);
        assert!(analysis.estimated_file_size_bytes > 0);

        // Check breakdown structure
        assert!(analysis.breakdown.word_permutations > 0);
        assert!(analysis.breakdown.leet_variants > 0);
        assert_eq!(analysis.breakdown.case_variants, 3);
        assert_eq!(analysis.breakdown.special_char_variants, 1); // disabled

        // Check by-word-count breakdown
        assert!(!analysis.breakdown.by_word_count.is_empty());
        for breakdown in &analysis.breakdown.by_word_count {
            assert!(breakdown.word_count > 0);
            assert!(breakdown.combinations > 0);
            assert!(breakdown.average_length > 0.0);
        }
    }

    #[test]
    fn test_with_special_characters() {
        let words = vec!["admin".to_string()];
        let config = CombinatorialConfig {
            max_words: 1,
            include_special_chars: true,
        };

        let analysis = calculate_total_combinations(&words, &config).unwrap();

        // Should have more combinations with special chars
        let config_no_special = CombinatorialConfig {
            include_special_chars: false,
            ..config
        };

        let analysis_no_special = calculate_total_combinations(&words, &config_no_special).unwrap();

        assert!(analysis.total_combinations > analysis_no_special.total_combinations);
    }

    #[test]
    fn test_duplicate_word_removal() {
        let words = vec!["admin".to_string(), "admin".to_string(), "pass".to_string()];
        let config = CombinatorialConfig {
            max_words: 2,
            include_special_chars: false,
        };

        let analysis = calculate_total_combinations(&words, &config).unwrap();

        // Should treat duplicates as single words
        let unique_words = vec!["admin".to_string(), "pass".to_string()];
        let analysis_unique = calculate_total_combinations(&unique_words, &config).unwrap();

        assert_eq!(analysis.total_combinations, analysis_unique.total_combinations);
    }

    // Test with multiple random word configurations
    #[test]
    fn test_randomized_small_word_lists() {
        let mut rng = rand::thread_rng();

        // Test 10 different random configurations
        for _ in 0..10 {
            let words = generate_random_word_list(1, 3, 2, 6);
            let max_words = rng.gen_range(1..=3);
            let include_special_chars = rng.gen_bool(0.5);

            if let Err(e) = verify_calculation_is_reasonable(&words, max_words, include_special_chars) {
                panic!("Randomized test failed: {}", e);
            }
        }
    }

    // Test edge cases with specific characteristics
    #[test]
    fn test_randomized_edge_cases() {
        // Test words with no leetable characters
        let words_no_leet = vec!["xyz".to_string(), "qwrt".to_string()];
        verify_calculation_is_reasonable(&words_no_leet, 2, false).unwrap();

        // Test words with many leetable characters
        let words_many_leet = vec!["assassin".to_string(), "password".to_string()];
        verify_calculation_is_reasonable(&words_many_leet, 2, true).unwrap();

        // Test single letter words
        let words_single = vec!["a".to_string(), "i".to_string(), "s".to_string()];
        verify_calculation_is_reasonable(&words_single, 2, false).unwrap();

        // Test long words
        let words_long = vec!["supercalifragilisticexpialidocious".to_string()];
        verify_calculation_is_reasonable(&words_long, 1, false).unwrap();
    }

    // Test reproducible random sequences to ensure consistency
    #[test]
    fn test_reproducible_random_sequences() {
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        // Use fixed seed for reproducible tests
        let seed = 42;
        let mut rng = StdRng::seed_from_u64(seed);

        let words: Vec<String> = (0..3)
            .map(|_| generate_random_word(rng.gen_range(3..8)))
            .collect();

        verify_calculation_is_reasonable(&words, 2, false).unwrap();
        verify_calculation_is_reasonable(&words, 3, true).unwrap();
    }

    // Stress test with known problematic patterns
    #[test]
    fn test_problematic_patterns() {
        // Words that start with numbers when leet-transformed
        let words = vec!["admin".to_string()]; // becomes "4dmin"
        verify_calculation_is_reasonable(&words, 1, true).unwrap();

        // Words with duplicate case variations
        let words = vec!["aaa".to_string()]; // all case variations look similar
        verify_calculation_is_reasonable(&words, 1, false).unwrap();

        // Mixed case input words
        let words = vec!["AdMiN".to_string(), "PaSsWoRd".to_string()];
        verify_calculation_is_reasonable(&words, 2, false).unwrap();
    }
}