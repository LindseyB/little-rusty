# Little Rusty

A simple Rust application using WGPU for GPU-accelerated graphics rendering.

## Description

## Dependencies

- **wgpu** - Modern graphics API abstraction layer
- **winit** - Cross-platform window creation and event handling
- **pollster** - Async runtime for blocking on futures
- **env_logger** - Logging implementation

## Building and Running

Make sure you have Rust installed, then:

```bash
# Clone the repository
git clone https://github.com/LindseyB/little-rusty.git
cd little-rusty

# Build and run
cargo run
```

## Known Issues

- If you resize the window to zero dimensions, the application may panic. Avoid resizing to zero.

---

Made with ❤️ by Lindsey B
