# 安装

任选一种安装方式。

macOS 或 Linux release 二进制：

```sh
curl -fsSL https://raw.githubusercontent.com/IndenScale/slop-lint/main/install.sh | sh
```

Windows PowerShell：

```powershell
iwr https://raw.githubusercontent.com/IndenScale/slop-lint/main/install.ps1 -useb | iex
```

从 crates.io 安装：

```sh
cargo install slop-lint
```

然后把已安装的二进制接入支持的 Agent hook：

```sh
slop-lint install-hooks
```

查看 hook 状态：

```sh
slop-lint hook-status
```

从仓库运行发布 smoke：

```sh
scripts/smoke-install.sh
```
