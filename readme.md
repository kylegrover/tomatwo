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
    