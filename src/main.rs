use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use clap::Parser;
use glob::glob;
use once_cell::sync::Lazy;
use serde::Serialize;

const NVME_POLY: u64 = 0x9A6C9329AC4BC9B5;
const BUFFER_SIZE: usize = 32 * 1024;

static CRC64_TABLE: Lazy<[u64; 256]> = Lazy::new(|| make_table(NVME_POLY));

#[derive(Parser)]
#[command(name = "s3-crc", version, about = "Compute CRC64-NVMe checksums compatible with AWS S3.")]
struct Cli {
    /// Output checksum as uppercase hex
    #[arg(long)]
    uppercase: bool,

    /// Output results as formatted JSON
    #[arg(long)]
    json: bool,

    /// Output checksum as lowercase hex
    #[arg(long)]
    hex: bool,

    /// Files or globs to process. Use - for stdin.
    #[arg(required = true)]
    patterns: Vec<String>,
}

#[derive(Serialize)]
struct ResultEntry {
    file: String,
    crc64: String,
}

fn make_table(poly: u64) -> [u64; 256] {
    let mut table = [0u64; 256];
    for (i, slot) in table.iter_mut().enumerate() {
        let mut crc = i as u64;
        for _ in 0..8 {
            if crc & 1 == 1 {
                crc = (crc >> 1) ^ poly;
            } else {
                crc >>= 1;
            }
        }
        *slot = crc;
    }
    table
}

fn compute_crc64_from_reader(mut reader: impl Read) -> io::Result<u64> {
    let mut crc = !0u64;
    let mut buffer = [0u8; BUFFER_SIZE];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        for &byte in &buffer[..bytes_read] {
            let index = ((crc as u8) ^ byte) as usize;
            crc = CRC64_TABLE[index] ^ (crc >> 8);
        }
    }

    Ok(!crc)
}

fn compute_crc64_from_file(path: &Path) -> io::Result<u64> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    compute_crc64_from_reader(reader)
}

fn compute_crc64_from_stdin() -> io::Result<u64> {
    let stdin = io::stdin();
    let handle = stdin.lock();
    compute_crc64_from_reader(handle)
}

fn format_checksum(sum: u64, uppercase: bool, hex: bool) -> String {
    if hex {
        format!("{sum:016x}")
    } else if uppercase {
        format!("{sum:016X}")
    } else {
        let bytes = sum.to_be_bytes();
        BASE64_STANDARD.encode(bytes)
    }
}

fn main() {
    let cli = Cli::parse();
    let mut outputs = Vec::new();

    for pattern in &cli.patterns {
        if pattern == "-" {
            match compute_crc64_from_stdin() {
                Ok(sum) => {
                    let formatted = format_checksum(sum, cli.uppercase, cli.hex);
                    if cli.json {
                        outputs.push(ResultEntry {
                            file: "stdin".to_string(),
                            crc64: formatted,
                        });
                    } else {
                        println!("{formatted}  stdin");
                    }
                }
                Err(err) => {
                    eprintln!("error reading from stdin: {err}");
                }
            }
            continue;
        }

        match glob(pattern) {
            Ok(paths) => {
                for entry in paths {
                    match entry {
                        Ok(path) => {
                            let display = path.display().to_string();
                            match compute_crc64_from_file(&path) {
                                Ok(sum) => {
                                    let formatted = format_checksum(sum, cli.uppercase, cli.hex);
                                    if cli.json {
                                        outputs.push(ResultEntry {
                                            file: display,
                                            crc64: formatted,
                                        });
                                    } else {
                                        println!("{formatted}  {display}");
                                    }
                                }
                                Err(err) => {
                                    eprintln!("error on {display}: {err}");
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("glob error for pattern {pattern}: {err}");
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("invalid pattern {pattern}: {err}");
            }
        }
    }

    if cli.json {
        match serde_json::to_string_pretty(&outputs) {
            Ok(json) => println!("{json}"),
            Err(err) => {
                eprintln!("json marshal error: {err}");
                std::process::exit(1);
            }
        }
    }
}
