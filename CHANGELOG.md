# Changelog

All notable changes to `rgrim` will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-09-26

### Bug Fixes

- *(ui)* Resolve float-literal f32 fallback lint
- *(ui)* Propagate eframe errors from sniper overlay
- *(capture)* Wait for a compositor-stable frame after closing the editor
- *(editor)* Use one (premultiplied) alpha convention for colors
- *(ui)* Write sniper toolbar fallback offset as a plain addition
- *(editor)* Render click-only strokes as dots in preview and export

### Documentation

- List the New button in the editor toolbar table
- Cover remaining public API and modules with doc comments
- Add pull request template

### Features

- *(editor)* Add New button to return to capture flow
- *(ui)* Increase default font size by 20%

### Miscellaneous Tasks

- Add linter checking to workflow (#29)
- Fix cargo-release signing settings (#31)
- Run clippy after installing linux dependencies (#32)
- Upgrade actions/checkout to v7 (#35)
- Update ci.yml to use shared reusable Rust ci (#36)
- Add git-cliff changelog configuration
- Update dependencies
- Pin Rust toolchain to stable via rust-toolchain.toml
- Pin reusable workflows to immutable commit SHAs
- Add dependabot for cargo and github-actions

### Other

- Add Makefile with test and check targets
- Stop auto-pushing cargo-release commits and tags

### Performance

- *(ui)* Pass sniper overlay background by reference
- *(editor)* Move image into EditorApp instead of cloning
- *(export)* Borrow pixel bytes for clipboard ImageData
- *(ui)* Stop perpetual repaint of the idle sniper overlay
- *(export)* Return the cached screenshot directory as &'static Path

### Refactoring

- Separate pixel-level drawing into dedicated draw module (#28)
- *(ci)* Split unified validation into separate parallel jobs (#34)
- *(ci)* Migrate release to reusable workflow (#37)
- *(ui)* Migrate to eframe/egui 0.36 app and panel APIs
- *(editor)* Replace Tool::None with Option<DrawTool>
- *(editor)* Move image directly into the once-called app creator
- *(editor)* Encode sticky status messages with Option<f64>
- *(main)* Extract try_auto_save from the capture loop
- Keep error source chains via anyhow::Context
## [0.2.0] - 2026-05-25

### Bug Fixes

- Add Q key to close editor, matching sniper overlay (#23)

### Documentation

- Add cross-platform global hotkey guide to README (#25)

### Features

- Add CLI modes and dashboard panel (#15)

### Performance

- Cache screenshot directory resolution with OnceLock (#22)

### Refactoring

- Remove unnecessary clone of Option<String> in EditorApp::new (#16)
- *(crop)* Eliminate full-image clone with safe coordinate clamping (#17)
- *(tool)* Move drawing properties into Tool::drawing_properties() (#18)
- *(ui)* Use consistent mutex error handling in sniper overlay (#19)
- Remove unused SelectionRect struct (#20)
## [0.1.0] - 2026-05-25

### Documentation

- Update README.md (#12)

### Features

- Add hardware-accelerated primary monitor screen capture
- *(ui)* Implement full-screen sniper overlay for region selection (#2)
- *(editor)* Implement image editor window with drawing tools (#3)
- *(ui)* Fallback to full viewport on minimal area selection and add Enter key shortcut (#4)
- *(export)* Introduce screenshot auto-save and customizable paths (#6)
- Scale selected region to physical pixels and improve viewport settings (#9)
- *(ci)* Add GitHub Actions release workflow and binary size optimizations (#13)

### Miscellaneous Tasks

- Cargo init
- Add dependencies

### Refactoring

- *(editor)* Eliminate double allocation and panic path in EditorApp::new (#11)
