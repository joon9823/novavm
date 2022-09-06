// copy from libwasmvm

use std::os::raw::c_char;

static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0"); // Add trailing NULL byte for C string

/// Returns a version number of this library as a C string.
///
/// The string is owned by libkernelvm and must not be mutated or destroyed by the caller.
#[no_mangle]
pub extern "C" fn version_str() -> *const c_char {
    VERSION.as_ptr() as *const _
}


#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    use crate::version::version_str;
    use crate::version::VERSION;

    #[test]
    fn test_version_str() {
        let ver = unsafe { CStr::from_ptr(version_str()) };
        let mut verstr= ver.to_str().expect("test failed").to_owned();
        verstr.push('\0');
        assert_eq!(verstr.as_str(), VERSION);
    }
}
