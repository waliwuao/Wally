# Wally - 规范化 Git 命令行工具集

Wally 是一个基于 Rust 开发的 Git 助手，旨在通过抽象复杂的 Git 指令来建立标准化的交互式工作流。它专注于强制执行开发规范、保持线性的提交历史，并为开发者提供直观的操作反馈。

---

## 核心功能

*   **自动化安全同步**：更新代码时自动完成 Stash、Rebase 和 Pop，确保工作区安全且历史线形。
*   **约定式提交**：强制执行 Conventional Commits 规范，自动构建标准化的提交信息。
*   **多维回滚系统**：不仅支持基于 Commit Log 的重置，还支持基于 Reflog 的操作级撤销（Undo）。
*   **项目活动分析**：内置统计引擎，直观展示开发频率、代码贡献量及高频修改文件。
*   **项目模板系统**：通过自定义 JSON 模板快速初始化标准化的开发环境。

---

## 安装说明

### 环境要求

*   已安装 Rust 编译环境 (Cargo)
```bash
curl https://sh.rustup.rs -sSf | sh

```
*   系统中已安装 Git

### 编译与安装

在项目根目录下执行：

```bash
cargo build --release
cargo install --path .
```

---

## 命令指南

### 1. 项目初始化 (Project Setup)

*   **`wally new [项目名]`**
    从预设模板初始化仓库。支持交互式向导选择模板，并强制设置 `main` 为默认分支。
*   **`wally context`**
    扫描项目并生成 `info/context.md`。整合目录树和追踪文件的源码，专为 AI 代码分析设计的上下文生成工具。

### 2. 日常开发与同步 (Development & Sync)

*   **`wally update`**
    执行最安全的同步流程：自动暂存修改 -> 重命名 master 为 main -> `pull --rebase` -> 还原暂存 -> 交互式冲突引导。
*   **`wally commit`**
    全能提交助手：交互式勾选暂存文件 -> 选择提交类型（feat, fix 等） -> 填写作用域与描述 -> 自动询问并执行 Push。
*   **`wally branch`**
    交互式分支管理中心：
    *   **Switch**: 快速切换分支。
    *   **Create**: 创建带规范前缀（feat/, fix/ 等）的新分支。
    *   **Merge**: 安全地将指定分支合并到当前分支。
    *   **Delete**: 批量清理已合并或冗余的本地分支。
*   **`wally reset`**
    双模式回滚工具：
    *   **Undo Recent Actions**: 基于 `reflog` 撤销最近的 Git 操作（如错误的 rebase 或 merge）。
    *   **Reset to Commit**: 基于提交历史执行 `HARD reset`。
*   **`wally stats`**
    项目活跃度看板：展示过去 7 天的提交频率、行数增减统计，以及过去 30 天内修改最频繁的 Top 5 文件。

### 3. 模板管理 (Template Management)

*   **`wally list`**：列出所有已安装的项目模板及其描述。
*   **`wally install [文件路径]`**：从本地 JSON 文件导入自定义模板。
*   **`wally uninstall [模板名]`**：从系统中移除特定的自定义模板。

---

## 模板配置参考

Wally 使用 JSON 格式定义模板。示例如下：

```json
{
  "template_name": "rust-basic",
  "description": "基础 Rust 项目结构",
  "files": {
    "Cargo.toml": "[package]\nname = \"{{name}}\"\nversion = \"0.1.0\"\nedition = \"2021\"",
    "src/main.rs": "fn main() {\n    println!(\"Hello, world!\");\n}",
    ".gitignore": "target/\nCargo.lock",
    "README.md": "# 项目标题"
  }
}
```

---

## 设计哲学

1.  **线性历史**：通过强制 `rebase` 消除无谓的 merge commits，保持清晰的线性演进过程。
2.  **操作安全感**：通过自动 `stash` 和 `reflog` 可视化，降低误操作导致代码丢失的风险。
3.  **减少上下文切换**：将统计、上下文生成、规范检查集成在工具内，让开发者尽量留在终端。
