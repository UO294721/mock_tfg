# Icons Directory

This directory should contain application icons for bundling.

## Required Icon Files

When you want to enable bundling (set `bundle.active: true` in `tauri.conf.json`), you need:

- `icon.ico` (Windows) - 256x256 or larger
- `icon.icns` (macOS) - 1024x1024 source
- `32x32.png` - 32x32 pixels
- `128x128.png` - 128x128 pixels
- `128x128@2x.png` - 256x256 pixels
- `icon.png` - 512x512 or larger (source image)

## Generating Icons

You can use the Tauri CLI to generate icons from a single source image:

```bash
npm install -g @tauri-apps/cli
tauri icon path/to/your/icon.png
```

This will generate all required icon sizes automatically.

## Current Status

**Bundling is currently disabled** in `tauri.conf.json` to allow development without icons.
To enable bundling for production builds, add icons here and set `bundle.active: true`.

## Quick Start

1. Create a 512x512 or 1024x1024 PNG icon
2. Run: `npm run tauri icon path/to/icon.png`
3. Update `tauri.conf.json` to set `bundle.active: true`
