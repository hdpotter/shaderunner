# Changelog

## [0.1.4] - 2024-02-08

### Added
- Methods to inspect vertices and indices on `Mesh`.

### Fixed
- No longer crashes when trying to render meshes with no indices.

## [0.1.3] - 2024-02-06

### Changed
- Update wgpu to 23.0.1.
- Update winit to 0.19.4.
- Update egui and its wgpu and winit integrations to 0.30.0.
- Move `UIManager` functionality into `Renderer::run_ui()`.
- Modify `Game` trait to reflect new winit event callbacks.

### Added
- Immediate mode line drawing on `Renderer`.

### Fixed
- egui widgets now respond to events.

## [0.1.2] - 2024-10-25

### Changed

- Modify `GameProgram` struct to pass all types of events to `input` instead of just `WindowEvent`s, but to intercept and filter out events that are useful for resizing etc.
- Modify `Game` trait `resize` and `input` functions to take a reference instead of an owned value.
- Rename `RenderGame` to `ExampleGame` in examples.

## [0.1.1] - 2024-10-09

### Added

- `GameProgram` struct to allow program creation using the `Game` trait; this functionality was formerly `EchoesProgram` in examples.
- Reexport most functionality at top level for convenience.

## [0.1.0] - 2024-09-30

### Added

- Create crate with basic rendering functionality.