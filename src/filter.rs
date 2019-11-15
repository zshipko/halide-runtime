pub use dlopen::wrapper::*;
pub use dlopen_derive::*;

pub use dlopen;

pub fn load<T: WrapperApi>(path: &str) -> Result<Container<T>, dlopen::Error> {
    unsafe { Container::load(path) }
}
