use std::ffi::{CStr, CString};
use std::path::Path;
use std::sync::Arc;

use libloading::Library;

use crate::img::bindings::{ImageError, ImageHandle, RawPath};
use bindings::{Functions, FunctionsFn};

mod bindings;

pub struct ImageFactory {
    lib: Lib,
}

impl ImageFactory {
    pub fn new() -> Result<Self, anyhow::Error> {
        let lib = unsafe {
            let lib = libloading::Library::new(bindings::lib_path())?;
            Lib::new(lib)?
        };

        Ok(Self { lib })
    }

    pub fn open_image<P: AsRef<Path>>(&self, path: P) -> Result<Image, anyhow::Error> {
        Image::open(self.lib.clone(), path)
    }
}

pub struct Image {
    lib: Lib,
    handle: ImageHandle,
}

impl Image {
    fn open<P: AsRef<Path>>(lib: Lib, path: P) -> Result<Self, anyhow::Error> {
        let path_cstring = path_to_cstring(path)?;
        let handle = unsafe { lib.open_image(&path_cstring) }?;
        Ok(Self { lib, handle })
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), anyhow::Error> {
        let path_cstring = path_to_cstring(path)?;
        unsafe { Ok(self.lib.save_image(self.handle, &path_cstring)?) }
    }

    pub fn blur(&self, sigma: f32) -> Self {
        let handle = unsafe { self.lib.blur_image(self.handle, sigma) };
        Self {
            lib: self.lib.clone(),
            handle,
        }
    }

    pub fn mirror(&mut self) {
        unsafe { self.lib.mirror_image(self.handle) }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            self.lib.destroy_image(self.handle);
        }
    }
}

fn path_to_cstring<P: AsRef<Path>>(path: P) -> Result<CString, anyhow::Error> {
    let path_str = path
        .as_ref()
        .to_str()
        .ok_or_else(|| anyhow::Error::msg("Only UTF-8 file path accepted"))?;

    let path_bytes = path_str.as_bytes();
    Ok(CString::new(path_bytes)?)
}

#[derive(Clone)]
struct Lib {
    lib: Arc<Library>,
    functions: Functions,
}

impl Lib {
    pub unsafe fn new(lib: Library) -> Result<Self, anyhow::Error> {
        let load_fn: libloading::Symbol<FunctionsFn> = lib.get(b"functions")?;
        let functions = load_fn();

        if functions.size != std::mem::size_of::<Functions>() {
            return Err(anyhow::Error::msg(
                "Lib Functions size != app Functions size",
            ));
        }

        Ok(Self {
            lib: Arc::new(lib),
            functions,
        })
    }

    pub unsafe fn open_image(&self, path: &CStr) -> Result<ImageHandle, ImageError> {
        let raw_path = path.as_ptr();
        let mut handle = ImageHandle::new_null();
        let err = (self.functions.open_image)(RawPath(raw_path), &mut handle);
        match err {
            ImageError::NoError => Ok(handle),
            err => Err(err),
        }
    }

    pub unsafe fn save_image(&self, handle: ImageHandle, path: &CStr) -> Result<(), ImageError> {
        let raw_path = path.as_ptr();

        let err = (self.functions.save_image)(RawPath(raw_path), handle);
        match err {
            ImageError::NoError => Ok(()),
            err => Err(err),
        }
    }

    pub unsafe fn destroy_image(&self, handle: ImageHandle) {
        (self.functions.destroy_image)(handle)
    }

    pub unsafe fn blur_image(&self, handle: ImageHandle, sigma: f32) -> ImageHandle {
        (self.functions.blur_image)(handle, sigma)
    }

    pub unsafe fn mirror_image(&self, handle: ImageHandle) {
        (self.functions.mirror_image)(handle)
    }
}
