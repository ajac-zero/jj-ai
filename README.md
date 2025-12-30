# jjai

A [Jujutsu (jj)](https://github.com/jj-vcs/jj) extension that generates commit descriptions using an LLM.

## How It Works

`jjai` adds an `ai describe` subcommand to `jj` that generates commit descriptions using OpenAI's API based on the commit's diff.

## Installation

```bash
cargo install --path .
```

Or build manually:

```bash
cargo build --release
# Binary at target/release/jjai
```

## Configuration

Set the following environment variables:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `OPENAI_API_KEY` | Yes | — | Your OpenAI API key |
| `JJAI_MODEL` | No | `gpt-4o-mini` | OpenAI model to use |
| `JJAI_MAX_TOKENS` | No | `256` | Max tokens for generated descriptions |

## Usage

Use `jjai` alongside `jj` for AI-powered commit descriptions:

```bash
# Generate description for current commit
jjai ai

# Generate description for a specific revision
jjai ai abc123

# Preview without applying
jjai ai --dry-run

# All standard jj commands work too
jjai log
jjai status
```

## Shell Alias (Optional)

Add to your shell config:

```bash
alias jj="jjai"
```
