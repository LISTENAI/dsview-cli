<h1 align="center">DSView CLI</h1>

<p align="center">无需 DSView 图形界面，即可脚本化使用 DSLogic Plus 采集和协议解码。</p>

<p align="center">
  <a href="README.md">English</a> | <a href="README.zh-CN.md">简体中文</a>
</p>

<p align="center">
  <a href="https://github.com/LISTENAI/dsview-cli/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/LISTENAI/dsview-cli/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/LISTENAI/dsview-cli/releases/latest"><img alt="Latest Release" src="https://img.shields.io/github/v/release/LISTENAI/dsview-cli"></a>
  <a href="LICENSE"><img alt="License" src="https://img.shields.io/github/license/LISTENAI/dsview-cli"></a>
</p>

DSView CLI 是面向 DSLogic Plus 设备的命令行工具。它把 DSView/libsigrok 的设备栈封装成可脚本化的流程，用于自动化采集、VCD 导出、协议解码器检查和离线解码。

## 功能特性

- 对 DSLogic Plus 执行有限长度逻辑分析采集。
- 导出 VCD 波形文件和 JSON 元数据，便于后续工具处理。
- 通过命令行查看 DSView 协议解码器。
- 从 JSON 逻辑采样输入执行离线协议解码。
- 发布为自包含 release bundle，内含原生运行时、固件资源、解码器脚本和 Python 运行时。
- 默认输出 JSON，便于自动化；同时提供适合人工阅读的 text 输出。

## 平台支持

| 平台 | 架构 | Release bundle | 一行安装脚本 |
| --- | --- | --- | --- |
| Linux | x86_64, ARM64 | 支持 | 支持 |
| macOS | Intel, Apple Silicon | 支持 | 支持 |
| Windows | x86_64, ARM64 | 支持 | 支持 |

当前设备支持范围聚焦于 **DSLogic Plus**。

## 安装

### macOS 和 Linux

安装最新发布的 release bundle 到 `~/.local/opt/dsview-cli`，并在 `~/.local/bin` 创建启动器：

```bash
curl -fsSL https://raw.githubusercontent.com/LISTENAI/dsview-cli/refs/heads/master/scripts/install.sh | sh
```

安装指定 release tag：

```bash
curl -fsSL https://raw.githubusercontent.com/LISTENAI/dsview-cli/refs/heads/master/scripts/install.sh | sh -s -- --version v1.2.3
```

常用安装选项：

```bash
curl -fsSL https://raw.githubusercontent.com/LISTENAI/dsview-cli/refs/heads/master/scripts/install.sh | sh -s -- --prefix ~/.local/opt/dsview-cli --bin-dir ~/.local/bin --version v1.2.3
curl -fsSL https://raw.githubusercontent.com/LISTENAI/dsview-cli/refs/heads/master/scripts/install.sh | sh -s -- --dry-run
```

安装器会保持 release bundle 的目录结构不变，并执行以下冒烟检查：

```bash
dsview-cli --version
dsview-cli devices list --help
dsview-cli decode list --format json
```

### Windows

安装最新发布的 release bundle 到 `%LOCALAPPDATA%\Programs\dsview-cli`，创建启动器，并把启动器目录追加到用户 `PATH`：

```powershell
irm https://raw.githubusercontent.com/LISTENAI/dsview-cli/refs/heads/master/scripts/install.ps1 | iex
```

安装指定 release tag：

```powershell
$script = Join-Path $env:TEMP "dsview-cli-install.ps1"
irm https://raw.githubusercontent.com/LISTENAI/dsview-cli/refs/heads/master/scripts/install.ps1 -OutFile $script
powershell -ExecutionPolicy Bypass -File $script -Version v1.2.3
```

安装后请重启 shell，让更新后的 `PATH` 生效。

### 手动下载

从 GitHub Releases 下载对应目标平台的 `.tar.gz` bundle，使用 release checksum 文件校验后解压：

```bash
VERSION=v1.2.3
TARGET=x86_64-unknown-linux-gnu
curl -LO "https://github.com/LISTENAI/dsview-cli/releases/download/$VERSION/dsview-cli-$VERSION-$TARGET.tar.gz"
curl -LO "https://github.com/LISTENAI/dsview-cli/releases/download/$VERSION/dsview-cli-$VERSION-SHA256SUMS.txt"
sha256sum --check --ignore-missing "dsview-cli-$VERSION-SHA256SUMS.txt"
tar -xzf "dsview-cli-$VERSION-$TARGET.tar.gz"
```

如果系统没有 `sha256sum`，可以使用 `shasum -a 256`，并把 archive hash 与 checksum 文件中的对应条目进行比较。

可以直接从解压后的 bundle 根目录运行 `dsview-cli`，也可以把一个 wrapper 放到 `PATH` 中。请保持解压后的目录完整。不要只把可执行文件复制到其他目录；CLI 会根据可执行文件所在位置查找运行时、解码器脚本、Python 和固件资源。

## 快速开始

### 列出设备

```bash
dsview-cli devices list --format text
```

示例输出：

```text
Device 1 (handle: 1)
  Model: DSLogic Plus
  ID: dslogic-plus
  Native: DSLogic Plus [0]
```

### 查看采集选项

```bash
dsview-cli devices options --handle 1 --format text
```

建议在采集前运行该命令，用于查看 operation mode、stop option、channel mode、threshold 和 filter 支持的 token。

### 采集波形

```bash
dsview-cli capture \
  --handle 1 \
  --sample-rate-hz 100000000 \
  --sample-limit 2048 \
  --channels 0,1,2,3 \
  --output capture.vcd
```

该命令会以 100 MHz 从通道 0-3 采集 2048 个样本，并写出：

- `capture.vcd` - Value Change Dump 格式的波形数据。
- `capture.json` - 元数据 sidecar，包含采集配置、完成状态和产物路径。

采集选项 override 使用 `devices options` 报告的 token：

```bash
dsview-cli capture \
  --handle 1 \
  --operation-mode buffer \
  --sample-rate-hz 100000000 \
  --sample-limit 2048 \
  --channels 0,1 \
  --output capture.vcd
```

### 列出协议解码器

```bash
dsview-cli decode list --format text
```

检查某个解码器，查看它的通道、选项、注释和堆叠兼容性：

```bash
dsview-cli decode inspect i2c --format json
```

### 校验并执行离线解码

创建解码配置：

```json
{
  "version": 1,
  "decoder": {
    "id": "i2c",
    "channels": {
      "scl": 0,
      "sda": 1
    },
    "options": {}
  },
  "stack": []
}
```

使用内置解码器 registry 校验配置：

```bash
dsview-cli decode validate --config decode.json --format text
```

创建逻辑采样输入。这里使用的是低层离线解码输入格式，不是 VCD 文件，也不是 capture metadata sidecar：

```json
{
  "samplerate_hz": 100000000,
  "format": "split_logic",
  "sample_bytes": [3, 1, 3, 2, 3],
  "unitsize": 1
}
```

从解码配置和采样输入执行离线解码：

```bash
dsview-cli decode run \
  --config decode.json \
  --input logic-input.json \
  --output decode-report.json \
  --format json
```

离线解码输入包含 `samplerate_hz`、`format`、`sample_bytes`、`unitsize` 和可选的 `logic_packet_lengths` 等字段。`sample_bytes` 是 JSON 字节数组；对 `split_logic` 输入来说，每个字节是一个打包后的逻辑采样。请使用 `decode inspect` 确认解码器需要的 channel ID 和 option 名称。

## 命令参考

### `devices list`

列出连接的 DSLogic Plus 设备。

```bash
dsview-cli devices list [--format json|text] [--resource-dir PATH]
```

### `devices open`

按 handle 打开设备并验证初始化。

```bash
dsview-cli devices open --handle HANDLE [--format json|text] [--resource-dir PATH]
```

### `devices options`

从设备读取采集选项能力。

```bash
dsview-cli devices options --handle HANDLE [--format json|text] [--resource-dir PATH]
```

### `capture`

执行有限长度采集并导出 VCD 文件。

```bash
dsview-cli capture \
  --handle HANDLE \
  --sample-rate-hz HZ \
  --sample-limit SAMPLES \
  --channels IDX[,IDX...] \
  --output PATH.vcd \
  [--metadata-output PATH.json] \
  [--operation-mode TOKEN] \
  [--stop-option TOKEN] \
  [--channel-mode TOKEN] \
  [--threshold-volts VOLTS] \
  [--filter TOKEN] \
  [--wait-timeout-ms MS] \
  [--poll-interval-ms MS] \
  [--format json|text] \
  [--resource-dir PATH]
```

### `decode list`

列出内置协议解码器。

```bash
dsview-cli decode list [--format json|text] [--decode-runtime PATH] [--decoder-dir PATH]
```

### `decode inspect`

查看单个解码器描述。

```bash
dsview-cli decode inspect DECODER_ID [--format json|text] [--decode-runtime PATH] [--decoder-dir PATH]
```

### `decode validate`

只校验 JSON 解码配置，不运行解码会话。

```bash
dsview-cli decode validate --config PATH [--format json|text] [--decode-runtime PATH] [--decoder-dir PATH]
```

### `decode run`

运行离线解码会话，并可选择把标准报告写入文件。

```bash
dsview-cli decode run \
  --config PATH \
  --input PATH \
  [--output PATH] \
  [--format json|text] \
  [--decode-runtime PATH] \
  [--decoder-dir PATH]
```

## Release Bundle 结构

只要保持解压后的目录完整，release archive 就是可重定位的：

```text
dsview-cli-<version>-<target>/
|-- dsview-cli[.exe]
|-- runtime/
|   `-- libdsview_runtime.so | libdsview_runtime.dylib | dsview_runtime.dll
|-- decode-runtime/
|   `-- libdsview_decode_runtime.so | libdsview_decode_runtime.dylib | dsview_decode_runtime.dll
|-- decoders/
|   `-- DSView protocol decoder scripts
|-- python/
|   `-- protocol decoding 使用的 Python runtime 和标准库
|-- resources/
|   |-- DSLogicPlus.fw
|   |-- DSLogic.fw
|   |-- DSLogicPlus.bin
|   `-- DSLogicPlus-pgl12.bin
`-- Windows 需要时包含 *.dll, python*.dll, vcruntime*.dll
```

内置 Python runtime 已针对分发体积做裁剪。测试包、`ensurepip`、`tkinter` 等 GUI 模块以及其他不必要文件会被省略。

平台说明：

- Linux bundle 在 `python/lib` 下包含链接到的 `libpython*.so*`，并让 decode runtime 从该位置加载。
- macOS bundle 在 `python/` 下包含所需的 Python framework 或 dylib 内容，会把 Python 链接改写为 bundle-relative 路径，并对修改过的运行时文件做 ad-hoc signing。
- Windows bundle 在根目录包含 Python DLL，在 `python/` 下包含 Python 库内容。

## 从源码构建

克隆仓库和子模块：

```bash
git clone --recursive https://github.com/LISTENAI/dsview-cli.git
cd dsview-cli
```

安装原生依赖。

Linux (Ubuntu/Debian)：

```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  cmake \
  pkg-config \
  libglib2.0-dev \
  libusb-1.0-0-dev \
  libfftw3-dev
```

macOS：

```bash
brew install cmake pkg-config glib libusb fftw
```

Windows + vcpkg：

```powershell
vcpkg install glib:x64-windows libusb:x64-windows fftw3:x64-windows pkgconf:x64-windows
```

Windows ARM64 源码构建请使用 `arm64-windows` triplet。

构建并测试：

```bash
cargo build --release
cargo test --workspace
```

源码构建适合开发，但它不等价于完整可重定位的 release archive。Release archive 由打包流程生成，确保原生运行时、解码器脚本、固件资源和 Python runtime 被一起组装和验证。

## 开发说明

上游 DSView 项目作为 git submodule 引入，提供本 CLI 使用的原生设备通信栈。正常开发中请把该 submodule 视为依赖代码。

工作区职责：

- `dsview-cli` - 命令行界面和面向用户的命令契约。
- `dsview-core` - discovery、capture、decode 和 validation 流程的安全 Rust 编排层。
- `dsview-sys` - DSView/libsigrok 组件的原生绑定和 C/C++ 集成边界。

原生边界被有意隔离，以便把 unsafe 和平台相关行为限制在较窄的 Rust API 后面。

## 打包和 CI

CI 会为所有支持的 target 构建、打包 release archive、验证 archive 结构，并从解压后的 bundle 执行命令发现冒烟测试。

验证内容包括：

- 可执行文件、采集 runtime、解码 runtime、解码器脚本、固件资源和 Python runtime 均存在。
- Unix decode runtime 链接会解析到 bundle 内置 Python 路径。
- Windows bundle 包含原生 DLL 和 Python DLL 依赖。
- `dsview-cli --help`、`devices list --help` 和 `decode list --format json` 能从解压后的 bundle 中正常运行。

Windows ARM64 CI 使用 GitHub-hosted `windows-11-arm` runner。该基础设施在 2026 年 4 月仍属于 public preview，runner 镜像细节后续可能变化。

## 故障排查

### 找不到 `libpython*.so*` 或 Python framework

请使用当前完整 release bundle，并保持解压后的目录完整。这个错误通常意味着使用了旧 bundle，或者只把可执行文件复制到了其他目录。

### `decode list` 失败或没有解码器

检查 `decode-runtime/`、`decoders/` 和 `python/` 是否仍在可执行文件旁边。如果使用 `--decode-runtime` 或 `--decoder-dir`，两者必须匹配同一套 decoder runtime 构建。

### Linux 普通用户无法访问 USB 设备

安装 udev 规则，重载规则后重新插拔设备：

```bash
printf '%s\n' 'SUBSYSTEM=="usb", ATTRS{idVendor}=="2a0e", MODE="0666"' | \
  sudo tee /etc/udev/rules.d/99-dsview-cli.rules >/dev/null
sudo udevadm control --reload-rules
sudo udevadm trigger
```

### Windows 安装后找不到命令

重启 shell，让更新后的用户 `PATH` 生效；或者使用生成的 launcher 完整路径运行。

## 贡献

欢迎提交 issue 和 pull request。请保持面向 JSON 消费者的命令输出稳定，并在行为变化时同步更新文档。

## 许可证

本仓库中的 Rust 代码使用 Apache License, Version 2.0。详见 `LICENSE`。

上游 DSView submodule 和内置第三方组件保留各自许可证。Vendored DSView/libsigrok 源码随附其原有 license 文件，不会被本仓库重新授权。
