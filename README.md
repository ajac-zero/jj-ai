# jjai

A [Jujutsu (jj)](https://github.com/jj-vcs/jj) wrapper that automatically generates commit descriptions using an LLM when commits are rewritten.

## How It Works

`jjai` wraps the `jj` CLI transparently. When a command rewrites a commit (same `ChangeId`, new `CommitId`) and that commit has an empty description, `jjai` generates one using OpenAI's API based on the commit's diff.

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
| `JJAI_ENABLED` | No | `true` | Set to `0` or `false` to disable |
| `JJAI_MODEL` | No | `gpt-4o-mini` | OpenAI model to use |
| `JJAI_MAX_TOKENS` | No | `256` | Max tokens for generated descriptions |

## Usage

Use `jjai` as a drop-in replacement for `jj`:

```bash
# Instead of:
jj new

# Use:
jjai new
```

All `jj` commands work identically. When commits with empty descriptions are rewritten, `jjai` automatically fills them in.

To disable temporarily:

```bash
JJAI_ENABLED=0 jjai <command>
```

## Shell Alias (Optional)

Add to your shell config:

```bash
alias jj="jjai"
```
