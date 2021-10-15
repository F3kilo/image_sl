use image::DynamicImage;
use std::convert::{TryFrom, TryInto};
use std::ffi::{c_void, CStr};
use std::os::raw::c_char;
use std::path::Path;

#[repr(transparent)]
struct ImageHandle(*const c_void);

impl ImageHandle {
    /// # Panics
    /// Panics if `self.0` == null.
    pub unsafe fn as_image(&self) -> &'static mut DynamicImage {
        let ptr = self.0 as *mut DynamicImage;
        ptr.as_mut().unwrap() // Expect null checks before
    }

    /// # Safety
    /// `self.0` != null.
    pub unsafe fn into_image(self) -> Box<DynamicImage> {
        let ptr = self.0 as *mut DynamicImage;
        Box::from_raw(ptr)
    }
}

#[repr(transparent)]
struct RawPath(*const c_char);

#[repr(C)]
#[derive(Debug)]
enum ImageError {
    NoError = 0,
    Io,
    Decoding,
    Encoding,
    Parameter,
    Unsupported,
}

impl From<image::ImageError> for ImageError {
    fn from(e: image::ImageError) -> Self {
        match e {
            image::ImageError::Decoding(_) => Self::Decoding,
            image::ImageError::Encoding(_) => Self::Encoding,
            image::ImageError::Unsupported(_) => Self::Unsupported,
            image::ImageError::Parameter(_) => Self::Parameter,
            image::ImageError::IoError(_) => Self::Io,
            _ => Self::Unsupported,
        }
    }
}

type OpenImageFn = unsafe extern "C" fn(RawPath, *mut ImageHandle) -> ImageError;
type SaveImageFn = unsafe extern "C" fn(RawPath, ImageHandle) -> ImageError;
type DestroyImageFn = unsafe extern "C" fn(ImageHandle);

#[allow(unused)]
pub struct FunctionsBlock {
    size: usize,
    open_image: OpenImageFn,
    save_image: SaveImageFn,
    destroy_image: DestroyImageFn,
}

impl Default for FunctionsBlock {
    fn default() -> Self {
        Self {
            size: std::mem::size_of::<Self>(),
            open_image: img_open,
            save_image: img_save,
            destroy_image: img_destroy,
        }
    }
}

#[no_mangle]
pub extern "C" fn functions() -> *const FunctionsBlock {
    Box::leak(Box::new(FunctionsBlock::default()))
}

// Exported functions

/// # Safety
/// `path` is valid, null-terminated, UTF-8 string
/// `handle` is valid pointer to void*
unsafe extern "C" fn img_open(path: RawPath, handle: *mut ImageHandle) -> ImageError {
    if handle.is_null() || path.0.is_null() {
        return ImageError::Parameter;
    }

    let path: &Path = match (&path).try_into() {
        Ok(p) => p,
        Err(e) => return e,
    };

    let img = match image::open(path) {
        Ok(i) => i,
        Err(e) => return e.into(),
    };

    let leaked_handle = Box::leak(Box::new(img));
    let ptr = leaked_handle as *mut DynamicImage;
    *handle = ImageHandle(ptr as _);
    ImageError::NoError
}

/// # Safety
/// `path` is valid, null-terminated, UTF-8 string
/// `handle` is valid image handle obtained from `img_open()` function
unsafe extern "C" fn img_save(path: RawPath, handle: ImageHandle) -> ImageError {
    if handle.0.is_null() || path.0.is_null() {
        return ImageError::Parameter;
    }

    let path: &Path = match (&path).try_into() {
        Ok(p) => p,
        Err(e) => return e,
    };

    let img = handle.as_image();
    match img.save(path) {
        Ok(_) => ImageError::NoError,
        Err(e) => e.into(),
    }
}

unsafe extern "C" fn img_destroy(handle: ImageHandle) {
    handle.into_image();
}

// Utils

impl<'a> TryFrom<&'a RawPath> for &'a Path {
    type Error = ImageError;

    fn try_from(value: &'a RawPath) -> Result<Self, Self::Error> {
        if value.0.is_null() {
            return Err(ImageError::Parameter);
        }

        let s = unsafe { CStr::from_ptr(value.0) };
        let utf8_str = match s.to_str() {
            Ok(s) => s,
            Err(_) => return Err(ImageError::Parameter),
        };

        let path: &Path = Path::new(utf8_str);
        Ok(path)
    }
}
