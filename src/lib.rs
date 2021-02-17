pub mod ffi;
pub mod error;

use const_cstr::const_cstr;

use std::ptr;
use std::ffi::CString;
use std::sync::Once;

use error::FliteError;

pub type FliteResult<T> = Result<T, FliteError>;

macro_rules! drop_impl {
    ($st:ty) => {
        impl Drop for $st {
            fn drop(&mut self) {
                unsafe {
                    ffi::cst_free(self.inner as *mut std::ffi::c_void)
                }
            }
        }
    };
}

macro_rules! accessor {
    ($i:ident, $t:ty) => {
        pub fn $i(&self) -> $t {
            unsafe {(*self.inner).$i}
        }
    };
}

static FLITE_INIT: Once = Once::new();

const_cstr! {
    ENG = "eng";
    USENGLISH = "usenglish";
}

fn flite_init() {
    FLITE_INIT.call_once(|| unsafe {
        ffi::flite_init();
        ffi::flite_add_lang(ENG.as_ptr(), Some(ffi::usenglish_init), Some(ffi::cmu_lex_init));
        ffi::flite_add_lang(USENGLISH.as_ptr(), Some(ffi::usenglish_init), Some(ffi::cmu_lex_init));
    });
}

#[derive(Debug)]
pub struct Voice {
    inner: *mut ffi::cst_voice,
}

drop_impl!(Voice);

impl Default for Voice {
    fn default() -> Voice {
        flite_init();

        let inner = unsafe {
            ffi::register_cmu_us_kal(ptr::null())
        };

        Voice {
            inner
        }
    }
}

// TODO: make these actually work
/*
impl Voice {
    pub fn from_name(name: impl Into<Vec<u8>>) -> FliteResult<Voice> {
        flite_init();

        let name_cstring = CString::new(name)?;

        // TODO: Is using an unknown voice an error/unsafe (null)? 
        let inner = unsafe {
            ffi::flite_voice_select(name_cstring.as_ptr())
        };

        Ok(Voice {
            inner
        })
    }

    pub fn from_file(path: impl Into<Vec<u8>>) -> FliteResult<Voice> {
        flite_init();

        let path_cstring = CString::new(path)?;

        let inner = unsafe {
            ffi::flite_voice_load(path_cstring.as_ptr())
        };

        unsafe {
            ffi::flite_add_voice(inner);
        }

        Ok(Voice {
            inner
        })
    }
}*/

pub struct Wave {
    inner: *mut ffi::cst_wave
}

drop_impl!(Wave);

impl Wave {
    pub fn speak(text: impl Into<Vec<u8>>, voice: &Voice) -> FliteResult<Wave> {
        flite_init();

        let text_cstring = CString::new(text)?;

        let inner = unsafe {
            ffi::flite_text_to_wave(text_cstring.as_ptr(), voice.inner)
        };

        Ok(Wave {
            inner
        })
    }

    pub fn samples(&self) -> &[i16] {
        let len = (self.num_samples() * self.num_channels()) as usize;

        unsafe{std::slice::from_raw_parts((*self.inner).samples, len)}
    }

    accessor!(sample_rate, i32);
    accessor!(num_samples, i32);
    accessor!(num_channels, i32);
}
