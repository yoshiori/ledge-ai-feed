# Ledge.ai RSS Feed Generator

[![Tests](https://github.com/yoshiori/ledge-ai-feed/workflows/Tests/badge.svg)](https://github.com/yoshiori/ledge-ai-feed/actions/workflows/test.yml)
[![RSS Update](https://github.com/yoshiori/ledge-ai-feed/workflows/Update%20RSS%20Feed/badge.svg)](https://github.com/yoshiori/ledge-ai-feed/actions/workflows/update-rss.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

An automated RSS 2.0 feed generator for Ledge.ai articles, built with Rust and Test-Driven Development.

## Features

- **Automated Article Collection**: Automatically fetches latest articles from Ledge.ai
- **Full Content**: Includes complete article content in RSS feeds
- **Hourly Updates**: Automatically runs every hour via GitHub Actions
- **High Performance**: Fast and safe implementation using Rust

## RSS Feed URL

Access the RSS feed at:

```
https://yoshiori.github.io/ledge-ai-feed/rss.xml
```

## Technical Specifications

- **Language**: Rust (Edition 2021)
- **Development Method**: T-wada style TDD (Test-Driven Development)
- **Automation**: GitHub Actions (cron: hourly execution)
- **Format**: RSS 2.0

## Dependencies

- `rss` - RSS generation
- `scraper` - HTML parsing
- `reqwest` - HTTP client
- `chrono` - Date handling
- `pulldown-cmark` - Markdown → HTML conversion

## Local Usage

```bash
# Install dependencies
cargo build

# Generate RSS feed
cargo run

# Check generated feed
cat rss.xml
```

## Development

This project was built using T-wada style TDD (Test-Driven Development):

1. **Red**: Write failing tests
2. **Green**: Write code to pass tests
3. **Refactor**: Improve code quality

```bash
# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy
```

### Pre-commit Hook

A pre-commit hook is automatically installed that runs:
- Code formatting check (`cargo fmt --check`)
- Clippy lints (`cargo clippy`)
- All tests (`cargo test`)
- Compilation check (`cargo check`)

This ensures code quality and prevents commits with failing tests or linting issues.

## Automation Schedule

Automated execution via GitHub Actions:

- **Scheduled**: Runs at minute 0 of every hour (`cron: '0 * * * *'`)
- **Manual**: Can be triggered manually from GitHub UI
- **On Changes**: Runs when `src/` directory changes are pushed

## License

MIT License - See [LICENSE](LICENSE) file for details.

---

🤖 This RSS feed is automatically generated. Enjoy the latest tech news!