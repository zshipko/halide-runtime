pub use dlopen::wrapper::Container;
pub use dlopen::wrapper::WrapperApi;
pub use dlopen_derive::*;

pub fn load_filter<T: WrapperApi>(path: &str) -> Result<Container<T>, dlopen::Error> {
    unsafe { Container::load(path) }
}
