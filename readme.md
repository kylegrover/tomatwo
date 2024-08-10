# T 🍅 M A T W 🍅

tomato.py, rewritten in rust

available as lib, cli, and gui

**Test**: `cargo run --release`\
**Build**: `cargo build`

### to do:
- fix 'play datamoshed video' button (crashes)
- fix 'bake output' (saves avi instead of mp4)

### basic usage

**gui:**\
1\. select a video (any type)\
2. choose mode, settings\
3. 'taste it' to preview before saving\
4. 'jar it' to save moshed avi\
5. 'bake output' to save as playable mp4

**cli:**\
`tomatwo -i food-test.mp4`

### loose ideas:
- https://dioxuslabs.com/ for frontend?
    run in browser with wasm? ffmpeg in browser?
- onnx https://github.com/microsoft/onnxruntime for inference?
    https://github.com/pykeio/ort
    "WASM only via emscripten (limitation of onnxruntime)"
    


very us lynx
https://github.com/ucnv/aviglitch
https://gitlab.freedesktop.org/gstreamer/gstreamer-rs
    https://gitlab.freedesktop.org/gstreamer/gstreamer-rs/-/blob/main/examples/src/bin/ges.rs
https://ottverse.com/i-p-b-frames-idr-keyframes-differences-usecases/
https://lib.rs/crates/egui-video
    https://github.com/n00kii/egui-video
https://github.com/dioxuslabs/dioxus
https://github.com/g-l-i-t-c-h-o-r-s-e/Datamosh-Den/blob/main-build/Datamosh%20Den.ahk