# P455w0rd - Password List Generator

A high-performance Rust-based password list generator for security testing and penetration testing purposes.

## Features

- **Word Combination Generation**: Creates combinations from input wordlists
- **Leet Speak Transformation**: Converts letters to numbers (a→4, e→3, i→1, etc.)
- **Case Variations**: Generates lowercase, uppercase, and capitalized versions
- **Special Character Padding**: Adds common special characters (!@#$%) at beginning/end
- **WPA2 Compatibility**: Built-in support for WPA2 password length requirements (8-63 chars)
- **Streaming Output**: Memory-efficient chunked writing to handle large wordlists
- **Progress Display**: Real-time status with ETA and generation rate
- **Flexible Input**: Support for file input or command-line arguments

## Installation

```bash
cargo build --release
```

## Usage

### Basic Usage

```bash
# Generate from command line words
./target/release/p455w0rd word1 word2 word3

# Generate from input file
./target/release/p455w0rd -i wordlist.txt

# Specify output file
./target/release/p455w0rd -o custom_passwords.txt word1 word2
```

### Advanced Options

```bash
# WPA2-compatible passwords (8-63 characters)
./target/release/p455w0rd --wpa2 -i wordlist.txt

# Custom length constraints
./target/release/p455w0rd --min-length 6 --max-length 16 -i wordlist.txt

# Limit number of passwords generated
./target/release/p455w0rd --limit 1000000 -i wordlist.txt

# Quiet mode (no progress display)
./target/release/p455w0rd --quiet -i wordlist.txt

# Append to existing file
./target/release/p455w0rd --append -o existing.txt word1 word2
```

## Command Line Options

- `-i, --input <FILE>`: Input file containing words (one per line or comma-separated)
- `-o, --output <FILE>`: Output file path (default: passwords.txt)
- `--wpa2`: Generate WPA2-compatible passwords (8-63 characters)
- `--min-length <NUM>`: Minimum password length (default: 4)
- `--max-length <NUM>`: Maximum password length (default: 20)
- `--limit <NUM>`: Maximum number of passwords to generate (0 = unlimited)
- `--chunk-size <NUM>`: Buffer size for writing (default: 100000)
- `--quiet`: Disable progress display
- `--append`: Append to output file instead of overwriting

## Input Format

### File Input
Words can be provided in a text file with:
- One word per line
- Comma-separated words on a single line
- Mixed format (some lines with single words, others comma-separated)

### Command Line Input
Words can be provided directly as arguments:
```bash
./target/release/p455w0rd admin password login user
```

## Output

The tool generates password combinations with:
- Original words and their transformations
- Leet speak variations (admin → 4dm1n)
- Case variations (Admin, ADMIN, admin)
- Special character padding (!admin, admin$, etc.)
- Multiple word combinations up to 6 words

## Performance

- **Memory Efficient**: Streams output in configurable chunks
- **Parallel Processing**: Uses Rayon for CPU-intensive operations
- **Progress Tracking**: Real-time status with generation rate and ETA
- **Deduplication**: Automatic removal of duplicate passwords

## Example Output

Given input words: `admin`, `password`

Sample generated passwords:
```
admin
4dm1n
Admin
ADMIN
password
p4ssw0rd
Password
PASSWORD
adminpassword
4dm1np4ssw0rd
!admin
admin!
password123
...
```

## Use Cases

- **Penetration Testing**: Generate targeted wordlists for specific organizations
- **Security Assessment**: Create password lists based on company information
- **Red Team Operations**: Build custom dictionaries for password attacks
- **Security Research**: Analyze password patterns and variations

## Disclaimer

This tool is intended for legitimate security testing and research purposes only. Users are responsible for ensuring compliance with applicable laws and regulations. Only use this tool on systems you own or have explicit permission to test.

## Dependencies

- `clap`: Command-line argument parsing
- `itertools`: Iterator utilities for combinations
- `rayon`: Parallel processing
- `indicatif`: Progress bars and status display
- `crossterm`: Terminal manipulation

## License

This project is distributed under the MIT License.