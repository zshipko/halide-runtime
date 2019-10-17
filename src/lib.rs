mod filter;
mod halide;

use halide::*;

pub use filter::{Filter, Manager};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum Kind {
    Int = halide_type_code_t::halide_type_int as u8,
    UInt = halide_type_code_t::halide_type_uint as u8,
    Float = halide_type_code_t::halide_type_float as u8,
}

/// Type is used to define the type of pixel data in terms of kind and bits
/// For example, Type(Kind::UInt, 8) uses one 8-bit unsigned integer per channel
/// and Type(Kind::Float, 32) uses a float per channel, etc...
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Type(pub Kind, pub u8);

impl Type {
    pub fn bits(&self) -> u8 {
        return self.1;
    }

    pub fn kind(&self) -> Kind {
        return self.0;
    }

    pub fn size(&self) -> usize {
        self.bits() as usize / 8
    }
}

/// Buffer wraps read-only image data in a way that can be passed
/// as an argument to Halide filters
pub struct Buffer(halide_buffer_t);

fn halide_buffer(
    width: i32,
    height: i32,
    channels: i32,
    t: Type,
    data: *mut u8,
) -> halide_buffer_t {
    let t = halide_type_t {
        code: t.0 as u8,
        bits: t.1,
        lanes: 1,
    };

    let mut dim = Vec::new();
    if channels > 1 {
        dim.push(halide_dimension_t {
            flags: 0,
            min: 0,
            extent: channels,
            stride: 1,
        });
    }

    dim.push(halide_dimension_t {
        flags: 0,
        min: 0,
        extent: width,
        stride: channels,
    });

    dim.push(halide_dimension_t {
        flags: 0,
        min: 0,
        extent: height,
        stride: channels * width,
    });

    dim.shrink_to_fit();

    let buf = halide_buffer_t {
        device: 0,
        device_interface: std::ptr::null(),
        dimensions: if channels < 2 { 2 } else { 3 },
        host: data,
        flags: 0,
        padding: std::ptr::null_mut(),
        type_: t,
        dim: dim.as_mut_ptr(),
    };

    std::mem::forget(dim);

    buf
}

impl Buffer {
    pub fn new(width: i32, height: i32, channels: i32, t: Type, data: *mut u8) -> Self {
        assert!(!data.is_null());
        Buffer(halide_buffer(width, height, channels, t, data))
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            Vec::from_raw_parts(
                self.0.dim,
                self.0.dimensions as usize,
                self.0.dimensions as usize,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn it_works() {
        let width = 800;
        let height = 600;
        let channels = 3;
        let t = Type(Kind::UInt, 8);
        let mut data = vec![0u8; width * height * channels * t.size()];
        let buf = Buffer::new(
            width as i32,
            height as i32,
            channels as i32,
            t,
            data.as_mut_ptr(),
        );

        let brighter_path = "./libbrighter.so";
        let mgr = Manager::new();
        assert!(mgr.load(brighter_path));
        let brighter: unsafe extern "C" fn(*const Buffer, u8, *mut Buffer) -> i32 =
            mgr.filter(brighter_path, "brighter").unwrap().get();

        let mut inplace = Buffer::new(
            width as i32,
            height as i32,
            channels as i32,
            t,
            data.as_mut_ptr(),
        );

        unsafe {
            assert!(brighter(&buf, 10, &mut inplace) == 0);
        }

        for i in data {
            assert!(i == 10);
        }
    }
}
