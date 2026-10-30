# rgrim

**A lightweight, cross-platform screenshot tool with built-in annotation.**

Capture your entire screen or select a specific region, annotate screenshots with a pen or highlighter, and copy or save the result.

Designed for Linux (X11 and Wayland), macOS, and Windows.

## Features

- **Flexible screen capture:** Capture the entire screen or select a rectangular region.
- **Built-in annotation:** Draw freehand or highlight important areas directly on your screenshots.
- **Custom colors:** Choose colors independently for the pen and highlighter.
- **Clipboard integration:** Copy your annotated screenshots directly to the system clipboard.
- **Automatic file saving:** Save screenshots to your preferred directory.
- **Delayed capture:** Schedule screenshots with a 3- or 5-second countdown.
- **Desktop notifications:** Get notified when a screenshot is successfully saved.
- **Cross-platform:** Works on Linux, macOS, and Windows.

---

## Dependencies

- **capture**: `xcap`, `image`, `anyhow`
- **ui / editor**: `eframe` (egui 0.37), `image`
- **export**: `chrono`, `dirs`
- **clipboard**: `arboard`, `wl-copy` (optional Wayland fallback)

---

## Installation & Dependencies

Ensure your compiler target satisfies the underlying platform-specific requirements.

### Linux Prerequisites
When running on Linux (X11/Wayland), ensure the development packages for `xcb`, `libxkbcommon`.

```bash
# On Debian/Ubuntu based environments
sudo apt update && sudo apt install -y \
    libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libxkbcommon-dev libdbus-1-dev wl-clipboard

```

### Build from Source

Clone the project space and build using the cargo package suite:

```bash
git clone [https://github.com/riccione/rgrim.git](https://github.com/riccione/rgrim.git)
cd rgrim
cargo build --release

```

---

## Command-Line Flags

| Flag | Description |
| --- | --- |
| `--gui` / `-g` | Open the dashboard window instead of starting a capture immediately |
| `--delay <SECONDS>` / `-d <SECONDS>` | Wait before the first capture, e.g. `rgrim --delay 3` or `rgrim --delay=5`. A per-second countdown is printed while waiting. Only the first capture is delayed this way; later captures use the dashboard or editor delay buttons |
| `--help` / `-h` | Print usage |

The dashboard's capture buttons choose the delay per click: **Take Screenshot Now**, **Capture in 3 s**, and **Capture in 5 s** (`--delay` applies to non-GUI runs). The editor toolbar mirrors this with **In 3 s** and **In 5 s** re-capture buttons.

---

## Global Keybindings

### 1. Sniper Mode Keybindings

When launching `rgrim`, the selection layer takes focus. Use these commands to manipulate your selection:

| Shortcut | Action |
| --- | --- |
| `Left-Click & Drag` | Draw custom rectangular screenshot boundaries |
| `Enter` | Instantly capture the entire display canvas |
| `Escape` / `Q` | Abort screen-grab operation and exit silently |

### 2. Editor Canvas Mode Keybindings

Once selection captures finish, the sketch engine launches with a dedicated toolbar footprint:

| Active Interaction | Mode / Tool Purpose |
| --- | --- |
| **New Button** | Discards the editor session and immediately re-enters sniper capture mode |
| **In 3 s / In 5 s Buttons** | Discards the editor session and starts the next capture after the chosen delay, printing a countdown while waiting |
| **Pen Button** | Sharp freehand illustration stroke ($3\text{px}$ profile) |
| **Highlighter Button** | Semi-transparent highlight channel ($24\text{px}$ profile) |
| **Color Swatch** | Click the swatch beside Pen or Highlighter to pick that tool's color; the highlighter keeps its fixed translucency and choices persist across New captures |
| **Clear Button** | Completely flushes all vector stroke markers from active viewport |
| **Copy Button** | Encapsulates visual matrix directly to system clipboards |
| **Save Button** | Writes the customized composite out to targeted disk files |
| `Escape` | Terminate session immediately without writing any files |

---

## Screenshot Target Directory Priority

When saving captured visual content, the file engine checks configuration priorities in the following sequence:

1. **`RGRIM_DIR` Environment Hook:** Explicit customized configuration override path (e.g. `RGRIM_DIR=/mnt/nas/screencaps`).
2. **XDG User Directories Standard:** Defaults to your environment's native storage array at `~/Pictures/Screenshots` or `$XDG_PICTURES_DIR/Screenshots`.
3. **Local Working Fallback:** Fallback placement inside `./screenshots` within the binary execution root directory.

*Filenames are constructed deterministically using the ISO layout archetype: `screenshot_YYYY-MM-DD_HH-MM-SS.png`.*

---

## System Global Hotkey Configuration

Because `rgrim` operates as an on-demand native overlay, you must map the binary execution hook to your operating system's global shortcut daemon. This allows the tool to activate instantly in the background.

### Linux (Sway, i3wm, GNOME, KDE)

Linux handles custom global hotkeys natively via your desktop environment or window manager configuration files.

#### Sway / i3wm
Append these rules inside `~/.config/sway/config` (or `~/.config/i3/config`):
```sway
bindsym Mod4+Shift+g exec /relative/path/to/rgrim
```

#### GNOME

1. Go to **Settings** -> **Keyboard** -> **Keyboard Shortcuts** -> **View and Customize Shortcuts**.
2. Scroll to the bottom, select **Custom Shortcuts**, and click the **+** icon.
3. Set **Name** to `rgrim Sniper`, **Command** to `/absolute/path/to/rgrim`, and map the shortcut to `Super + Shift + G`.

#### KDE Plasma

1. Go to **System Settings** -> **Shortcuts** -> **Custom Shortcuts**.
2. Select **Edit** -> **New** -> **Global Shortcut** -> **Command/URL**.
3. Map your trigger key matrix (`Meta + Shift + G`) and point the target path directly to your binary.

### Windows (10 / 11)

Windows does not feature an elite built-in CLI-to-keyboard mapper, so choose one of these native/official solutions:

#### Method A: Microsoft PowerToys (Recommended & Cleanest)

1. Download **Microsoft PowerToys** from the Microsoft Store or GitHub.
2. Navigate to **Keyboard Manager** -> **Remap a shortcut** -> **Add shortcut remapping**.
3. Map the physical shortcut `Win + Shift + G`.
4. Set the **Action** dropdown to **Run Program** and paste the absolute file track path targeting your compiled `rgrim.exe`.

#### Method B: Standard Desktop Shortcut Property (No extra software)

1. Right-click your compiled `target\release\rgrim.exe` -> **Show more options** -> **Create shortcut** (move it to your Desktop).
2. Right-click the newly generated desktop shortcut -> **Properties**.
3. Click the **Shortcut key** box and input your combination (Note: Windows native desktop shortcut properties require `Ctrl + Alt + G` instead).

### macOS

To avoid heavy third-party automation tools, you can leverage Apple's native environment features:

#### Using the native Shortcuts App

1. Launch the macOS **Shortcuts** application.
2. Select **+** to build a brand new workflow pipeline layout.
3. Drag the **Run Shell Script** action block into the layout pane and input `/absolute/path/to/rgrim`.
4. Inside the right-hand options configuration drawer, toggle **Use as Quick Action**.
5. Check **Provide Keyboard Shortcut** and bind it directly to `Cmd + Shift + G` (or `Opt + Shift + G`).
6. *Note: Ensure you grant the executable **Accessibility** and **Screen Recording** permissions under macOS System Settings upon your first launch.*

---

## Contributing

Contributions are welcome! Whether you're fixing a bug, improving documentation, or proposing a new feature, feel free to get involved.

See [Contributing Guide](docs/CONTRIBUTING.md) for development workflow, branch naming, commit conventions, and pull request guidelines.

---

## License

This project is licensed under the terms of the [**Apache License 2.0**](LICENSE).
