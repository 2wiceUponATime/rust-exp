use core::{ffi::c_char, ptr::NonNull, str::Utf8Error};

use alloc::{
    borrow::ToOwned,
    ffi::NulError,
    string::{FromUtf8Error, String},
    vec::Vec,
};

use crate::{CStrArg, Free, IntoCStrPtr, Owned, as_ptr, opaque, variant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FResult {
    Ok = 0,
    DiskErr,
    IntErr,
    NotReady,
    NoFile,
    NoPath,
    InvalidName,
    Denied,
    Exist,
    InvalidObject,
    WriteProtected,
    InvalidDrive,
    NotEnabled,
    NoFilesystem,
    MkfsAborted,
    Timeout,
    Locked,
    NotEnoughCore,
    TooManyOpenFiles,
    InvalidParameter,
}

impl FResult {
    pub fn as_result(self) -> Result<(), FError> {
        Err(match self {
            Self::Ok => return Ok(()),
            Self::DiskErr => FError::DiskErr,
            Self::IntErr => FError::IntErr,
            Self::NotReady => FError::NotReady,
            Self::NoFile => FError::NoFile,
            Self::NoPath => FError::NoPath,
            Self::InvalidName => FError::InvalidName,
            Self::Denied => FError::Denied,
            Self::Exist => FError::Exist,
            Self::InvalidObject => FError::InvalidObject,
            Self::WriteProtected => FError::WriteProtected,
            Self::InvalidDrive => FError::InvalidDrive,
            Self::NotEnabled => FError::NotEnabled,
            Self::NoFilesystem => FError::NoFilesystem,
            Self::MkfsAborted => FError::MkfsAborted,
            Self::Timeout => FError::Timeout,
            Self::Locked => FError::Locked,
            Self::NotEnoughCore => FError::NotEnoughCore,
            Self::TooManyOpenFiles => FError::TooManyOpenFiles,
            Self::InvalidParameter => FError::InvalidParameter,
        })
    }
}

#[repr(u8)]
pub enum FError {
    DiskErr = 1,
    IntErr,
    NotReady,
    NoFile,
    NoPath,
    InvalidName,
    Denied,
    Exist,
    InvalidObject,
    WriteProtected,
    InvalidDrive,
    NotEnabled,
    NoFilesystem,
    MkfsAborted,
    Timeout,
    Locked,
    NotEnoughCore,
    TooManyOpenFiles,
    InvalidParameter,
}

variant!(
    pub enum OpenError {
        NulError(NulError),
        NullPointer,
    }
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum Whence {
    Start = 0,
    Current,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeekFrom {
    Start(u32),
    Current(i32),
    End(i32),
}

variant!(
    pub enum ListDirError {
        Fs(FError),
        Nul(NulError),
        Utf8(Utf8Error),
    }
);

variant!(
    pub enum SeekError {
        Fs(FError),
        NegativeTell,
        NegativeSize,
        OffsetUnderflow,
    }
);

const MIN_CHUNK: usize = 1024;
const MAX_CHUNK: usize = 4096;

#[cpp(func = "vexFileMountSD")]
fn mount_sd_inner() -> FResult;

pub fn mount_sd() -> Result<(), FError> {
    mount_sd_inner().as_result()
}

#[cpp(func = "vexFileStatus")]
fn file_status_inner(filename: *const c_char) -> u32;

pub fn file_status(filename: impl CStrArg) -> Result<u32, NulError> {
    Ok(file_status_inner(filename.into_cstr_ptr()?))
}

#[cpp(func = "vexFileDriveStatus")]
pub fn drive_status(drive: u32);

#[cpp(func = "vexFileDirectoryGet")]
fn list_dir_inner(path: *const c_char, buf: *mut c_char, len: u32) -> FResult;

pub fn list_dir(path: impl CStrArg) -> Result<Vec<String>, ListDirError> {
    let path = path.into_cstr_ptr()?;
    let mut cap = MIN_CHUNK;
    let bytes = loop {
        let mut buf = alloc::vec![0u8; cap];
        list_dir_inner(path, buf.as_mut_ptr(), cap as u32).as_result()?;

        let end = buf.iter().position(|&b| b == 0).unwrap_or(cap);
        if end < cap - 1 || cap >= MAX_CHUNK {
            buf.truncate(end);
            break buf;
        }
        cap *= 2;
    };
    let items: Vec<String> = bytes
        .split(|&b| b == 0)
        .map(|bytes| str::from_utf8(bytes).map(|s| s.to_owned()))
        .collect::<Result<_, _>>()?;
    Ok(items)
}

opaque!(pub File);

macro_rules! open_wrappers {
    ($($func:ident => $inner:ident),+ $(,)?) => {
        $(pub fn $func(filename: impl CStrArg) -> Result<Owned<Self>, OpenError> {
            unsafe {
                Owned::from_raw(Self::$inner(filename.into_cstr_ptr()?))
                    .ok_or(OpenError::NullPointer)
            }
        })+
    };
}

impl File {
    #[cpp(body = r#"return vexFileOpen(filename, "");"#)]
    fn open_read_inner(filename: *const c_char) -> *mut File;

    #[cpp(func = "vexFileOpenWrite", method = false)]
    fn open_write_inner(filename: *const c_char) -> *mut File;

    #[cpp(func = "vexFileOpenCreate", method = false)]
    fn open_create_inner(filename: *const c_char) -> *mut File;

    open_wrappers! {
        open_read => open_read_inner,
        open_write => open_write_inner,
        open_create => open_create_inner,
    }

    as_ptr!();

    #[cpp(func = "vexFileSize", method = false)]
    fn size_inner(&self) -> i32;

    pub fn size(&self) -> Option<u32> {
        self.size_inner().try_into().ok()
    }

    #[cpp(func = "vexFileTell", method = false)]
    fn tell_inner(&self) -> i32;

    pub fn tell(&self) -> Option<u32> {
        self.tell_inner().try_into().ok()
    }

    #[cpp(func = "vexFileSync", method = false)]
    pub fn sync(&self);

    #[cpp(body = "return vexFileRead(buf, 1, buf_len, self);")]
    fn read_inner(&self, buf: *mut c_char, buf_len: u32) -> i32;

    pub fn read_to_buffer(&self, buf: &mut [u8]) -> Option<u32> {
        self.read_inner(buf.as_mut_ptr(), buf.len() as u32)
            .try_into()
            .ok()
    }

    pub fn read_to_end(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut len = 0;

        loop {
            if len == buf.len() {
                buf.resize((buf.len() * 2).max(MIN_CHUNK), 0);
            }
            let Some(bytes_read) = self.read_to_buffer(&mut buf[len..]) else {
                break;
            };
            len += bytes_read as usize;
        }

        buf.truncate(len);
        buf
    }

    pub fn read_to_string(&self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.read_to_end())
    }

    #[cpp(body = "return vexFileWrite(buf, 1, buf_len, self);")]
    fn write_inner(&self, buf: *const c_char, buf_len: u32) -> i32;

    pub fn write(&self, bytes: &[u8]) -> Option<u32> {
        self.write_inner(bytes.as_ptr(), bytes.len() as u32)
            .try_into()
            .ok()
    }

    #[cpp(func = "vexFileSeek", method = false)]
    fn seek_inner(&self, offset: u32, whence: Whence) -> FResult;

    pub fn seek(&self, pos: SeekFrom) -> Result<(), SeekError> {
        let (offset, whence) = match pos {
            SeekFrom::Start(n) => (n, Whence::Start),
            SeekFrom::Current(n) if n >= 0 => (n as u32, Whence::Current),
            SeekFrom::End(n) if n >= 0 => (n as u32, Whence::End),
            SeekFrom::Current(n) => (
                self.tell()
                    .ok_or(SeekError::NegativeTell)?
                    .checked_add_signed(n)
                    .ok_or(SeekError::OffsetUnderflow)?,
                Whence::Start,
            ),
            SeekFrom::End(n) => (
                self.size()
                    .ok_or(SeekError::NegativeSize)?
                    .checked_add_signed(n)
                    .ok_or(SeekError::OffsetUnderflow)?,
                Whence::Start,
            ),
        };
        self.seek_inner(offset, whence)
            .as_result()
            .map_err(Into::into)
    }
}

impl Free for File {
    #[cpp(func = "vexFileClose")]
    unsafe fn free(handle: NonNull<File>);
}
