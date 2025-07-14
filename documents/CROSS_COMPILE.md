# Cross-Compilation Guide - Building Windows .exe from Linux

## Quick Setup

### 1. Install Windows Target
```bash
rustup target add x86_64-pc-windows-gnu
```

### 2. Install MinGW Cross-Compiler
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install gcc-mingw-w64-x86-64

# Fedora/RHEL
sudo dnf install mingw64-gcc

# Arch Linux
sudo pacman -S mingw-w64-gcc
```

### 3. Build Windows Executable
```bash
cargo build --release --target x86_64-pc-windows-gnu
```

The Windows `.exe` will be created at:
```
target/x86_64-pc-windows-gnu/release/vampire-rpg.exe
```

## Alternative Method (if MinGW doesn't work)

### Use Windows MSVC Target
```bash
# Add MSVC target
rustup target add x86_64-pc-windows-msvc

# Build (requires additional setup)
cargo build --release --target x86_64-pc-windows-msvc
```

Note: MSVC target may require additional Windows SDK components.

## Testing the Windows Executable

### Option 1: Wine (Linux)
```bash
# Install Wine
sudo apt install wine

# Test the executable
wine target/x86_64-pc-windows-gnu/release/vampire-rpg.exe
```

### Option 2: Copy to Windows Machine
The `.exe` file is completely self-contained and can be copied to any Windows 10/11 machine and run directly without any dependencies.

## Build Script

Create this build script for easy cross-compilation:

```bash
#!/bin/bash
# build-windows.sh

echo "Building Windows executable from Linux..."

# Check if target is installed
rustup target list --installed | grep -q x86_64-pc-windows-gnu
if [ $? -ne 0 ]; then
    echo "Installing Windows target..."
    rustup target add x86_64-pc-windows-gnu
fi

# Build for Windows
echo "Cross-compiling to Windows..."
cargo build --release --target x86_64-pc-windows-gnu

if [ $? -eq 0 ]; then
    echo "✓ Success! Windows executable created:"
    echo "  target/x86_64-pc-windows-gnu/release/vampire-rpg.exe"
    ls -lh target/x86_64-pc-windows-gnu/release/vampire-rpg.exe
else
    echo "✗ Build failed"
    exit 1
fi
```

Make it executable:
```bash
chmod +x build-windows.sh
./build-windows.sh
```

## Troubleshooting

### "linker not found" error
Install the cross-compiler:
```bash
sudo apt install gcc-mingw-w64-x86-64
```

### Large executable size
The executable will be larger (~8-10MB) due to static linking, but this ensures it runs on any Windows machine without dependencies.

### OpenGL compatibility
Macroquad uses OpenGL which is available on all modern Windows systems, so the game will work on Windows 10/11 out of the box.

The cross-compiled Windows executable provides identical functionality to the Linux version with the same pixel art graphics, game mechanics, and performance.