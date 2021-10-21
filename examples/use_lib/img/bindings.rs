use std::error::Error;
use std::ffi::c_void;
use std::fmt::{Display, Formatter};
use std::os::raw::c_char;
use std::path::Path;

/// Statically known path to library.
#[cfg(target_os = "linux")]
pub fn lib_path() -> &'static Path {
    Path::new("target/release/libimage_sl.so")
}

/// Statically known path to library.
#[cfg(target_os = "windows")]
pub fn lib_path() -> &'static Path {
    Path::new("target/release/image_sl.dll")
}

/// Incapsulate raw pointer to image.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct ImageHandle(*const c_void);

impl ImageHandle {
    /// Creates new null pointer.
    pub unsafe fn new_null() -> Self {
        Self(std::ptr::null())
    }
}

/// Contain pointer to null-terminated UTF-8 path.
#[repr(transparent)]
pub struct RawPath(pub *const c_char);

/// Error codes for image oprerations.
#[repr(u32)]
#[derive(Debug)]
#[allow(dead_code)]
pub enum ImageError {
    NoError = 0,
    Io,
    Decoding,
    Encoding,
    Parameter,
    Unsupported,
}

impl Display for ImageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageError::NoError => write!(f, "image no error"),
            ImageError::Io => write!(f, "image i/o error"),
            ImageError::Decoding => write!(f, "image decoding error"),
            ImageError::Encoding => write!(f, "image encoding error"),
            ImageError::Parameter => write!(f, "image parameter error"),
            ImageError::Unsupported => write!(f, "image unsupported error"),
        }
    }
}

/// Required for converting `ImageError` to `anyhow::Error`.
impl Error for ImageError {}

/// Load functions block
pub type FunctionsFn = unsafe extern "C" fn() -> Functions;

/// Loads image from file
pub type OpenImageFn = unsafe extern "C" fn(RawPath, *mut ImageHandle) -> ImageError;
/// Saves image to file
pub type SaveImageFn = unsafe extern "C" fn(RawPath, ImageHandle) -> ImageError;
/// Destroys image
pub type DestroyImageFn = unsafe extern "C" fn(ImageHandle);

/// Performs a Gaussian blur on the supplied image.
pub type BlurImageFn = unsafe extern "C" fn(ImageHandle, f32) -> ImageHandle;
/// Flips image horizontally
pub type MirrorImageFn = unsafe extern "C" fn(ImageHandle);

/// Contains functions provided by library. Allow to import just `functions()` function and get all
/// functionality of library through this struct.
/// `size` field contain size of this struct. It helps to avoid versioning and some other errors.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Functions {
    pub size: usize,
    pub open_image: OpenImageFn,
    pub save_image: SaveImageFn,
    pub destroy_image: DestroyImageFn,
    pub blur_image: BlurImageFn,
    pub mirror_image: MirrorImageFn,
}
