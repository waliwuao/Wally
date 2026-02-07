# Wally

**A friendly, interactive Git CLI companion written in Rust.**

Wally is designed to streamline your Git workflow, enforce best practices (like Conventional Commits), and bridge the gap between complex Git commands and developer productivity. Whether you are a Git beginner or a seasoned developer looking to reduce keystrokes, Wally has you covered.

It acts as a transparent wrapper around Git, providing interactive TUI menus while always showing you the underlying commands being executed.

## ✨ Key Features

*   **🚀 Interactive TUI**: Run `wally` without arguments to access a full command menu. No need to memorize CLI flags.
*   **✅ Smart Staging**: Interactive `add` command with **diff previews**, file toggling, and bulk selection.
*   **Conventional Commits**: Built-in wizard to enforce standard commit messages (feat, fix, chore, etc.) with scopes and bodies.
*   **🛡️ Safe Syncing**:
    *   **Update**: Automatically stashes local changes, pulls with rebase, and pops the stash to prevent conflicts.
    *   **Push**: Detects remote divergence (e.g., after squashing) and offers safe `force-with-lease` pushing.
*   **🌿 Branch Management**: Create, switch, delete, and **squash** branches interactively.
*   **🤖 AI Context Generation**: Scans your project and generates `context.md` and `json` templates optimized for LLM/AI analysis.
*   **🎨 Project Templates**: Initialize new projects with custom file structures defined in JSON.
*   **📈 Activity Stats**: Visualize recent commit frequency, line changes, and most modified files.
*   **🏷️ Semantic Tagging**: Automates version bumping (Major/Minor/Patch) and tagging.

## 📦 Installation

### Prerequisites
*   **Git**: Ensure `git` is installed and available in your PATH.
*   **Rust**: You need Cargo to build from source.

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/wally.git
cd wally

# Install locally
cargo install --path .
```

Ensure your Cargo bin directory (usually `~/.cargo/bin`) is in your system PATH.

## 📖 Usage Guide

You can use Wally in two ways:
1.  **Interactive Mode**: Simply run `wally` to open the main menu.
2.  **Direct Mode**: Run `wally <command>` to execute a specific task.

### 1. Workflow Essentials

*   **`wally add`**
    Select files to stage.
    *   `SPACE`: Toggle selection.
    *   `a`: Toggle select all.
    *   `→` / `←`: Expand/Collapse diff preview.
    *   `ENTER`: Confirm staging.

*   **`wally commit`**
    Construct a commit message following Conventional Commits. You will be prompted for the type (feat, fix, etc.), scope, subject, and body.

*   **`wally push`**
    Push changes to the remote.
    *   Automatically handles upstream setting if missing.
    *   Detects if a force push is needed (e.g., after squashing) and uses `--force-with-lease` for safety.

*   **`wally update`**
    Safely sync with the remote. It performs: `git stash` -> `git pull --rebase` -> `git stash pop`.

### 2. Branching & History

*   **`wally branch`**
    A unified menu to:
    *   **Switch**: Interactive branch selection.
    *   **Create**: Generate standardized branch names (e.g., `feat/login-page`).
    *   **Squash**: Squash all commits in your feature branch into a single staged change relative to a base branch (main/master).
    *   **Delete**: Remove local and remote branches.

*   **`wally reset`**
    *   **Reflog**: Undo recent actions (even "lost" commits).
    *   **Log**: Hard reset to a specific commit history.

### 3. Project Utilities

*   **`wally new [name]`**
    Initialize a new project using a built-in or custom template.

*   **`wally context`**
    Generates a snapshot of your project in `info/`.
    *   `context.md`: Markdown file tree and code blocks (great for pasting into ChatGPT/Claude).
    *   `project_template.json`: A JSON representation of your project structure.

*   **`wally stats`**
    View productivity metrics: commit counts (last 7 days), line additions/deletions, and top modified files.

*   **`wally tag`**
    View the current version and select the next semantic version (Patch/Minor/Major) to create and push a tag.

## 🛠️ Template System

Wally supports custom project templates defined in JSON.

### Template Structure
Create a `.json` file (e.g., `react-starter.json`):

```json
{
  "template_name": "react-starter",
  "description": "A basic React setup",
  "files": {
    "src/": "",
    "src/App.js": "console.log('Hello World');",
    ".gitignore": "node_modules/\ndist/",
    "README.md": "# My React Project"
  }
}
```

### Managing Templates
*   **Install**: `wally install ./my-template.json`
*   **List**: `wally list`
*   **Uninstall**: `wally uninstall my-template`

Templates are stored in `~/.wally/templates/`.

## 🤝 Contributing

Contributions are welcome! If you have ideas for new features or improvements:

1.  Fork the repository.
2.  Create your feature branch (`wally branch` -> Create).
3.  Commit your changes (`wally commit`).
4.  Push to the branch (`wally push`).
5.  Open a Pull Request.

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

<p align="center">
  Built with 🦀 Rust by <a href="https://github.com/waliwuao">waliwuao</a>
</p>