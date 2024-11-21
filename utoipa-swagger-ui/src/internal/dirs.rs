// based on https://github.com/pykeio/ort/blob/main/ort-sys/src/internal/dirs.rs and https://github.com/dirs-dev/dirs-sys-rs/blob/main/src/lib.rs

pub const PACKAGE_NAME: &str = "utoipa-swagger-ui";

#[cfg(all(target_os = "windows", target_arch = "x86"))]
macro_rules! win32_extern {
    ($library:literal $abi:literal $($link_name:literal)? $(#[$doc:meta])? fn $($function:tt)*) => (
        #[link(name = $library, kind = "raw-dylib", modifiers = "+verbatim", import_name_type = "undecorated")]
        extern $abi {
            $(#[$doc])?
            $(#[link_name=$link_name])?
            fn $($function)*;
        }
    )
}
#[cfg(all(target_os = "windows", not(target_arch = "x86")))]
macro_rules! win32_extern {
	($library:literal $abi:literal $($link_name:literal)? $(#[$doc:meta])? fn $($function:tt)*) => (
		#[link(name = $library, kind = "raw-dylib", modifiers = "+verbatim")]
		extern "C" {
			$(#[$doc])?
			$(#[link_name=$link_name])?
			fn $($function)*;
		}
	)
}

#[cfg(target_os = "windows")]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
mod windows {
    use std::{
        ffi::{c_void, OsString},
        os::windows::prelude::OsStringExt,
        path::PathBuf,
        ptr, slice,
    };

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct GUID {
        data1: u32,
        data2: u16,
        data3: u16,
        data4: [u8; 8],
    }

    impl GUID {
        pub const fn from_u128(uuid: u128) -> Self {
            Self {
                data1: (uuid >> 96) as u32,
                data2: (uuid >> 80 & 0xffff) as u16,
                data3: (uuid >> 64 & 0xffff) as u16,
                #[allow(clippy::cast_possible_truncation)]
                data4: (uuid as u64).to_be_bytes(),
            }
        }
    }

    type HRESULT = i32;
    type PWSTR = *mut u16;
    type PCWSTR = *const u16;
    type HANDLE = isize;
    type KNOWN_FOLDER_FLAG = i32;

    win32_extern!("SHELL32.DLL" "system" fn SHGetKnownFolderPath(rfid: *const GUID, dwflags: KNOWN_FOLDER_FLAG, htoken: HANDLE, ppszpath: *mut PWSTR) -> HRESULT);
    win32_extern!("KERNEL32.DLL" "system" fn lstrlenW(lpstring: PCWSTR) -> i32);
    win32_extern!("OLE32.DLL" "system" fn CoTaskMemFree(pv: *const ::core::ffi::c_void) -> ());

    fn known_folder(folder_id: GUID) -> Option<PathBuf> {
        unsafe {
            let mut path_ptr: PWSTR = ptr::null_mut();
            let result = SHGetKnownFolderPath(&folder_id, 0, HANDLE::default(), &mut path_ptr);
            if result == 0 {
                let len = lstrlenW(path_ptr) as usize;
                let path = slice::from_raw_parts(path_ptr, len);
                let ostr: OsString = OsStringExt::from_wide(path);
                CoTaskMemFree(path_ptr as *const c_void);
                Some(PathBuf::from(ostr))
            } else {
                CoTaskMemFree(path_ptr as *const c_void);
                None
            }
        }
    }

    #[allow(clippy::unusual_byte_groupings)]
    const FOLDERID_LOCAL_APP_DATA: GUID = GUID::from_u128(0xf1b32785_6fba_4fcf_9d557b8e7f157091);

    #[must_use]
    pub fn known_folder_local_app_data() -> Option<PathBuf> {
        known_folder(FOLDERID_LOCAL_APP_DATA)
    }
}
#[cfg(target_os = "windows")]
#[must_use]
pub fn cache_dir() -> Option<std::path::PathBuf> {
    self::windows::known_folder_local_app_data().map(|h| h.join(PACKAGE_NAME))
}

#[cfg(unix)]
#[allow(non_camel_case_types)]
mod unix {
    use std::{
        env,
        ffi::{c_char, c_int, c_long, CStr, OsString},
        mem,
        os::unix::prelude::OsStringExt,
        path::PathBuf,
        ptr,
    };

    type uid_t = u32;
    type gid_t = u32;
    type size_t = usize;
    #[repr(C)]
    struct passwd {
        pub pw_name: *mut c_char,
        pub pw_passwd: *mut c_char,
        pub pw_uid: uid_t,
        pub pw_gid: gid_t,
        pub pw_gecos: *mut c_char,
        pub pw_dir: *mut c_char,
        pub pw_shell: *mut c_char,
    }

    extern "C" {
        fn sysconf(name: c_int) -> c_long;
        fn getpwuid_r(
            uid: uid_t,
            pwd: *mut passwd,
            buf: *mut c_char,
            buflen: size_t,
            result: *mut *mut passwd,
        ) -> c_int;
        fn getuid() -> uid_t;
    }

    const SC_GETPW_R_SIZE_MAX: c_int = 70;

    #[must_use]
    #[cfg(target_os = "linux")]
    pub fn is_absolute_path(path: OsString) -> Option<PathBuf> {
        let path = PathBuf::from(path);
        if path.is_absolute() {
            Some(path)
        } else {
            None
        }
    }

    #[cfg(not(target_os = "windows"))]
    #[must_use]
    pub fn home_dir() -> Option<PathBuf> {
        return env::var_os("HOME")
            .and_then(|h| if h.is_empty() { None } else { Some(h) })
            .or_else(|| unsafe { fallback() })
            .map(PathBuf::from);

        #[cfg(any(target_os = "android", target_os = "ios", target_os = "emscripten"))]
        unsafe fn fallback() -> Option<OsString> {
            None
        }
        #[cfg(not(any(target_os = "android", target_os = "ios", target_os = "emscripten")))]
        unsafe fn fallback() -> Option<OsString> {
            let amt = match sysconf(SC_GETPW_R_SIZE_MAX) {
                n if n < 0 => 512,
                n => n as usize,
            };
            let mut buf = Vec::with_capacity(amt);
            let mut passwd: passwd = mem::zeroed();
            let mut result = ptr::null_mut();
            match getpwuid_r(
                getuid(),
                &mut passwd,
                buf.as_mut_ptr(),
                buf.capacity(),
                &mut result,
            ) {
                0 if !result.is_null() => {
                    let ptr = passwd.pw_dir as *const _;
                    let bytes = CStr::from_ptr(ptr).to_bytes();
                    if bytes.is_empty() {
                        None
                    } else {
                        Some(OsStringExt::from_vec(bytes.to_vec()))
                    }
                }
                _ => None,
            }
        }
    }
}

#[cfg(target_os = "linux")]
#[must_use]
pub fn cache_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("XDG_CACHE_HOME")
        .and_then(self::unix::is_absolute_path)
        .or_else(|| self::unix::home_dir().map(|h| h.join(".cache").join(PACKAGE_NAME)))
}

#[cfg(target_os = "macos")]
#[must_use]
pub fn cache_dir() -> Option<std::path::PathBuf> {
    self::unix::home_dir().map(|h| h.join("Library/Caches").join(PACKAGE_NAME))
}
