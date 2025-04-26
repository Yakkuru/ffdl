# FFDL

CLI tool to download files from fuckingfast.co

## Installation

```bash
cargo install --path .
```

## Usage

Download a single file:
```bash
ffdl -u "https://fuckingfast.co/your-link"
```

Download multiple files from a text file:
```bash
ffdl -f urls.txt
```

The text file should contain one URL per line.

## Examples

```bash
# Single file
ffdl -u "https://fuckingfast.co/abc123"

# Multiple files
ffdl -f downloads.txt
```

## Building from source

```bash
git clone https://github.com/yourusername/ffdl
cd ffdl
cargo build --release
```
