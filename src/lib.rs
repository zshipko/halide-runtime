//! halide-runtime is a Rust wrapper for the [Halide](https://github.com/halide/Halide) runtime

use std::marker::PhantomData;

pub mod runtime;
use runtime::*;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum Kind {
    Int = halide_type_code_t::halide_type_int as u8,
    UInt = halide_type_code_t::halide_type_uint as u8,
    Float = halide_type_code_t::halide_type_float as u8,
}

/// Type is used to define the type of pixel data in terms of kind and bits
/// For example, Type::new(Kind::UInt, 8) uses one 8-bit unsigned integer per channel
/// and Type::new(Kind::Float, 32) uses a float per channel, etc...
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Type(pub Kind, pub u8, pub u16);

impl Type {
    pub fn new(kind: Kind, bits: u8) -> Type {
        Type(kind, bits, 1)
    }

    pub fn new_with_lanes(kind: Kind, bits: u8, lanes: u16) -> Type {
        Type(kind, bits, lanes)
    }

    pub fn bits(&self) -> u8 {
        self.1
    }

    pub fn kind(&self) -> Kind {
        self.0
    }

    pub fn size(&self) -> usize {
        self.bits() as usize / 8
    }
}

/// Buffer wraps image data in a way that can be passed
/// as an argument to Halide filters
#[repr(transparent)]
pub struct Buffer<'a>(halide_buffer_t, PhantomData<&'a ()>);

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
        lanes: t.2,
    };

    let mut dim = vec![
        halide_dimension_t {
            flags: 0,
            min: 0,
            extent: width,
            stride: channels,
        },
        halide_dimension_t {
            flags: 0,
            min: 0,
            extent: height,
            stride: channels * width,
        },
    ];

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

impl<'a> From<&'a halide_buffer_t> for Buffer<'a> {
    fn from(buf: &'a halide_buffer_t) -> Buffer {
        let mut dest = *buf;
        let mut dim = Vec::new();

        for i in 0..dest.dimensions as usize {
            unsafe {
                dim.push(*dest.dim.add(i));
            }
        }

        dest.dim = dim.as_mut_ptr();
        std::mem::forget(dim);

        Buffer(dest, PhantomData)
    }
}

impl<'a> Clone for Buffer<'a> {
    fn clone(&self) -> Self {
        let mut dest = self.0;
        let mut dim = Vec::new();

        for i in 0..dest.dimensions as usize {
            unsafe {
                dim.push(*dest.dim.add(i));
            }
        }

        dest.dim = dim.as_mut_ptr();
        std::mem::forget(dim);

        Buffer(dest, PhantomData)
    }
}

impl<'a> Buffer<'a> {
    pub fn new<T>(width: i32, height: i32, channels: i32, t: Type, data: &'a mut [T]) -> Self {
        Buffer(
            halide_buffer(width, height, channels, t, data.as_mut_ptr() as *mut u8),
            PhantomData,
        )
    }

    pub fn new_const<T>(width: i32, height: i32, channels: i32, t: Type, data: &'a [T]) -> Self {
        Buffer(
            halide_buffer(width, height, channels, t, data.as_ptr() as *mut u8),
            PhantomData,
        )
    }

    pub fn copy_to_host(&mut self) {
        unsafe {
            runtime::halide_copy_to_host(std::ptr::null_mut(), &mut self.0);
        }
    }

    #[cfg(feature = "gpu")]
    pub fn copy_to_device(&mut self, device: &gpu::Device) {
        unsafe {
            runtime::halide_copy_to_device(std::ptr::null_mut(), &mut self.0, device.0);
        }
    }
}

impl<'a> Drop for Buffer<'a> {
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

#[cfg(feature = "gpu")]
pub mod gpu {
    use crate::*;

    extern "C" {
        fn halide_opencl_device_interface() -> *const halide_device_interface_t;

        fn halide_opengl_device_interface() -> *const halide_device_interface_t;

        fn halide_cuda_device_interface() -> *const halide_device_interface_t;

        #[cfg(target_os = "macos")]
        fn halide_metal_device_interface() -> *const halide_device_interface_t;
    }

    pub struct Device(pub *const halide_device_interface_t);

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

        #[cfg(target_os = "macos")]
        pub fn metal() -> Device {
            unsafe { Device(halide_metal_device_interface()) }
        }
    }

    pub fn set_gpu_device(i: i32) {
        unsafe {
            halide_set_gpu_device(i);
        }
    }

    pub fn get_gpu_device() {
        unsafe {
            halide_get_gpu_device(std::ptr::null_mut());
        }
    }

    impl<'a> Buffer<'a> {
        pub fn set_device(&mut self, device: u64, handle: Device) {
            self.0.device = device;
            self.0.device_interface = handle.0;
        }
    }
}

pub type Status = runtime::Status;

#[cfg(test)]
mod tests {
    use crate::*;

    extern "C" {
        pub fn brighter(a: *const Buffer, b: *mut Buffer) -> Status;
    }

    #[test]
    fn it_works() {
        let width = 800;
        let height = 600;
        let channels = 3;
        let t = Type::new(Kind::UInt, 8);
        let input = vec![0u8; width * height * channels * t.size()];
        let mut output = vec![0u8; width * height * channels * t.size()];

        {
            let buf = Buffer::new_const(width as i32, height as i32, channels as i32, t, &input);
            let mut out = Buffer::new(width as i32, height as i32, channels as i32, t, &mut output);

            unsafe {
                assert!(brighter(&buf, &mut out) == Status::Success);
            }

            out.copy_to_host();
        }

        for i in output.iter() {
            assert!(*i == 10);
        }
    }
}
