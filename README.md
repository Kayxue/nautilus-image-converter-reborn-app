# Nautilus Image Converter Reborn (App)

Application part of Nautilus Image Converter Reborn, which can help you convert, resize, and rotate images.

> [!NOTE]
> Still under heavy development. Stay tuned.

## Repository

Extensions repository: Uninitialized
App repository: This repository

## Architecture

![Architecture](https://github.com/Kayxue/nautilus-image-converter-reborn-app/blob/master/archtecture.png)

## Technology Stack

- Rust
- gtk-rs
- Relm4
- fast_image_resize
- image
- imageproc

## TODO
- [X] Arguments receiving logic.
- [X] Read image
- [X] Save image
- [X] Basic Image resizing using fast_image_resize
- [ ] Basic image rotation using image
- [ ] Top window detection
- [ ] GUI
- [ ] Better error handling
- [ ] Image conversion
- [ ] (if possible) Rewrite extension part using Rust.
- [ ] (if possible) Merge nautilus-extension part into this repo.
