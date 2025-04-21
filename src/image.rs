use std::ffi::c_void;
use std::ffi::CString;

use stb_image::stb_image::stbi_image_free;
use stb_image::stb_image::stbi_load;

pub struct Image {
    pub pixels: Vec<u8>,
    pub width: i32,
    pub height: i32,
}

impl Image {
    pub fn load(file_path: &str) -> Self {
        let c_path = CString::new(file_path).unwrap();

        let mut width = 0;
        let mut height = 0;
        let mut channels = 3;
        let stbi_rgb_alpha = 4;

        let pixels = unsafe {
            stbi_load(
                c_path.as_ptr(),
                &mut width,
                &mut height,
                &mut channels,
                stbi_rgb_alpha,
            )
        };

        if pixels.is_null() {
            panic!("image is null after load");
        }

        Image {
            pixels: unsafe { std::slice::from_raw_parts(pixels, (width * height * 4) as usize) }
                .to_vec(),
            width,
            height,
        }
    }
}

use std::mem;

impl Drop for Image {
    fn drop(&mut self) {
        let mut pixels = mem::replace(&mut self.pixels, vec![]);
        unsafe { stbi_image_free(pixels.as_mut_ptr() as *mut c_void) };
        mem::forget(pixels);
    }
}

use std::ops::Deref;

impl Deref for Image {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.pixels
    }
}

use std::ops::DerefMut;

impl DerefMut for Image {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pixels
    }
}
