# jj-ai

A plugin for JJ that adds AI-powered utility commands.

Commands:
  - `describe`: Automatically generate a commit message following a standard (`generic`, `conventional`, `gitmoji`)


## Installation

Install the binary:

```bash
cargo install jj-ai
```

Add plugin to jj:

```bash
jj config set --user aliases.ai '["util", "exec", "--", "jj-ai"]'
```

## Configuration

Set the following environment variables:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `OPENROUTER_API_KEY` | Yes | — | Your OpenRouter API key |
| `JJAI_MODEL` | No | `openai/gpt-4o-mini` | OpenRouter model to use |

## Usage

Generate description for current commit:

```bash
jj ai describe
```

Generate description for a specific revision(s):

```bash
# jj ai describe -r <revset>
jj ai describe -r @     # <- Generate commit message for the current commit (default)
jj ai describe -r ..    # <- Generate commit messages for all commits
```

By default, `jj ai describe` will skip generating messages for commits that already have one.
You can overwrite this behaviour with the `--overwrite` flag.

```bash
jj ai describe --overwrite
```

Preview generated commit without applying:

```bash
jj ai describe --dry-run
```
