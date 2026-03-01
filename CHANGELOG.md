# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.4] - 2026-03-02

### Fixed

- **Side face normals**: 3D extruded glyphs now correctly show side faces when rendered with standard back-face culling. Previously the side face normals pointed inward instead of outward, causing them to be culled by renderers like Bevy.

## [0.3.3] - Previous

- Previous stable release
