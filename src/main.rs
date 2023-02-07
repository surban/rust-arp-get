use std::env;
use std::net::{IpAddr, SocketAddr};

use macaddr::MacAddr6;

/// Look up to MAC address of an IP address.
fn ip_to_mac(addr: IpAddr, dev: &[u8]) -> std::io::Result<Option<MacAddr6>> {
    #[cfg(target_os = "linux")]
    {
        use libc::{arpreq, ioctl, ATF_COM, SIOCGARP};
        use os_socketaddr::OsSocketAddr;
        use std::{
            io::Error,
            mem::{size_of_val, MaybeUninit},
            os::fd::AsRawFd,
        };
        use tokio::net::TcpSocket;

        let socket = TcpSocket::new_v4()?;

        unsafe {
            let mut req = MaybeUninit::<arpreq>::zeroed().assume_init();
            let mut len = size_of_val(&req.arp_pa) as u32;

            let addr = OsSocketAddr::from(SocketAddr::new(addr, 0));
            addr.copy_to_raw(&mut req.arp_pa as _, &mut len as _)
                .unwrap();

            let dev: Vec<_> = dev.iter().map(|c| *c as i8).collect();
            req.arp_dev[..dev.len()].copy_from_slice(&dev);

            if ioctl(socket.as_raw_fd(), SIOCGARP, &mut req as *mut _) < 0 {
                return Err(Error::last_os_error());
            }

            if req.arp_flags & ATF_COM != 0 {
                let mac: [u8; 6] = req
                    .arp_ha
                    .sa_data
                    .iter()
                    .map(|v| *v as u8)
                    .take(6)
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap();
                Ok(Some(MacAddr6::from(mac)))
            } else {
                Ok(None)
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    Err(std::io::ErrorKind::Unsupported.into())
}

#[tokio::main]
async fn main() {
    let args: Vec<_> = env::args().collect();
    let ip: IpAddr = args[1].parse().unwrap();
    let dev = &args[2];

    let mac = ip_to_mac(ip, dev.as_bytes()).unwrap().unwrap();
    println!("{dev}: {ip} -> {mac}");
}
