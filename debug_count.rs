use std::fs::File;
use std::io::{BufRead, BufReader};
use p455w0rd::combinatorics::{calculate_total_combinations, CombinatorialConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let words = vec!["admin".to_string(), "password".to_string()];
    let config = CombinatorialConfig {
        max_words: 2,
        min_len: 4,
        max_len: 50,
        include_special_chars: false,
    };

    let analysis = calculate_total_combinations(&words, &config)?;
    println!("Calculated combinations: {}", analysis.total_combinations);

    // Count actual lines in generated file
    let file = File::open("passwords.txt")?;
    let reader = BufReader::new(file);
    let line_count = reader.lines().count();
    println!("Actual generated passwords: {}", line_count);

    println!("Difference: {}", line_count as i64 - analysis.total_combinations as i64);

    Ok(())
}