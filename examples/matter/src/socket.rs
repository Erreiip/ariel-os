
use core::net::{IpAddr, Ipv6Addr, SocketAddr, SocketAddrV6};
use embassy_net::{IpAddress, IpListenEndpoint};

pub fn socket_to_listenendpoint(x: SocketAddr) -> IpListenEndpoint {

    let ipv6_slice = match x {
        SocketAddr::V6(ref a) => *a.ip(),
        _ => panic!("Not possible")
    }.octets();

    let ip_address = IpAddress::v6(
        ipv6_slice[0].into(),
        ipv6_slice[1].into(),
        ipv6_slice[2].into(),
        ipv6_slice[3].into(),
        ipv6_slice[4].into(),
        ipv6_slice[5].into(),
        ipv6_slice[6].into(),
        ipv6_slice[7].into()
    );

    IpListenEndpoint {
        addr: Some(ip_address),
        port: x.port()
    }
}

pub fn ipvaddr_to_embassy_ipaddr(x: IpAddr) -> IpAddress {

    match x {
        IpAddr::V4(ref a) => {
            let octects = a.octets();
            IpAddress::v4(
                octects[0].into(),
                octects[1].into(),
                octects[2].into(),
                octects[3].into()
            )
        }
        IpAddr::V6(ref a) => {
            let octects = a.octets();
            IpAddress::v6(
                octects[0].into(),
                octects[1].into(),
                octects[2].into(),
                octects[3].into(),
                octects[4].into(),
                octects[5].into(),
                octects[6].into(),
                octects[7].into()
            )
        }
    }
}