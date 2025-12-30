# jj-ai

A standalone CLI tool that generates commit descriptions for [Jujutsu (jj)](https://github.com/jj-vcs/jj) repositories using an LLM.

## Installation

```bash
cargo install --path .
```

Or build manually:

```bash
cargo build --release
# Binary at target/release/jj-ai
```

## Configuration

Set the following environment variables:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `OPENAI_API_KEY` | Yes | — | Your OpenAI API key |
| `JJAI_MODEL` | No | `gpt-4o-mini` | OpenAI model to use |
| `JJAI_MAX_TOKENS` | No | `256` | Max tokens for generated descriptions |

## Usage

Run from within a jj repository:

```bash
# Generate description for current commit
jj-ai

# Generate description for a specific revision
jj-ai abc123

# Preview without applying
jj-ai --dry-run
```
