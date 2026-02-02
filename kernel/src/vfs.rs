use crate::{
    ERR, Fd, OK, ResultCode,
    drivers::{DEV_NULL, DEV_SERIAL, Driver, NullDriver, SerialDriver},
};

const MAX_FDS: usize = 16;

pub struct FileDescriptor {
    pub dev: u32,
    pub inode: u32,
    pub offset: usize,
    pub flags: u32,
}

pub struct FdTable {
    fds: [Option<FileDescriptor>; MAX_FDS],
}

impl FdTable {
    pub const fn new() -> Self {
        Self {
            fds: [const { None }; MAX_FDS],
        }
    }

    pub fn allocate(&mut self, dev: u32, inode: u32, flags: u32) -> Result<Fd, ()> {
        for (i, slot) in self.fds.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(FileDescriptor {
                    dev,
                    inode,
                    offset: 0,
                    flags,
                });
                return Ok(i as Fd);
            }
        }
        Err(())
    }

    pub fn get(&self, fd: Fd) -> Option<&FileDescriptor> {
        if fd < 0 || fd >= MAX_FDS as Fd {
            return None;
        }
        self.fds[fd as usize].as_ref()
    }

    pub fn get_mut(&mut self, fd: Fd) -> Option<&mut FileDescriptor> {
        if fd < 0 || fd >= MAX_FDS as Fd {
            return None;
        }
        self.fds[fd as usize].as_mut()
    }

    pub fn close(&mut self, fd: Fd) -> ResultCode {
        if fd < 0 || fd >= MAX_FDS as Fd {
            return ERR;
        }
        self.fds[fd as usize] = None;
        OK
    }
}

pub struct Vfs {
    null: NullDriver,
    serial: SerialDriver,
}

impl Vfs {
    pub const fn new() -> Self {
        Self {
            serial: SerialDriver {},
            null: NullDriver {},
        }
    }

    pub fn open(&mut self, path: &str, flags: u32, fd_table: &mut FdTable) -> Fd {
        let dev = match path {
            "/dev/null" => DEV_NULL,
            "/dev/serial" => DEV_SERIAL,
            _ => return ERR,
        };

        fd_table.allocate(dev, 0, flags).unwrap_or(ERR)
    }

    pub fn write(&mut self, fd_desc: &mut FileDescriptor, buf: &[u8]) -> ResultCode {
        match fd_desc.dev {
            DEV_NULL => self.null.write(buf),
            DEV_SERIAL => self.serial.write(buf),
            _ => ERR,
        }
    }

    pub fn read(&mut self, fd_desc: &mut FileDescriptor, buf: &mut [u8]) -> ResultCode {
        match fd_desc.dev {
            DEV_NULL => self.null.read(buf),
            DEV_SERIAL => self.serial.read(buf),
            _ => ERR,
        }
    }

    pub fn ioctl(&mut self, fd_desc: &FileDescriptor, cmd: u32, arg: usize) -> ResultCode {
        match fd_desc.dev {
            DEV_NULL => self.null.ioctl(cmd, arg),
            DEV_SERIAL => self.serial.ioctl(cmd, arg),
            _ => ERR,
        }
    }
}
