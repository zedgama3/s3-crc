// main.go
package main

import (
	"encoding/base64"
	"encoding/binary"
	"encoding/json"
	"flag"
	"fmt"
	"hash/crc64"
	"io"
	"log"
	"os"
	"path/filepath"
)

func computeCRC64(path string) (uint64, error) {
	const nvmePoly = 0x9A6C9329AC4BC9B5 // reflected NVMe polynomial
	var r io.Reader
	if path == "-" {
		r = os.Stdin
	} else {
		f, err := os.Open(path)
		if err != nil {
			return 0, err
		}
		defer f.Close()
		r = f
	}

	table := crc64.MakeTable(nvmePoly)
	crc := ^uint64(0)           // initial value 0xFFFFFFFFFFFFFFFF
	buf := make([]byte, 32<<10) // 32 KiB buffer

	for {
		n, err := r.Read(buf)
		if n > 0 {
			for _, b := range buf[:n] {
				crc = table[byte(crc)^b] ^ (crc >> 8)
			}
		}
		if err == io.EOF {
			break
		}
		if err != nil {
			return 0, err
		}
	}

	return ^crc, nil // final XOR
}

func main() {
	uppercase := flag.Bool("uppercase", false, "hex uppercase")
	asJSON := flag.Bool("json", false, "JSON output")
	asHex := flag.Bool("hex", false, "hex output")
	flag.Usage = func() {
		fmt.Fprintf(os.Stderr, "Usage: %s [options] <file-glob>...\n", os.Args[0])
		flag.PrintDefaults()
		fmt.Fprintln(os.Stderr, "Compute CRC64 checksum (CRC64-NVMe) for files matching the glob patterns.")
		fmt.Fprintln(os.Stderr, "This is compatible with the CRC64 hash used by AWS S3.")
		fmt.Fprintln(os.Stderr, "If no files match, read from stdin.")
	}
	flag.Parse()
	if flag.NArg() < 1 {
		flag.Usage()
		os.Exit(1)
	}
	log.SetFlags(0)

	type result struct {
		File  string `json:"file"`
		CRC64 string `json:"crc64"`
	}
	var outputs []result

	for _, pattern := range flag.Args() {
		if pattern == "-" {
			// Special case: read from stdin
			sum, err := computeCRC64("-")
			if err != nil {
				log.Printf("error reading from stdin: %v\n", err)
				continue
			}
			var out string
			switch {
			case *asHex:
				out = fmt.Sprintf("%016x", sum)
			case *uppercase:
				out = fmt.Sprintf("%016X", sum)
			default:
				buf := make([]byte, 8)
				binary.BigEndian.PutUint64(buf, sum)
				out = base64.StdEncoding.EncodeToString(buf)
			}
			if *asJSON {
				outputs = append(outputs, result{File: "stdin", CRC64: out})
			} else {
				fmt.Printf("%s  stdin\n", out)
			}
			continue
		}

		matches, _ := filepath.Glob(pattern)
		for _, path := range matches {
			sum, err := computeCRC64(path)
			if err != nil {
				log.Printf("error on %s: %v\n", path, err)
				continue
			}
			var out string
			switch {
			case *asHex:
				out = fmt.Sprintf("%016x", sum)
			case *uppercase:
				out = fmt.Sprintf("%016X", sum)
			default:
				buf := make([]byte, 8)
				binary.BigEndian.PutUint64(buf, sum)
				out = base64.StdEncoding.EncodeToString(buf)
			}
			if *asJSON {
				outputs = append(outputs, result{File: path, CRC64: out})
			} else {
				fmt.Printf("%s  %s\n", out, path)
			}
		}
	}

	if *asJSON {
		data, err := json.MarshalIndent(outputs, "", "  ")
		if err != nil {
			log.Fatalf("json marshal error: %v", err)
		}
		fmt.Println(string(data))
	}
}
