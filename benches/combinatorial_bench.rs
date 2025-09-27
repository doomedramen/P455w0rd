use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use p455w0rd::combinatorics::{calculate_total_combinations, CombinatorialConfig};

fn benchmark_combinatorial_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("combinatorial_calculation");

    // Test with different word counts
    for word_count in [2, 3, 4, 5, 10].iter() {
        let words: Vec<String> = (0..*word_count)
            .map(|i| format!("word{}", i))
            .collect();

        for max_words in [2, 3, 4].iter() {
            if *max_words <= *word_count {
                group.bench_with_input(
                    BenchmarkId::new(format!("words_{}_maxwords_{}", word_count, max_words), word_count),
                    &(words.clone(), *max_words),
                    |b, (words, max_words)| {
                        b.iter(|| {
                            let config = CombinatorialConfig {
                                max_words: *max_words,
                                include_special_chars: true,
                            };
                            calculate_total_combinations(black_box(words), black_box(&config)).unwrap()
                        })
                    },
                );
            }
        }
    }

    group.finish();
}

fn benchmark_word_variants(c: &mut Criterion) {
    let mut group = c.benchmark_group("word_variants");

    let test_words = vec![
        "admin".to_string(),
        "password".to_string(),
        "assassin".to_string(),
        "hello".to_string(),
        "testing".to_string(),
    ];

    for word in test_words {
        group.bench_with_input(
            BenchmarkId::new("calculate_actual_word_variants", &word),
            &word,
            |b, word| {
                b.iter(|| {
                    p455w0rd::combinatorics::calculate_actual_word_variants(black_box(word))
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_combinatorial_calculation, benchmark_word_variants);
criterion_main!(benches);