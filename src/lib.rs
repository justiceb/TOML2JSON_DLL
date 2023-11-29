use libc::{c_char};
use std::{
    ffi::{CStr, CString},
    ptr,
};

#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn toml2json(
    toml_string: *const c_char,
    prettyprint: u8,
    json_string_len: *mut u32,
) -> *mut c_char {
    // convert c-string pointer input to RUST owned string
    let toml_string = unsafe { CStr::from_ptr(toml_string).to_string_lossy().into_owned() };

    // Turn our collected input into a value. We can't be more specific than
    // value, since we're doing arbitrary valid TOML conversions.
    let value = match toml::from_str::<toml::Value>(&toml_string) {
        Ok(t) => t,
        Err(_) => {
            unsafe { *json_string_len = 1; };
            return ptr::null_mut()
        },
    };

    // Spit back out, but as JSON. `serde_json` *does* support streaming, so
    // we do it.
    let mut json_string = String::new();
    match prettyprint {
        1 => json_string = match serde_json::to_string_pretty(&value) {
            Ok(t) => t,
            Err(_) => {
                unsafe { *json_string_len = 2; };
                return ptr::null_mut()
            },
        },
        _ => json_string = match serde_json::to_string(&value){
            Ok(t) => t,
            Err(_) => {
                unsafe { *json_string_len = 2; };
                return ptr::null_mut()
            },
        },
    }

    // pass the length of the string back to the caller through the num_bytes pointer
    let return_json_string_len = json_string.len() as u32;
    unsafe { *json_string_len = return_json_string_len; }

    // convert json string to ctring pointer to be passed to LabVIEW
    return string_to_cstring_ptr(&json_string)
}

fn string_to_cstring_ptr(s: &str) -> *mut c_char {
    let raw_string = match CString::new(s).unwrap().into_raw() {
        ptr if ptr.is_null() => {
            println!("Unable to allocate memory for string");
            return CString::new("").unwrap().into_raw();
        }
        ptr => ptr,
    };
    raw_string
}

// exported function that frees the memory allocated for a string
// this *must* be called for every string returned from a function in this library
#[no_mangle]
pub extern "C" fn cstring_free_memory(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe { let _ = CString::from_raw(s); };
}