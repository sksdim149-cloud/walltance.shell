# Walltance Shell 🐚

A lightweight, techno-minimalist interactive shell written in **Rust**. Designed for power users who prefer tiling window managers (like DriftWM/Hyprland) and want a highly customizable, module-based configuration.

## ✨ Features

-   **Module-Based Config:** Organized `.walltancerc` with dedicated blocks for `aliases`, `functions`, and `bash` scripts.
-   **Rust-Powered:** Built for speed and safety.
-   **Smart Aliases:** Quick command shortcuts without the overhead of heavy shells.
-   **Custom Functions:** Support for multi-line command execution.
-   **Integrated Bash Bridge:** Run native bash scripts directly during initialization.
-   **Environment Expansion:** Full support for system environment variables (e.g., `$USER`, `$HOME`).
-   **Techno-Minimalist UI:** Clean prompt with Tokyo Night/True Black aesthetic support.

## 🛠 Installation

### Prerequisites

-   **Rust & Cargo:** Make sure you have the latest stable toolchain installed.
-   **Arch Linux (Recommended):** Optimized for CachyOS/Arch environments.

### Build from Source

```bash
# Clone the repository
git clone [https://github.com/yourusername/walltance.git](https://github.com/yourusername/walltance.git)
cd walltance

# Build and install locally
cargo install --path .

To use Walltance as your default shell:
Bash

echo "/usr/local/bin/walltance" | sudo tee -a /etc/shells
chsh -s /usr/local/bin/walltance

⚙️ Configuration

Walltance uses a unique modular configuration file located at ~/.walltancerc.
Example .walltancerc
Bash

# Define your shortcuts
aliases {
    ls = eza --icons --group-directories-first
    ll = eza -lh
    n  = nvim
    y  = yazi
}

# Run native bash commands on startup
bash {
    fastfetch -c neofetch --logo ~/logo.txt
}

# Define custom shell functions
functions {
    cls {
        clear
        ls
    }
}

🚀 Usage

Simply type your commands as you would in any other shell. Walltance handles:

    Piping: ls | grep rust

    Logical AND: cargo build && ./target/release/app

    Directory Shortcuts: Auto-CD into paths if the input is a valid directory.

🏗 Project Structure

    src/main.rs: Core engine, parser, and execution logic.

    ShellContext: Handles state, history, and configuration mapping.

🤝 Contributing

This is a personal project by a Linux enthusiast. Feel free to open issues or submit PRs if you want to add new modules (like env { } or plugins { }).

Developed with ❤️ in Rust by thespilindell.
