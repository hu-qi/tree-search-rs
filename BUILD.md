# 构建指南

## 快速開始

### Linux

```bash
# 下載預構建二進制
wget https://api.gitcode.com/huqi/tree-search-rs/releases/download/v1.3.0/treesearch-linux-x86_64
chmod +x treesearch-linux-x86_64
./treesearch-linux-x86_64 --help
```

### Windows

```powershell
# 自構建（需安裝 Rust）
git clone https://gitcode.com/huqi/tree-search-rs.git
cd tree-search-rs
cargo build --release
.\target\release\treesearch.exe --help
```

### macOS

```bash
# 自構建（需安裝 Rust）
git clone https://gitcode.com/huqi/tree-search-rs.git
cd tree-search-rs
cargo build --release
./target/release/treesearch --help
```

## 安裝 Rust

### Linux / macOS

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Windows

下載並運行 [rustup-init.exe](https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe)

## 多平臺構建

項目支持以下平臺：

| 平臺 | 目標 | 狀態 |
|------|------|------|
| Linux x86_64 | `x86_64-unknown-linux-gnu` | ✅ 預構建 |
| Windows x86_64 | `x86_64-pc-windows-msvc` | ✅ 支持 |
| macOS x86_64 | `x86_64-apple-darwin` | ✅ 支持 |

## GitHub Actions

項目包含 `.github/workflows/release.yml`，支持自動多平臺構建。

遷移到 GitHub 後，推送 tag 即可觸發：

```bash
git tag v1.4.0
git push origin v1.4.0
```

## 依賴說明

所有依賴均為跨平臺：

- `tantivy` - 全文搜索
- `lopdf` - PDF 解析
- `docx-rs` - DOCX 解析
- `jieba-rs` - 中文分詞
- `ignore` - ripgrep 文件遍歷

無平臺特定代碼。
