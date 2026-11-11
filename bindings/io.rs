use core::fmt::{self, Write};

#[cpp(body = "return errno;")]
fn errno() -> i32;

#[cpp(body = r#"printf("%.*s", (int)value_len, value);"#)]
pub fn print_str(value: &str);

#[cpp(body = "return fflush(stdout);")]
fn flush_inner() -> i32;

pub fn flush() -> Result<(), IoError> {
    if flush_inner() == 0 {
        Ok(())
    } else {
        Err(IoError::from_raw(errno()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IoError(i32);

impl IoError {
    pub const fn from_raw(errno: i32) -> Self {
        Self(errno)
    }

    pub const fn raw(self) -> i32 {
        self.0
    }

    pub const fn kind(self) -> IoErrorKind {
        IoErrorKind::Other
    }

    pub const fn as_result(self) -> Result<(), Self> {
        if self.raw() == 0 { Ok(()) } else { Err(self) }
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let _ = Stdout.write_fmt(args);
}

impl From<IoError> for Result<(), IoError> {
    fn from(value: IoError) -> Self {
        value.as_result()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum IoErrorKind {
    Other,
}

struct Stdout;

impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        print_str(s);
        Ok(())
    }
}
