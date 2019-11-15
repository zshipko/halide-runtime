pub use dlopen::wrapper::*;
pub use dlopen_derive::*;

pub use dlopen;

pub fn load<T: WrapperApi>(path: &str) -> Result<Container<T>, dlopen::Error> {
    unsafe { Container::load(path) }
}

pub fn load_self<T: WrapperApi>() -> Result<Container<T>, dlopen::Error> {
    unsafe { Container::load_self() }
}
