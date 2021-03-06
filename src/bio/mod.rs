use libc::{c_void, c_int};
use std::io::{EndOfFile, IoResult, IoError, OtherIoError};
use std::io::{Reader, Writer};
use std::ptr;

use ffi;
use ssl::error::{SslError};

pub struct MemBio {
    bio: *mut ffi::BIO,
    owned: bool
}

impl Drop for MemBio {
    fn drop(&mut self) {
        if self.owned {
            unsafe {
                ffi::BIO_free_all(self.bio);
            }
        }
    }
}

impl MemBio {
    /// Creates a new owned memory based BIO
    pub fn new() -> Result<MemBio, SslError> {
        ffi::init();

        let bio = unsafe { ffi::BIO_new(ffi::BIO_s_mem()) };
        try_ssl_null!(bio);

        Ok(MemBio {
            bio: bio,
            owned: true
        })
    }

    /// Returns a "borrow", i.e. it has no ownership
    pub fn borrowed(bio: *mut ffi::BIO) -> MemBio {
        MemBio {
            bio: bio,
            owned: false
        }
    }

    /// Consumes current bio and returns wrapped value
    /// Note that data ownership is lost and
    /// should be managed manually
    pub unsafe fn unwrap(mut self) -> *mut ffi::BIO {
        self.owned = false;
        self.bio
    }

    /// Temporarily gets wrapped value
    pub unsafe fn get_handle(&self) -> *mut ffi::BIO {
        self.bio
    }
}

impl Reader for MemBio {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        let ret = unsafe {
            ffi::BIO_read(self.bio, buf.as_ptr() as *mut c_void,
                          buf.len() as c_int)
        };

        if ret <= 0 {
            let is_eof = unsafe { ffi::BIO_eof(self.bio) };
            let err = if is_eof {
                IoError {
                    kind: EndOfFile,
                    desc: "MemBio EOF",
                    detail: None
                }
            } else {
                IoError {
                    kind: OtherIoError,
                    desc: "MemBio read error",
                    detail: Some(format!("{}", SslError::get()))
                }
            };
            Err(err)
        } else {
            Ok(ret as uint)
        }
    }
}

impl Writer for MemBio {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        let ret = unsafe {
            ffi::BIO_write(self.bio, buf.as_ptr() as *const c_void,
                           buf.len() as c_int)
        };
        if buf.len() != ret as uint {
            Err(IoError {
                kind: OtherIoError,
                desc: "MemBio write error",
                detail: Some(format!("{}", SslError::get()))
            })
        } else {
            Ok(())
        }
    }
}
