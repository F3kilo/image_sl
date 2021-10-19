# image_sl
This repository shows how to create and use shared library in Rust.  
It exports some functions from [`image`](https://crates.io/crates/image) crate. `use_lib` example loads and uses them.

## Library interface
To use this library you need to import and call just one function `functions()`. 
Returned structure contains pointers to all useful functions of this library.

```rust
/// Contains functions provided by library. Allow to import just `functions()` function and get all 
/// functionality of library through this struct. `size` field contain size of this struct. 
/// It helps to avoid versioning and some other errors.
#[repr(C)]
pub struct FunctionsBlock {
    size: usize,
    open_image: OpenImageFn,
    save_image: SaveImageFn,
    destroy_image: DestroyImageFn,
    blur_image: BlurImageFn,
    mirror_image: MirrorImageFn,
}

/// Returns all functions of this library.
#[no_mangle]
pub extern "C" fn functions() -> FunctionsBlock {...}

/// Loads image from file function type.
type OpenImageFn = unsafe extern "C" fn(RawPath, *mut ImageHandle) -> ImageError;
/// Saves image to file function type.
type SaveImageFn = unsafe extern "C" fn(RawPath, ImageHandle) -> ImageError;
/// Destroys image function type.
type DestroyImageFn = unsafe extern "C" fn(ImageHandle);

/// Performs a Gaussian blur on the supplied image function type.
type BlurImageFn = unsafe extern "C" fn(ImageHandle, f32) -> ImageHandle;
/// Flips image horizontally function type.
type MirrorImageFn = unsafe extern "C" fn(ImageHandle);

/// Incapsulate raw pointer to image.
#[repr(transparent)]
struct ImageHandle(*mut c_void);

/// Contain pointer to null-terminated UTF-8 path.
#[repr(transparent)]
struct RawPath(*const c_char);

/// Error codes for image oprerations.
#[repr(C)]
enum ImageError {
    NoError = 0,
    Io,
    Decoding,
    Encoding,
    Parameter,
    Unsupported,
}
```
