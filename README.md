# Gemini-Keychecker

A tool to validate Google Gemini API keys.

## Usage

### Basic Usage

```bash
# Validate keys from keys.txt and save valid ones to output_keys.txt
./target/release/gemini-keychecker

# Specify custom input and output files
./target/release/gemini-keychecker -i my_keys.txt -o valid_keys.txt
```

### Advanced Usage

```bash
# Use proxy with authentication
./target/release/gemini-keychecker -x http://username:password@proxy.example.com:8080

# Adjust concurrency and timeout
./target/release/gemini-keychecker -c 50 -t 30

# Use custom API host
./target/release/gemini-keychecker -u https://custom-api.googleapis.com/
```

## Command Line Options

```
Options:
  -i, --input-path <INPUT_PATH>    Input file containing API keys [default: keys.txt]
  -o, --output-path <OUTPUT_PATH>  Output file for valid keys [default: output_keys.txt]
  -u, --api-host <API_HOST>        API host URL [default: https://generativelanguage.googleapis.com/]
  -t, --timeout-sec <TIMEOUT_SEC>  Request timeout in seconds [default: 60]
  -c, --concurrency <CONCURRENCY>  Max concurrent requests [default: 30]
  -x, --proxy <PROXY>              Proxy URL (supports http://user:pass@host:port)
  -h, --help                       Print help
  -V, --version                    Print version
```

## Input Format

Create a text file with one API key per line

## Proxy Configuration

The tool supports HTTP/HTTPS proxies with optional authentication:

```bash
# HTTP proxy without authentication
./target/release/gemini-keychecker -x http://proxy.example.com:8080

# HTTP proxy with authentication
./target/release/gemini-keychecker -x http://username:password@proxy.example.com:8080

# HTTPS proxy with authentication
./target/release/gemini-keychecker -x https://user:pass@secure-proxy.com:8443
```
