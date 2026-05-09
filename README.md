# aicmd

The aicmd poriveds set of Linux tools via uutils, you can use linux like tools in Windows. It automatically injecting [uutils/coreutils](https://github.com/uutils/coreutils) Linux commands into Windows `PATH`.

Designed to solve the problem of AI agent using Unix-like commands as tools to execute in Windows.

## Features

- **Zero-config Linux toolchain** — auto-downloads uutils coreutils on first run (83 commands: `ls`, `cat`, `head`, `tail`, `grep`, `wc`, `sort`, `uniq`, `cut`, `tr`, `sed`-like via `cut`/`tr`, etc.)
- **Two execution modes** — detects OpenSSH's calling convention automatically:
  - `aicmd.exe -c <command>` — non-interactive single command (SSH default)
  - `aicmd.exe` — interactive persistent shell
- **60-second timeout** — prevents runaway commands in non-interactive mode
- **English locale** — forces `LC_ALL=C` for consistent output parsing
- **PATH injection** — tools directory prepended to `PATH`, transparent to the agent

## How It Works

```
SSH Agent connects → OpenSSH invokes: aicmd.exe -c "ls -la /d/project"
                                              │
                                              ▼
                                     aicmd.exe detects -c mode
                                              │
                                              ▼
                              Checks C:\agent_tools\coreutils\ls.exe
                              ├─ Not found → auto-download uutils 0.8.0
                              └─ Found → skip
                                              │
                                              ▼
                              Spawns: cmd.exe /c "ls -la /d/project"
                              with PATH = C:\agent_tools\coreutils;...
                                              │
                                              ▼
                              uutils ls.exe runs natively, returns output
```

## Supported Linux Commands (83)

`arch`, `basename`, `cat`, `cksum`, `comm`, `cp`, `cut`, `date`, `dd`, `df`, `dir`, `dirname`, `du`, `echo`, `env`, `expand`, `expr`, `factor`, `false`, `fmt`, `fold`, `head`, `hostname`, `join`, `link`, `ln`, `ls`, `md5sum`, `mkdir`, `mktemp`, `more`, `mv`, `nl`, `nproc`, `numfmt`, `od`, `paste`, `pathchk`, `pr`, `printenv`, `printf`, `pwd`, `readlink`, `realpath`, `rm`, `rmdir`, `seq`, `sha1sum`, `sha224sum`, `sha256sum`, `sha384sum`, `sha512sum`, `shred`, `shuf`, `sleep`, `sort`, `split`, `sum`, `sync`, `tac`, `tail`, `tee`, `test`, `touch`, `tr`, `true`, `truncate`, `tsort`, `uname`, `unexpand`, `uniq`, `unlink`, `vdir`, `wc`, `whoami`, `yes`, and more.

## Installation

### Option 1: Download from Release

Grab `aicmd.exe` from the [latest release](https://github.com/cha0s-repo/aicmd/releases/latest) and place it in a directory on your `PATH`.

### Option 2: Build from Source

```bash
cargo build --release
# Output: target/release/aicmd.exe
```

## Usage

### As SSH Default Shell

Set `aicmd.exe` as the default shell for your SSH user:

```powershell
# Add aicmd directory to system PATH
setx PATH "%PATH%;C:\path\to\aicmd"

# Set as default shell (requires admin)
# In Windows Registry or OpenSSH config
```

### Direct Usage

```bash
# Non-interactive (SSH -c mode)
aicmd.exe -c ls -la D:\project
aicmd.exe -c cat README.md
aicmd.exe -c head -20 src/main.rs

# Interactive shell
aicmd.exe
```

### Remote Execution

```bash
ssh user@windows-host "aicmd -c ls D:\\project"
```

## Technical Details

- **Language**: Rust (single binary, no runtime dependencies)
- **Toolchain**: [uutils/coreutils](https://github.com/uutils/coreutils) 0.8.0 (standalone exe per command)
- **Tools location**: `C:\agent_tools\coreutils\` (auto-created on first run)
- **Download size**: ~53 MB zip, ~127 MB extracted
- **Binary size**: ~260 KB (aicmd.exe itself)

## License

MIT
