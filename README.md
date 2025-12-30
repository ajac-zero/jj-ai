# jjai

A [Jujutsu (jj)](https://github.com/jj-vcs/jj) extension that generates commit descriptions using an LLM.

## How It Works

`jj-ai` adds an `ai` subcommand to `jj` that generates commit descriptions using OpenAI's API based on the commit's diff.

## Installation

```bash
cargo install --path .
```

Or build manually:

```bash
cargo build --release
# Binary at target/release/jj-ai
```

Ensure `jj-ai` is in your `PATH` so that `jj` can discover it.

## Configuration

Set the following environment variables:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `OPENAI_API_KEY` | Yes | — | Your OpenAI API key |
| `JJAI_MODEL` | No | `gpt-4o-mini` | OpenAI model to use |
| `JJAI_MAX_TOKENS` | No | `256` | Max tokens for generated descriptions |

## Usage

```bash
# Generate description for current commit
jj ai

# Generate description for a specific revision
jj ai abc123

# Preview without applying
jj ai --dry-run
```
