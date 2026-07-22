use core::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use embassy_net::{IpAddress, IpEndpoint, IpListenEndpoint};

pub fn socket_to_listenendpoint(x: SocketAddr) -> IpListenEndpoint {
    let ip = match x {
        SocketAddr::V4(ref a) => {
            let ipv4_slice = a.ip().octets();
            IpAddress::v4(
                ipv4_slice[0].into(),
                ipv4_slice[1].into(),
                ipv4_slice[2].into(),
                ipv4_slice[3].into(),
            )
        },
        SocketAddr::V6(ref a) => {
            let ipv6_slice = a.ip().segments();
            IpAddress::v6(
                ipv6_slice[0].into(),
                ipv6_slice[1].into(),
                ipv6_slice[2].into(),
                ipv6_slice[3].into(),
                ipv6_slice[4].into(),
                ipv6_slice[5].into(),
                ipv6_slice[6].into(),
                ipv6_slice[7].into(),
            )
        }
    };

    IpListenEndpoint {
        addr: Some(ip),
        port: x.port(),
    }
}

pub fn socket_to_ipendpoint(x: SocketAddr) -> IpEndpoint {
        let ip = match x {
        SocketAddr::V4(ref a) => {
            let ipv4_slice = a.ip().octets();
            IpAddress::v4(
                ipv4_slice[0].into(),
                ipv4_slice[1].into(),
                ipv4_slice[2].into(),
                ipv4_slice[3].into(),
            )
        },
        SocketAddr::V6(ref a) => {
            let ipv6_slice = a.ip().segments();
            IpAddress::v6(
                ipv6_slice[0].into(),
                ipv6_slice[1].into(),
                ipv6_slice[2].into(),
                ipv6_slice[3].into(),
                ipv6_slice[4].into(),
                ipv6_slice[5].into(),
                ipv6_slice[6].into(),
                ipv6_slice[7].into(),
            )
        }
    };

    IpEndpoint {
        addr: ip,
        port: x.port(),
    }
}

pub fn ipendpoint_to_socket_address(x: IpEndpoint) -> SocketAddr {
    let port = x.port;
    match x.addr {
        IpAddress::Ipv4(ref a) => SocketAddr::new(Ipv4Addr::from_octets(a.octets()).into(), port),
        IpAddress::Ipv6(ref a) => SocketAddr::new(Ipv6Addr::from_octets(a.octets()).into(), port),
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
                octects[3].into(),
            )
        }
        IpAddr::V6(ref a) => {
            let segments = a.segments();
            IpAddress::v6(
                segments[0].into(),
                segments[1].into(),
                segments[2].into(),
                segments[3].into(),
                segments[4].into(),
                segments[5].into(),
                segments[6].into(),
                segments[7].into(),
            )
        }
    }
}
