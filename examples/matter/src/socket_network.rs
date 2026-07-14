use core::net::SocketAddr;

use embassy_net::{IpEndpoint, udp::UdpSocket};
use rs_matter::{error::{Error, ErrorCode::AttributeNotFound}, tlv::TLVValue::False, transport::network::{Address, NetworkReceive, NetworkSend}};

use crate::socket_utils::{ipendpoint_to_socket_address, socket_to_ipendpoint, socket_to_listenendpoint};

pub struct SocketNetwork<'a> {
    pub(crate) inner: &'a mut UdpSocket<'a>
}

impl NetworkSend for &SocketNetwork<'_> {
    async fn send_to(&mut self, data: &[u8], addr: Address) -> Result<(), Error> {
        
        let socket_addres = match addr {
            Address::Udp(ref a) => *a, 
            Address::Tcp(ref a) => *a, 
            _ => panic!("Not implemented") 
        };
        
        let ip_endpoint: IpEndpoint = socket_to_ipendpoint(socket_addres);

        match self.inner.send_to(data, ip_endpoint).await {
            Ok(o) => Ok(o),
            Err(_e) => Err(Error::new(AttributeNotFound))
        }
    }
}

impl NetworkReceive for &SocketNetwork<'_> {
    async fn wait_available(&mut self) -> Result<(), Error> {
        match self.inner.may_recv() {
            true => Ok(()),
            false => Err(Error::new(AttributeNotFound))
        }
    }

    async fn recv_from(&mut self, buffer: &mut [u8]) -> Result<(usize, Address), Error> {
        match self.inner.recv_from(buffer).await {
            Ok((size, endpoint)) => Ok((size, Address::Udp(ipendpoint_to_socket_address(endpoint.endpoint)))),
            Err(_e) => Err(Error::new(AttributeNotFound))
        }
    }
}