use std::io;
use std::os::unix::io::RawFd;

// taken from tokio/Mio
#[allow(unused_macros)]
macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        let res = unsafe { libc::$fn($($arg, )*) };
        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}
enum EpollOp {
    AddInterest,
    ChangeEvent,
    Deregister,
}

impl Into<i32> for EpollOp {
    fn into(self) -> i32 {
        match self {
            EpollOp::AddInterest => libc::EPOLL_CTL_ADD,
            EpollOp::ChangeEvent => libc::EPOLL_CTL_MOD,
            EpollOp::Deregister => libc::EPOLL_CTL_DEL,
        }
    }
}

pub struct EpollInstance {
    fd: RawFd,
}

impl EpollInstance {
    pub fn create(size: i32) -> io::Result<EpollInstance> {
        let fd = syscall!(epoll_create(size))?;
        if let Ok(flags) = syscall!(fcntl(fd, libc::F_GETFD)) {
            let _ = syscall!(fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC));
        }
        Ok(EpollInstance { fd })
    }

    pub fn create1(flags: i32) -> io::Result<EpollInstance> {
        let fd = syscall!(epoll_create1(flags))?;
        if let Ok(flags) = syscall!(fcntl(fd, libc::F_GETFD)) {
            let _ = syscall!(fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC));
        }
        Ok(EpollInstance { fd })
    }

    // This system call is used to add, modify, or remove entries in the
    // interest list of the epoll(7) instance referred to by the file
    // descriptor epfd.  It requests that the operation op be performed
    // for the target file descriptor, fd.
    fn ctl(&self, op: EpollOp, fd: RawFd, mut event: Option<libc::epoll_event>) -> io::Result<()> {
        let libc_op: i32 = op.into();
        let _ = match event {
            Some(mut e) => syscall!(epoll_ctl(self.fd, libc_op, fd, &mut e))?,
            None => syscall!(epoll_ctl(self.fd, libc_op, fd, std::ptr::null_mut()))?,
        };
        Ok(())
    }

    pub fn add_interest(&self, fd: RawFd, mut event: libc::epoll_event) -> io::Result<()> {
        self.ctl(EpollOp::AddInterest, fd, Some(event))
    }

    pub fn change_event(&self, fd: RawFd, mut event: libc::epoll_event) -> io::Result<()> {
        self.ctl(EpollOp::ChangeEvent, fd, Some(event))
    }

    pub fn deregister(&self, fd: RawFd) -> io::Result<()> {
        self.ctl(EpollOp::Deregister, fd, None)
    }

    pub fn wait(
        &self,
        events: *mut libc::epoll_event,
        max_events: i32,
        timeout: i32,
    ) -> io::Result<i32> {
        syscall!(epoll_wait(self.fd, events, max_events, timeout))
    }
}

impl Drop for EpollInstance {
    fn drop(&mut self) {
        let _ = syscall!(close(self.fd));
    }
}
