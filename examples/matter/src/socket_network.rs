
use embassy_net::Stack;
use embassy_net::{IpEndpoint, udp::UdpSocket};
use rs_matter::error::ErrorCode;
use rs_matter::{error::Error, transport::network::{Address, NetworkMulticast, NetworkReceive, NetworkSend}};

use ariel_os::time::Timer;
use ariel_os::log::info;

use crate::socket_utils::{ipendpoint_to_socket_address, socket_to_ipendpoint};

pub struct SocketNetwork<'a> {
    pub(crate) inner: &'a mut &'a UdpSocket<'a>,
    pub(crate) stack: &'a Stack<'a>
}

impl NetworkSend for &SocketNetwork<'_> {
    async fn send_to(&mut self, data: &[u8], addr: Address) -> Result<(), Error> {
        
        let socket_addres = match addr {
            Address::Udp(ref a) => *a, 
            Address::Tcp(ref a) => *a, 
            _ => panic!("Not implemented") 
        };
        
        let ip_endpoint: IpEndpoint = socket_to_ipendpoint(socket_addres);

        // match self.inner.send_to(data, ip_endpoint).await {
        //     Ok(o) => Ok(o),
        //     Err(_e) => Err(Error::new(ErrorCode::StdIoError))
        // }

        info!("{}", self.inner.payload_send_capacity());
        info!("{}", data.len());

        self.inner.send_to(data, ip_endpoint).await.expect("Noooo");

        Ok(())
    }
}

impl NetworkReceive for &SocketNetwork<'_> {
    async fn wait_available(&mut self) -> Result<(), Error> {
        while !self.inner.may_recv() {
            Timer::after_millis(100).await;
        }

        Ok(())
    }

    async fn recv_from(&mut self, buffer: &mut [u8]) -> Result<(usize, Address), Error> {
        match self.inner.recv_from(buffer).await {
            Ok((size, endpoint)) => Ok((size, Address::Udp(ipendpoint_to_socket_address(endpoint.endpoint)))),
            Err(_e) => Err(Error::new(ErrorCode::RxTimeout))
        }
    }
}

impl NetworkMulticast for &SocketNetwork<'_> {
    async fn join(&mut self, addr: core::net::IpAddr) -> Result<(), Error> {
        match self.stack.join_multicast_group(addr) {
            Ok(o) => Ok(o),
            Err(_e) => Err(Error::new(ErrorCode::MdnsError))
        }
    }

    async fn leave(&mut self, addr: core::net::IpAddr) -> Result<(), Error> {
        match self.stack.leave_multicast_group(addr) {
            Ok(o) => Ok(o),
            Err(_e) => Err(Error::new(ErrorCode::Busy))
        }
    }
}