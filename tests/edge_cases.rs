use p455w0rd::combinatorics::{calculate_total_combinations, CombinatorialConfig};

#[test]
fn test_empty_word_list() {
    let words = vec![];
    let config = CombinatorialConfig {
        max_words: 2,
        include_special_chars: false,
    };

    let result = calculate_total_combinations(&words, &config);
    assert!(result.is_err()); // Should return error for empty word list
}

#[test]
fn test_single_character_words() {
    let words = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let config = CombinatorialConfig {
        max_words: 2,
        include_special_chars: false,
    };

    let result = calculate_total_combinations(&words, &config);
    assert!(result.is_ok());
    let analysis = result.unwrap();
    // Should have some combinations even with single chars
    assert!(analysis.total_combinations > 0);
}

#[test]
fn test_words_with_numbers() {
    let words = vec!["admin123".to_string(), "pass456".to_string()];
    let config = CombinatorialConfig {
        max_words: 2,
        include_special_chars: false,
    };

    let result = calculate_total_combinations(&words, &config);
    assert!(result.is_ok());
    let analysis = result.unwrap();
    // Numbers don't have leet variants but should still have case variations
    assert!(analysis.total_combinations > 0);
}

#[test]
fn test_unicode_words() {
    let words = vec!["café".to_string(), "naïve".to_string()];
    let config = CombinatorialConfig {
        max_words: 2,
        include_special_chars: false,
    };

    let result = calculate_total_combinations(&words, &config);
    assert!(result.is_ok());
    let analysis = result.unwrap();
    // Unicode support - may not have perfect leet mapping but shouldn't crash
    assert!(analysis.total_combinations > 0);
}

#[test]
fn test_very_long_words() {
    let words = vec!["a".repeat(10), "b".repeat(10)]; // Use reasonable length
    let config = CombinatorialConfig {
        max_words: 2,
        include_special_chars: false,
    };

    let result = calculate_total_combinations(&words, &config);
    assert!(result.is_ok());
    let analysis = result.unwrap();
    // Should handle long words gracefully
    assert!(analysis.total_combinations > 0);
}

#[test]
fn test_duplicate_words() {
    let words = vec!["admin".to_string(), "admin".to_string(), "password".to_string()];
    let config = CombinatorialConfig {
        max_words: 2,
        include_special_chars: false,
    };

    let result = calculate_total_combinations(&words, &config);
    assert!(result.is_ok());
    let analysis = result.unwrap();
    // Should handle duplicates without issues
    assert!(analysis.total_combinations > 0);
}

#[test]
fn test_max_words_zero() {
    let words = vec!["admin".to_string(), "password".to_string()];
    let config = CombinatorialConfig {
        max_words: 0, // Should be treated as unlimited
        include_special_chars: false,
    };

    let result = calculate_total_combinations(&words, &config);
    assert!(result.is_ok());
    let analysis = result.unwrap();
    // Should use actual number of words when max_words is 0
    assert_eq!(analysis.total_combinations, analysis.breakdown.by_word_count.iter().map(|b| b.combinations).sum::<u64>());
}

#[test]
fn test_max_words_exceeds_word_count() {
    let words = vec!["admin".to_string(), "password".to_string()];
    let config = CombinatorialConfig {
        max_words: 5, // More words than available
        include_special_chars: false,
    };

    let result = calculate_total_combinations(&words, &config);
    assert!(result.is_ok());
    let analysis = result.unwrap();
    // Should be limited by actual word count
    assert!(analysis.total_combinations > 0);
}

#[test]
fn test_no_leetable_characters() {
    let words = vec!["xyz".to_string(), "qwrt".to_string()];
    let config = CombinatorialConfig {
        max_words: 2,
        include_special_chars: false,
    };

    let result = calculate_total_combinations(&words, &config);
    assert!(result.is_ok());
    let analysis = result.unwrap();
    // Should work with words that have no leetable characters
    assert!(analysis.total_combinations > 0);
}

#[test]
fn test_special_chars_only() {
    let words = vec!["admin".to_string()];
    let config = CombinatorialConfig {
        max_words: 1,
        include_special_chars: true,
    };

    let result = calculate_total_combinations(&words, &config);
    assert!(result.is_ok());
    let analysis = result.unwrap();
    // Should handle special characters correctly
    assert!(analysis.total_combinations > 0);
}