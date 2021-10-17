use image::DynamicImage;
use std::convert::{TryFrom, TryInto};
use std::ffi::{c_void, CStr};
use std::os::raw::c_char;
use std::path::Path;

#[repr(transparent)]
struct ImageHandle(*mut c_void);

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

    /// # Safety
    /// `self.0` != null.
    pub fn from_image(image: DynamicImage) -> Self {
        let reference = Box::leak(Box::new(image));
        let ptr = reference as *mut DynamicImage;
        Self(ptr as _)
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

/// Loads image from file
type OpenImageFn = unsafe extern "C" fn(RawPath, *mut ImageHandle) -> ImageError;
/// Saves image to file
type SaveImageFn = unsafe extern "C" fn(RawPath, ImageHandle) -> ImageError;
/// Destroys image
type DestroyImageFn = unsafe extern "C" fn(ImageHandle);

/// Performs a Gaussian blur on the supplied image.
type BlurImageFn = unsafe extern "C" fn(ImageHandle, f32) -> ImageHandle;
/// Flips image horizontally
type MirrorImageFn = unsafe extern "C" fn(ImageHandle);

#[allow(unused)]
#[repr(C)]
pub struct FunctionsBlock {
    size: usize,
    open_image: OpenImageFn,
    save_image: SaveImageFn,
    destroy_image: DestroyImageFn,
    blur_image: BlurImageFn,
    mirror_image: MirrorImageFn,
}

impl Default for FunctionsBlock {
    fn default() -> Self {
        Self {
            size: std::mem::size_of::<Self>(),
            open_image: img_open,
            save_image: img_save,
            destroy_image: img_destroy,
            blur_image: img_blur,
            mirror_image: img_mirror,
        }
    }
}

#[no_mangle]
pub extern "C" fn functions() -> FunctionsBlock {
    FunctionsBlock::default()
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

    *handle = ImageHandle::from_image(img);
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

unsafe extern "C" fn img_blur(handle: ImageHandle, sigma: f32) -> ImageHandle {
    let image = handle.as_image();
    let buffer = image::imageops::blur(image, sigma);
    let blurred = image::DynamicImage::ImageRgba8(buffer);
    ImageHandle::from_image(blurred)
}

unsafe extern "C" fn img_mirror(handle: ImageHandle) {
    let image_ref = handle.as_image();
    image::imageops::flip_horizontal_in_place(image_ref);
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
