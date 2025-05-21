# FFDL Fucking Fast CLI downloader

CLI tool to download files from fuckingfast.co

## Installation

install rust tools first
https://www.rust-lang.org/tools/install

then clone the repo, go in folder and

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
