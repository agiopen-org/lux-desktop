# Lux Desktop

## Introduction
Lux Desktop is an example client of [oagi-python](https://github.com/agiopen-org/oagi-python/tree/main) developed with [Tauri](https://tauri.app/).

## Prerequisites
To develop or build Lux Desktop locally, [node.js](https://nodejs.org/), [pnpm](https://pnpm.io/) and [Rust](https://rust-lang.org/) toolchain will be needed. You can install them with:
```bash
# install node.js
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh | bash
\. "$HOME/.nvm/nvm.sh"
nvm install 24

# install pnpm
npm install -g pnpm

# install rust toolchains
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Get Started
Clone this repository:
```bash
git clone https://github.com/agiopen-org/lux-desktop.git
```

Install dependencies with pnpm:
```bash
cd lux-desktop
pnpm install
```

Launch with dev mode and hot reload:
```bash
pnpm tauri dev
```

Build executable binary:
```bash
pnpm tauri build
```

Sign the application (optional but recommended for distribution):
```bash
# macOS
codesign --sign "Your certificate name" src-tauri/target/release/bundle/macos/lux-desktop.app

# Windows
# Sign the MSI installer with your code signing certificate
signtool sign /f "path\to\certificate.pfx" /p "password" /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 src-tauri\target\release\bundle\msi\lux-desktop_*_x64_en-US.msi
```

The built application can be found at:
- **macOS**: `src-tauri/target/release/bundle/macos/lux-desktop.app`
- **Windows**: `src-tauri/target/release/bundle/msi/`
