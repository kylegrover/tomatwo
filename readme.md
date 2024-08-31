# T 🍅 M A T W 🍅

⚠ **tomatwo is not done yet but you are welcome to fuck around and find out** ⚠

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


### release:
- `scripts/build_release.sh`

### benchmark:
- `python scripts/benchy.py`

in my tests the rust version is ~5x faster for small files and ~50x faster for medium files

```
Comparative Timings (seconds):
Average
Rust:    ❙❙❙❙❙❙❙                                  0.030302
Python:  ❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙   0.146390
Min
Rust:    ❙❙❙❙❙❙                                   0.023191
Python:  ❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙    0.142105
Max
Rust:    ❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙                      0.073001
Python:  ❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙ 0.152009
Comparison:
  Rust is 4.83x faster than Python on average
```

```
Average
Rust:                                             0.225427
Python:  ❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙   10.919359
Min
Rust:                                             0.220706
Python:  ❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙   10.769082
Max
Rust:                                             0.233042
Python:  ❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙❙ 11.247291
Comparison:
  Rust is 48.44x faster than Python on average
```

### loose ideas:
- https://dioxuslabs.com/ for frontend?
    run in browser with wasm? ffmpeg in browser?
- onnx https://github.com/microsoft/onnxruntime for inference?
    https://github.com/pykeio/ort
    "WASM only via emscripten (limitation of onnxruntime)"
- use video-rs for mp4 to avi and avi to mp4 if possible
  - https://github.com/oddity-ai/video-rs
  - not sure how it handles broken files
- use egui-video for display instead of ffplay
  - https://github.com/n00kii/egui-video
- compile for web with wasm
- compile python version with rustpython or py03
  


very us lynx
- https://github.com/ucnv/aviglitch
- https://gitlab.freedesktop.org/gstreamer/gstreamer-rs
    - https://gitlab.freedesktop.org/gstreamer/gstreamer-rs/-/blob/main/examples/src/bin/ges.rs
- https://ottverse.com/i-p-b-frames-idr-keyframes-differences-usecases/
- https://lib.rs/crates/egui-video
    - https://github.com/n00kii/egui-video
- https://github.com/dioxuslabs/dioxus
- https://github.com/g-l-i-t-c-h-o-r-s-e/Datamosh-Den/blob/main-build/Datamosh%20Den.ahk
