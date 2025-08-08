# Gemini-Keychecker

A high-performance tool to validate Google Gemini API keys with batch processing capabilities.

## Features

- **API key validation**: Validates Google Gemini API keys efficiently
- **High-performance processing**: High concurrency with HTTP/2 multiplexing and low memory footprint
- **Backup support**: Automatically creates backup files for all processed keys
- **Proxy support**: HTTP/HTTPS proxy with authentication
- **Smart retry**: Configurable retry mechanism for failed requests

## Installation

### Download Pre-built Binaries

Download the latest release from [GitHub Releases](https://github.com/Yoo1tic/Gemini-Keychecker/releases):

- **Linux**: `gemini-keychecker-linux-x86_64`
- **Windows**: `gemini-keychecker-windows-x86_64.exe`

### Build from Source

```bash
git clone https://github.com/Yoo1tic/Gemini-Keychecker.git
cd Gemini-Keychecker
cargo build --release
```

## Usage

### Quick Start

1. Create a `keys.txt` file with one API key per line
2. Run the tool:

```bash
./gemini-keychecker
```

The tool will:

- Validate all API keys and create output files for valid keys
- Generate a backup file (`backup_keys.txt`) containing all processed keys

### Configuration

The tool supports three configuration methods (in order of precedence):

1. **Command line arguments**
2. **Configuration file** (`Config.toml`)

### Configuration File

Create a `Config.toml` file in the same directory. See `Config.toml.example` for reference.

### Command Line Options

```bash
Options:
  -i, --input-path <INPUT_PATH>      Input file containing API keys [default: keys.txt]
  -b, --backup-path <BACKUP_PATH>    Backup file for all processed keys [default: backup_keys.txt]
  -u, --api-host <API_HOST>          API host URL [default: https://generativelanguage.googleapis.com/]
  -t, --timeout-sec <TIMEOUT_SEC>    Request timeout in seconds [default: 15]
  -c, --concurrency <CONCURRENCY>   Max concurrent requests [default: 50]
  -r, --max-retries <MAX_RETRIES>   Max retry attempts for failed requests [default: 2]
  -x, --proxy <PROXY>               Proxy URL (http://user:pass@host:port)
  -h, --help                        Print help
```

### Examples

#### Basic Usage

```bash
# Use default settings
./gemini-keychecker

# Custom input file
./gemini-keychecker -i my_api_keys.txt

# Custom backup location
./gemini-keychecker -b /path/to/backup/keys.txt
```

#### Performance Tuning

```bash
# High concurrency for fast validation
./gemini-keychecker -c 100 -t 10

# Conservative settings for rate-limited environments
./gemini-keychecker -c 10 -t 30 -r 5
```

#### Proxy Configuration

```bash
# HTTP proxy without authentication
./gemini-keychecker -x http://proxy.company.com:8080

# HTTP proxy with authentication
./gemini-keychecker -x http://username:password@proxy.company.com:8080

# HTTPS proxy
./gemini-keychecker -x https://user:pass@secure-proxy.com:8443
```

## Input Format

Create a text file with one API key per line:

```
AIzaSyxxxxxxxxxxxxxxefghij
AIzaSyxxxxxxxxxxxxxxefghij
AIzaSyxxxxxxxxxxxxxxefghij
```

## Performance

- **High-performance**: Optimized concurrent processing with configurable limits
- **HTTP/2 Multiplexing**: Enhanced connection efficiency
- **Smart Retry**: Automatic retry with exponential backoff
- **Low Memory Usage**: Streaming processing minimizes resource consumption
