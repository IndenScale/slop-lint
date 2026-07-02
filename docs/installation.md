# Installation

Choose one install method.

macOS or Linux release binary:

```sh
curl -fsSL https://raw.githubusercontent.com/IndenScale/slop-lint/main/install.sh | sh
```

Windows PowerShell release binary:

```powershell
iwr https://raw.githubusercontent.com/IndenScale/slop-lint/main/install.ps1 -useb | iex
```

Or install from crates.io:

```sh
cargo install slop-lint
```

Then attach the installed binary to supported agent hooks:

```sh
slop-lint install-hooks
```

Check the installed hook state:

```sh
slop-lint hook-status
```

Run the release smoke test from this repository:

```sh
scripts/smoke-install.sh
```
