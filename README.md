<p align="center"><img src="assets/icon.png" alt="MarkText" width="100" height="100"></p>

<h1 align="center">Spek-rs</h1>

Acoustic spectrum analyser. Spek alternative written in Rust. The program is used to create spectrograms of audio files. It uses FFmpeg for audio decoding, like the original.

<p align="center">
<img src=".github/assets/screenshot.png"/>
</p>

This application is a GUI for ffmpeg's showspectrumpic function, which allows you to generate spectrograms from audio files. For a detailed explanation of the available options and their functionalities, the best place to check is the official ffmpeg documentation: https://ffmpeg.org/ffmpeg-filters.html#showspectrumpic

I built this because the original Spek appears unmaintained, and installing it on rolling release distributions like Arch Linux often caused dependency issues with older libraries.

## Changelog

[CHANGELOG.md](CHANGELOG.md)

## Credits

This project is heavily inspired by the original [Spek](https://www.spek.cc/).

The GUI is built using [egui](https://github.com/emilk/egui).

It includes the [DejaVu Sans](https://dejavu-fonts.github.io/) font, which is distributed under its own license, see [LICENSE-DejaVuFonts.txt](./assets/LICENSE-DejaVuFonts.txt) for details.

The color palettes for the spectrogram are based on those found in the [FFmpeg](https://ffmpeg.org/) source code.
