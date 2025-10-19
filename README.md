# spek-rs
Spek alternative written in Rust. The program is used to create spectrograms of audio files. It uses FFmpeg for audio decoding (like the original) and [egui](https://github.com/emilk/egui) for the GUI.

<p align="center">
<img src=".github/assets/screenshot.png"/>
</p>

The main difference from the original is that the legend around the spectrogram is also created by FFmpeg, which makes it look a bit worse. So, basically, this program is a simple GUI for FFmpeg's spectrogram generation feature. I built this since the original Spek appears to be unmaintained, and installing it from the AUR was often problematic.

## TODO
- [x] Save and load settings from a configuration file
- [x] Draw captions at the top of the image like original
- [x] Custom legend rendering
- [ ] Add keyboard shortcuts

## Credits

This project is heavily inspired by the original [Spek](https://www.spek.cc/).

It includes the [DejaVu Sans](https://dejavu-fonts.github.io/) font, which is distributed under its own license, see [LICENSE-DejaVuFonts.txt](./assets/LICENSE-DejaVuFonts.txt) for details.

The color palettes for the spectrogram are based on those found in the [FFmpeg](https://ffmpeg.org/) source code.
