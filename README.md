# aicmd

Agent command executor with Linux toolchain support on Windows.

A Windows `cmd.exe` wrapper that automatically sets up [uutils/coreutils](https://github.com/uutils/coreutils) to provide Linux-like commands (ls, cat, grep, etc.) for SSH-based AI agents.

## Features

- Auto-downloads and configures uutils coreutils on first run
- Generates `.bat` proxy scripts for all supported Linux commands
- Supports both interactive and non-interactive (SSH `-c`) modes
- 60-second timeout for non-interactive command execution
- Injects Linux tools path at the front of `PATH`

## How It Works

1. On first run, downloads the latest `coreutils` release from GitHub
2. Extracts and generates `.bat` proxy scripts for each utility (ls, cat, head, tail, etc.)
3. Spawns `cmd.exe` with the tools directory prepended to `PATH`
4. SSH agents can now use familiar Linux commands transparently

## Build

```bash
cargo build --release
```

## Usage

Replace the default shell with `aicmd.exe` in your SSH server configuration, or use it directly as a command wrapper.
