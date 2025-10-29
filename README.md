<p align="center"><img src="assets/icon.png" alt="MarkText" width="100" height="100"></p>

<h1 align="center">Spek-rs</h1>

Acoustic spectrum analyser. Spek alternative written in Rust. The program is used to create spectrograms of audio files. It uses FFmpeg for audio decoding, like the original.

<p align="center">
<img src=".github/assets/screenshot.png"/>
</p>

This application is a GUI for ffmpeg's showspectrumpic function, which allows you to generate spectrograms from audio files. For a detailed explanation of the available options and their functionalities, the best place to check is the official ffmpeg documentation: https://ffmpeg.org/ffmpeg-filters.html#showspectrumpic

I built this because the original Spek appears unmaintained, and installing it on rolling release distributions like Arch Linux often caused dependency issues with older libraries.

## Download and Install 

<table>
    <!-- <tr>
        <th>OS</th>
        <th>Download</th>
    </tr> -->
    <tr>
        <td>Windows</td>
        <td>
            <a href="https://github.com/patryk-ku/spek-rs/releases/latest/download/spek-rs.msi">
                <img alt="Windows Installer" src="https://img.shields.io/badge/Installer-blue?style=for-the-badge&label=x64&labelColor=grey">
            </a>
            <br/>
            <a href="https://github.com/patryk-ku/spek-rs/releases/latest/download/spek-rs-portable.exe">
                <img alt="Windows Portable" src="https://img.shields.io/badge/Portable-slateblue?style=for-the-badge&label=x64&labelColor=grey">
            </a>
        </td>
    </tr>
    <tr>
        <td>Linux</td>
        <td>
            <a href="https://github.com/patryk-ku/spek-rs/releases/latest/download/spek-rs.deb">
                <img alt="Linux rpm" src="https://img.shields.io/badge/.deb-tomato?style=for-the-badge&label=x64&labelColor=grey">
            </a>
            <br/>
            <a href="https://github.com/patryk-ku/spek-rs/releases/latest/download/spek-rs.rpm">
                <img alt="Linux rpm" src="https://img.shields.io/badge/.rpm-firebrick?style=for-the-badge&label=x64&labelColor=grey">
            </a>
            <br/>
            <a href="https://github.com/patryk-ku/spek-rs/releases/latest/download/spek-rs.AppImage">
                <img alt="Linux AppImage" src="https://img.shields.io/badge/.AppImage-5a91b8?style=for-the-badge&label=x64&labelColor=grey">
            </a>
            <br/>
            <a href="https://aur.archlinux.org/packages/spek-rs-bin">
                <img alt="Linux AUR" src="https://img.shields.io/badge/AUR-0090DF?style=for-the-badge&label=x64&labelColor=grey">
            </a>
            <br/>
            <a href="https://github.com/patryk-ku/spek-rs/releases/latest/download/spek-rs">
                <img alt="Linux bin" src="https://img.shields.io/badge/Bin-lightgrey?style=for-the-badge&label=x64&labelColor=grey">
            </a>
        </td>
    </tr>
    <tr>
        <td>MacOS</td>
        <td>
            <a href="https://github.com/patryk-ku/spek-rs/releases/latest/download/spek-rs-macos-arm64.zip">
                <img alt="MacOS app" src="https://img.shields.io/badge/.App-hotpink?style=for-the-badge&label=arm64&labelColor=grey">
            </a>
            <br/>
            <a href="https://github.com/patryk-ku/spek-rs/releases/latest/download/spek-rs-macos-amd64.zip">
                <img alt="MacOS app" src="https://img.shields.io/badge/.App-hotpink?style=for-the-badge&label=x64&labelColor=grey">
            </a> 
        </td>
    </tr>
</table>

The Windows installer is experimental and may not work perfectly on all systems.

MacOS builds are available, but I can't test them since I don't own any Apple devices. Use at your own risk.

All available downloads can be found on the [Releases](https://github.com/patryk-ku/spek-rs/releases) page.

## Compile from source

1. Install Rust and Cargo using instructions from [Rust site](https://www.rust-lang.org/).
2. Clone the repository.
   ```sh
   git clone 'https://github.com/patryk-ku/spek-rs'
   cd spek-rs
   ```
3. Compile executable using Cargo.
   ```sh
   cargo build --release
   ```
4. The compiled executable file location is: `target/release/spek-rs`.

## Changelog

[CHANGELOG.md](CHANGELOG.md)

## Credits

This project is heavily inspired by the original [Spek](https://www.spek.cc/).

The GUI is built using [egui](https://github.com/emilk/egui).

It includes the [DejaVu Sans](https://dejavu-fonts.github.io/) font, which is distributed under its own license, see [LICENSE-DejaVuFonts.txt](./assets/LICENSE-DejaVuFonts.txt) for details.

The color palettes for the spectrogram are based on those found in the [FFmpeg](https://ffmpeg.org/) source code.
