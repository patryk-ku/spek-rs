# spek-rs
Spek alternative written in Rust. The program is used to create spectrograms of audio files. Like the original, it uses FFmpeg for audio decoding and [egui](https://github.com/emilk/egui) for the GUI.

The main difference from the original is that the legend around the spectrogram is also created by FFmpeg, which makes it look a bit worse. So, basically, this program is a simple GUI for FFmpeg's spectrogram generation feature.I built this since the original Spek appears to be unmaintained, and installing it from the AUR was often problematic.

## TODO
- [ ] Add keyboard shortcuts
- [ ] Draw text displaying the file path and spectrogram settings at the top of the image
- [ ] Custom legend rendering
