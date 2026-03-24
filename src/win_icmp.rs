use std::mem;

use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::NetworkManagement::IpHelper::{
    ICMP_ECHO_REPLY, IcmpCloseHandle, IcmpCreateFile, IcmpSendEcho,
};
use windows_sys::Win32::Networking::WinSock::{WSACleanup, WSAStartup, WSADATA};

pub type EchoReply = ICMP_ECHO_REPLY;

pub struct WsaSession;

impl WsaSession {
    pub fn startup() -> Result<Self, i32> {
        unsafe {
            let mut wsa_data: WSADATA = mem::zeroed();
            let result = WSAStartup(0x0202, &mut wsa_data);
            if result != 0 {
                return Err(result);
            }
        }
        Ok(Self)
    }
}

impl Drop for WsaSession {
    fn drop(&mut self) {
        unsafe {
            WSACleanup();
        }
    }
}

pub struct IcmpHandle {
    raw: isize,
}

impl IcmpHandle {
    pub fn open() -> Result<Self, u32> {
        let raw = unsafe { IcmpCreateFile() };
        if raw == -1 {
            Err(last_error())
        } else {
            Ok(Self { raw })
        }
    }

    pub fn raw(&self) -> isize {
        self.raw
    }
}

impl Drop for IcmpHandle {
    fn drop(&mut self) {
        unsafe {
            IcmpCloseHandle(self.raw);
        }
    }
}

pub fn send_echo(
    handle: &IcmpHandle,
    dest_addr: u32,
    payload: &[u8],
    reply_buf: &mut [u8],
    timeout_ms: u32,
) -> u32 {
    unsafe {
        IcmpSendEcho(
            handle.raw(),
            dest_addr,
            payload.as_ptr() as *mut _,
            payload.len() as u16,
            std::ptr::null_mut(),
            reply_buf.as_mut_ptr() as *mut _,
            reply_buf.len() as u32,
            timeout_ms,
        )
    }
}

pub fn last_error() -> u32 {
    unsafe { GetLastError() }
}
