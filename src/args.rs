use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Input file containing words (one per line) or comma-separated words
    #[arg(short, long)]
    pub input: Option<String>,

    /// Words provided directly as arguments
    pub words: Vec<String>,

    /// Output file path
    #[arg(short, long, default_value = "passwords.txt")]
    pub output: String,

    /// Generate passwords for WPA2 (8-63 characters)
    #[arg(long)]
    pub wpa2: bool,

    /// Minimum password length
    #[arg(long, default_value = "4")]
    pub min_length: usize,

    /// Maximum password length
    #[arg(long, default_value = "20")]
    pub max_length: usize,

    /// Maximum number of combinations to generate (0 = unlimited)
    #[arg(long, default_value = "0")]
    pub limit: usize,

    /// Number of passwords to buffer before writing to file
    #[arg(long, default_value = "100000")]
    pub chunk_size: usize,

    /// Disable interactive status display
    #[arg(long)]
    pub quiet: bool,

    /// Append to output file instead of overwriting
    #[arg(long)]
    pub append: bool,

    /// Maximum number of words to combine (default: unlimited)
    #[arg(long, default_value = "0")]
    pub max_words: usize,

    /// Skip special character padding
    #[arg(long)]
    pub no_special_chars: bool,

    /// Skip confirmation prompt for large generation jobs
    #[arg(long)]
    pub force: bool,
}

impl Args {
    pub fn get_length_constraints(&self) -> (usize, usize) {
        if self.wpa2 {
            (8, 63)
        } else {
            (self.min_length, self.max_length)
        }
    }

    pub fn get_max_words(&self) -> usize {
        if self.max_words == 0 {
            usize::MAX // Unlimited
        } else {
            self.max_words
        }
    }
}