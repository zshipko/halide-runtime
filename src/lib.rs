pub mod runtime;

pub mod filter;

use runtime::*;

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
#[repr(C)]
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

    if channels > 1 {
        dim.push(halide_dimension_t {
            flags: 0,
            min: 0,
            extent: channels,
            stride: 1,
        });
    }

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

impl<'a> From<&'a halide_buffer_t> for Buffer {
    fn from(buf: &'a halide_buffer_t) -> Buffer {
        let mut dest = buf.clone();
        let mut dim = Vec::new();

        for i in 0..dest.dimensions as usize {
            unsafe {
                dim.push(*dest.dim.add(i));
            }
        }

        dest.dim = dim.as_mut_ptr();
        std::mem::forget(dim);

        Buffer(buf.clone())
    }
}

impl Clone for Buffer {
    fn clone(&self) -> Self {
        Buffer::from(&self.0)
    }
}

impl Buffer {
    pub fn new(width: i32, height: i32, channels: i32, t: Type, data: *mut u8) -> Self {
        assert!(!data.is_null());
        Buffer(halide_buffer(width, height, channels, t, data))
    }

    pub fn set_device(&mut self, device: u64, handle: Device) {
        self.0.device = device;
        self.0.device_interface = handle.0;
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
    use crate::filter::*;
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

        #[derive(WrapperApi)]
        struct Brighter {
            brighter: unsafe extern "C" fn(a: *const Buffer, b: *mut Buffer) -> i32,
        }

        let api = filter::load::<Brighter>("./libbrighter.so").unwrap();

        let mut out = Buffer::new(
            width as i32,
            height as i32,
            channels as i32,
            t,
            data.as_mut_ptr(),
        );

        unsafe {
            assert!(api.brighter(&buf, &mut out) == 0);
        }

        for i in data {
            assert!(i == 10);
        }
    }
}

extern "C" {
    fn halide_opencl_device_interface() -> *const halide_device_interface_t;

    fn halide_opengl_device_interface() -> *const halide_device_interface_t;

    fn halide_cuda_device_interface() -> *const halide_device_interface_t;
}

pub struct Device(*const halide_device_interface_t);

impl Device {
    pub fn opencl() -> Device {
        unsafe { Device(halide_opencl_device_interface()) }
    }

    pub fn opengl() -> Device {
        unsafe { Device(halide_opengl_device_interface()) }
    }

    pub fn cuda() -> Device {
        unsafe { Device(halide_cuda_device_interface()) }
    }
}

pub fn set_gpu_device(i: i32) {
    unsafe {
        halide_set_gpu_device(i);
    }
}
