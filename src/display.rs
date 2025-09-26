use std::time::Instant;

pub fn update_status_display(
    total_count: usize,
    start_time: &Instant,
    output_file: &str,
    words: &[String],
    current_length: usize,
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
    println!("Current.Length...: {} character passwords", current_length);
    println!("Speed............: {:.0} P/s", rate);
    if show_progress {
        println!("Progress.........: {}/{} ({:.2}%)", total_count, estimated_total, progress_pct);
    } else {
        println!("Progress.........: {} passwords (estimate exceeded)", total_count);
    }
    println!("Generated........: {} passwords", total_count);
    println!();
}