这是一份为 `wally` 编写的专业中文版 `README.md`。内容保持了与英文版一致的结构和专业度，同时针对中文开发者的阅读习惯进行了优化。

你可以将此文件保存为 `README_CN.md`，或者直接作为主 `README.md` 使用。

***

# Wally

**一个基于 Rust 编写的友好、交互式且功能强大的 Git 命令行伴侣。**

Wally 旨在简化您的 Git 工作流，强制执行最佳实践（如约定式提交 Conventional Commits），并弥合复杂 Git 命令与开发者生产力之间的鸿沟。无论您是 Git 初学者还是希望减少击键次数的资深开发者，Wally 都能为您提供帮助。

它作为一个透明的 Git 包装器运行，提供交互式的终端界面 (TUI)，同时始终向您展示正在执行的底层命令，让您对操作了如指掌。

## ✨ 核心特性

*   **🚀 交互式 TUI**：无参数运行 `wally` 即可打开全功能命令菜单，无需记忆复杂的 CLI 参数。
*   **✅ 智能暂存 (Smart Staging)**：交互式的 `add` 命令，支持**Diff 差异预览**、文件切换和批量选择。
*   **📝 约定式提交**：内置向导，强制执行标准的提交信息格式（feat, fix, chore 等），包含 scope 和 body 的输入提示。
*   **🛡️ 安全同步**：
    *   **Update**：自动暂存 (Stash) 本地更改，使用变基 (Rebase) 拉取远程代码，最后自动恢复暂存，有效防止冲突。
    *   **Push**：检测远程分支差异（例如在 Squash 后），并提供安全的 `force-with-lease` 强制推送选项。
*   **🌿 分支管理**：交互式地创建、切换、删除分支，支持将特性分支的所有提交 **Squash（压缩）** 为一个暂存更改。
*   **🤖 AI 上下文生成**：扫描项目并生成 `context.md` 和 JSON 模板，专为 LLM/AI（如 ChatGPT/Claude）代码分析优化。
*   **🎨 项目模板**：使用 JSON 定义的自定义文件结构快速初始化新项目。
*   **📈 活动统计**：可视化最近的提交频率、代码行增删以及修改最频繁的文件。
*   **🏷️ 语义化标签**：自动计算版本号（Major/Minor/Patch）并创建 Git 标签。

## 📦 安装指南

### 前置要求
*   **Git**：确保系统中已安装 `git` 并在 PATH 中可用。
*   **Rust**：需要安装 Cargo 以进行源码编译。

### 源码编译安装

```bash
# 克隆仓库
git clone https://github.com/yourusername/wally.git
cd wally

# 本地安装
cargo install --path .
```

安装完成后，请确保您的 Cargo bin 目录（通常是 `~/.cargo/bin`）已包含在系统 PATH 中。

## 📖 使用指南

您可以通过两种方式使用 Wally：
1.  **交互模式**：直接运行 `wally` 打开主菜单。
2.  **直接模式**：运行 `wally <command>` 执行特定任务。

### 1. 核心工作流

*   **`wally add`**
    选择要暂存的文件。
    *   `SPACE` (空格)：切换选中状态。
    *   `a`：全选/取消全选。
    *   `→` / `←`：展开/收起差异 (Diff) 预览。
    *   `ENTER`：确认暂存。

*   **`wally commit`**
    构建符合约定式提交规范的信息。向导会提示您输入类型（feat, fix 等）、作用域 (scope)、简述 (subject) 和详情 (body)。

*   **`wally push`**
    将更改推送到远程仓库。
    *   如果没有设置上游分支，会自动处理。
    *   自动检测是否需要强制推送（例如在 Squash 之后），并使用安全的 `--force-with-lease` 模式。

*   **`wally update`**
    安全地与远程同步。执行流程：`git stash` -> `git pull --rebase` -> `git stash pop`。

### 2. 分支与历史

*   **`wally branch`**
    统一的分支管理菜单：
    *   **Switch**：交互式切换分支。
    *   **Create**：创建标准化的分支名（例如 `feat/login-page`）。
    *   **Squash**：基于主分支（main/master）将当前分支的所有提交压缩为一个暂存状态，便于生成干净的提交记录。
    *   **Delete**：删除本地及远程分支。

*   **`wally reset`**
    *   **Reflog**：通过引用日志撤销最近的操作（找回“丢失”的提交）。
    *   **Log**：硬重置 (Hard Reset) 到指定的历史提交。

### 3. 项目工具

*   **`wally new [name]`**
    使用内置或自定义模板初始化新项目。

*   **`wally context`**
    在 `info/` 目录下生成项目快照。
    *   `context.md`：包含文件树和代码块的 Markdown 文件（非常适合复制给 AI 进行分析）。
    *   `project_template.json`：项目结构的 JSON 表示。

*   **`wally stats`**
    查看生产力指标：过去 7 天的提交数、代码行变动统计以及最常修改的文件 TOP 5。

*   **`wally tag`**
    查看当前版本，并选择下一个语义化版本（Patch/Minor/Major）来创建并推送标签。

## 🛠️ 模板系统

Wally 支持通过 JSON 定义自定义项目模板。

### 模板结构
创建一个 `.json` 文件（例如 `react-starter.json`）：

```json
{
  "template_name": "react-starter",
  "description": "一个基础的 React 项目设置",
  "files": {
    "src/": "",
    "src/App.js": "console.log('Hello World');",
    ".gitignore": "node_modules/\ndist/",
    "README.md": "# My React Project"
  }
}
```

### 管理模板
*   **安装**：`wally install ./my-template.json`
*   **列表**：`wally list`
*   **卸载**：`wally uninstall my-template`

模板文件存储在 `~/.wally/templates/` 目录下。

## 🤝 贡献与参与

欢迎提交贡献！如果您有新功能的想法或改进建议：

1.  Fork 本仓库。
2.  创建您的特性分支 (`wally branch` -> Create)。
3.  提交您的更改 (`wally commit`)。
4.  推送到分支 (`wally push`)。
5.  提交 Pull Request。

## 📝 开源协议

本项目采用 MIT 许可证 - 详情请参阅 [LICENSE](LICENSE) 文件。

---

<p align="center">
  Built with 🦀 Rust by <a href="https://github.com/waliwuao">waliwuao</a>
</p>