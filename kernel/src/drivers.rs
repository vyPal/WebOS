use crate::{ERR, ResultCode, serial_write};

pub const DEV_NULL: u32 = 0;
pub const DEV_SERIAL: u32 = 1;

pub trait Driver {
    fn read(&mut self, buf: &mut [u8]) -> ResultCode;
    fn write(&mut self, buf: &[u8]) -> ResultCode;
    fn ioctl(&mut self, cmd: u32, arg: usize) -> ResultCode;
}

pub struct NullDriver {}

impl Driver for NullDriver {
    fn read(&mut self, _buf: &mut [u8]) -> ResultCode {
        0
    }

    fn write(&mut self, buf: &[u8]) -> ResultCode {
        buf.len() as ResultCode
    }

    fn ioctl(&mut self, _cmd: u32, _arg: usize) -> ResultCode {
        ERR
    }
}

pub struct SerialDriver {}

impl Driver for SerialDriver {
    fn read(&mut self, _buf: &mut [u8]) -> ResultCode {
        // TODO: Handle input once serial is moved to console, will probably require poll
        ERR
    }

    fn write(&mut self, buf: &[u8]) -> ResultCode {
        unsafe { serial_write(buf.as_ptr(), buf.len()) }
    }

    fn ioctl(&mut self, _cmd: u32, _arg: usize) -> ResultCode {
        ERR
    }
}
