pub use dlopen::wrapper::Container;
pub use dlopen::wrapper::WrapperApi as Filter;

pub use dlopen_derive::*;

pub fn load_filter<T: Filter>(path: &str) -> Result<Container<T>, dlopen::Error> {
    unsafe { Container::load(path) }
}
