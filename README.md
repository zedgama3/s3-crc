# s3-crc

A simple command-line utility to compute **CRC64-NVMe** checksums (compatible with AWS S3) for files or stdin.

🧮 Uses the **reflected NVMe polynomial** (`0x9A6C9329AC4BC9B5`) and matches the checksum format expected by Amazon S3.

## Features

- Compute CRC64 checksums from files or stdin
- Outputs in Base64 (default), hexadecimal (`--hex`), or uppercase hex (`--uppercase`)
- Optional JSON output for scripting/integration
- Supports file globs (`*.txt`, `dir/*`, etc.)

## Installation

Build and install from source with [Cargo](https://www.rust-lang.org/tools/install):

```sh
cargo install --path .
```

## Usage

```sh
s3-crc [options] <file-glob>...
```

### Examples

Compute Base64 checksum (default):

```sh
s3-crc myfile.txt
```

Hex-encoded output:

```sh
s3-crc --hex myfile.txt
```

Uppercase hex:

```sh
s3-crc --uppercase myfile.txt
```

Glob multiple files and output JSON:

```sh
s3-crc --json --hex "*.log"
```

Read from stdin:

```sh
cat file.bin | s3-crc -
```

### Options

| Flag         | Description                                 |
|--------------|---------------------------------------------|
| `--hex`      | Output checksum as lowercase hex            |
| `--uppercase`| Output checksum as uppercase hex            |
| `--json`     | Output results as formatted JSON            |

## Output Formats

- **Base64** (default): Compatible with AWS S3 ETag for CRC64
- **Hex**: Lowercase 16-digit hexadecimal
- **JSON**: Array of objects with file and checksum fields

## License

MIT License — see [LICENSE](https://opensource.org/licenses/MIT) for details.

---

© 2025 [Jared Fowkes](https://github.com/zedgama3)
