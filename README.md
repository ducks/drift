# drift

Repo drift auditor. Checks for stale configs, version mismatches, dead code
markers, and CI/local drift.

## Usage

```bash
# Run audit on current directory
drift

# Run audit on specific directory
drift /path/to/repo

# Output as JSON (for CI integration)
drift --json
```

## Checks

- **Stale configs**: Finds backup files (.old, .bak, .tmp, .swp, .orig)
- **Version mismatches**: Detects conflicts between rust-toolchain.toml and
  Cargo.toml, .nvmrc and package.json engines
- **Dead code markers**: Finds TODO, FIXME, XXX, HACK comments in source files
- **Git drift**: Reports uncommitted changes and untracked files
- **Gitignore drift**: Finds .gitignore entries that don't match any files

## Installation

```bash
cargo install --path .
```

## Built with lok

This project was built entirely through autonomous AI collaboration using
[lok](https://github.com/ducks/lok), a multi-LLM orchestration tool.

The development process:

1. **lok debate** decided what to build (repo drift auditor won consensus)
2. **lok spawn** planned the initial scaffold with parallel agents
3. **lok hunt --issues** found bugs and created GitHub issues
4. **lok pick-and-fix** autonomously fixes issues via multi-backend consensus
5. **lok review-pr** reviews PRs before merge

The robots are building the tools, finding bugs in their own code, fixing them,
and reviewing each other's work.

## License

MIT
