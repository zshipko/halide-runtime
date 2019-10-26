use std::cell::RefCell;
use std::collections::HashMap;

use dlopen::raw::Library;

pub struct Manager {
    libraries: RefCell<HashMap<String, Library>>,
}

pub struct Filter<'a, T>(pub T, &'a Manager);

impl<'a, T> Filter<'a, T> {
    pub fn get(self) -> T {
        self.0
    }
}

const SELF_KEY: &str = "____self";

impl Manager {
    pub fn new() -> Manager {
        Manager {
            libraries: RefCell::new(HashMap::new()),
        }
    }

    pub fn load_self(&self) -> bool {
        if (*self.libraries.borrow()).contains_key(SELF_KEY) {
            return true;
        }

        if let Ok(lib) = Library::open_self() {
            self.libraries.borrow_mut().insert(SELF_KEY.into(), lib);
            return true;
        }

        false
    }

    pub fn load(&self, name: &str) -> bool {
        if (*self.libraries.borrow()).contains_key(name) {
            return true;
        }

        if let Ok(lib) = Library::open(name) {
            self.libraries.borrow_mut().insert(name.into(), lib);
            return true;
        }

        false
    }

    pub fn remove(&self, name: &str) {
        if let Some(x) = self.libraries.borrow_mut().remove(name) {
            drop(x);
        }
    }

    pub fn reload(&self, name: &str) -> bool {
        self.remove(name);
        self.load(name)
    }

    pub fn filter<'a, T: Copy>(&'a self, lib: &str, name: &str) -> Option<Filter<'a, T>> {
        if let Some(lib) = self.libraries.borrow().get(lib) {
            match unsafe { lib.symbol::<T>(name) } {
                Ok(x) => return Some(Filter(x, self)),
                Err(_) => return None,
            }
        }

        None
    }
}
