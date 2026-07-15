/*
 *
 *    Copyright (c) 2025-2026 Project CHIP Authors
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

//! A module containing the mDNS code used in the examples

use ariel_os::{
    net,
};
use rs_matter::Matter;
use rs_matter::{crypto::Crypto, error::Error};

use embassy_net::udp::{PacketMetadata, UdpSocket};

use crate::socket_utils::socket_to_listenendpoint;
use crate::{socket_network::SocketNetwork, socket_utils::ipvaddr_to_embassy_ipaddr};

use rs_matter::transport::network::{Ipv6Addr};

use rs_matter::transport::network::mdns::builtin::{BuiltinMdns, Host};
use rs_matter::transport::network::mdns::{
    MDNS_IPV4_BROADCAST_ADDR, MDNS_SOCKET_DEFAULT_BIND_ADDR,
};

#[allow(unused)]
pub async fn run_mdns<C: Crypto>(matter: &Matter<'_>, crypto: C) -> Result<(), Error> {
    #[cfg(feature = "astro-dnssd")]
    rs_matter::transport::network::mdns::astro::AstroMdns::new()
        .run(matter)
        .await?;

    #[cfg(all(feature = "zeroconf", not(feature = "astro-dnssd")))]
    rs_matter::transport::network::mdns::zeroconf::ZeroconfMdns::new()
        .run(matter)
        .await?;

    #[cfg(all(
        feature = "resolve",
        not(any(feature = "zeroconf", feature = "astro-dnssd"))
    ))]
    rs_matter::transport::network::mdns::resolve::ResolveMdns::new(
        rs_matter::utils::zbus::Connection::system().await.unwrap(),
    )
    .run(matter)
    .await?;

    #[cfg(all(
        feature = "avahi",
        not(any(feature = "resolve", feature = "zeroconf", feature = "astro-dnssd"))
    ))]
    rs_matter::transport::network::mdns::avahi::AvahiMdns::new(
        rs_matter::utils::zbus::Connection::system().await.unwrap(),
    )
    .run(matter)
    .await?;

    #[cfg(not(any(
        feature = "avahi",
        feature = "resolve",
        feature = "zeroconf",
        feature = "astro-dnssd"
    )))]
    run_builtin_mdns(matter, crypto).await?;

    Ok(())
}

#[allow(unused)]
async fn run_builtin_mdns<C: Crypto>(matter: &Matter<'_>, crypto: C) -> Result<(), Error> {
    // NOTE:
    // When using a custom UDP stack (e.g. for `no_std` environments), replace with a UDP socket bind + multicast join for your custom UDP stack
    // The returned socket should be splittable into two halves, where each half implements `UdpSend` and `UdpReceive` respectively

    // let mut socket = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;
    // socket.set_reuse_address(true)?;
    // socket.set_only_v6(false)?;
    // socket.bind(&MDNS_SOCKET_DEFAULT_BIND_ADDR.into())?;
    // let socket = async_io::Async::<UdpSocket<'_>>::new_nonblocking(socket.into())?;

    // socket
    //     .get_ref()
    //     .join_multicast_v6(&MDNS_IPV6_BROADCAST_ADDR, interface)?;
    // socket
    //     .get_ref()
    //     .join_multicast_v4(&MDNS_IPV4_BROADCAST_ADDR, &ipv4_addr)?;
    let stack = net::network_stack().await.unwrap();
    stack.wait_config_up().await;
    stack
        .join_multicast_group(ipvaddr_to_embassy_ipaddr(MDNS_IPV4_BROADCAST_ADDR.into()))
        .expect("IPV4 Group");

    let mut rx_buffer = [0; 256];
    let mut tx_buffer = [0; 256];
    let mut rx_meta = [PacketMetadata::EMPTY; 1];
    let mut tx_meta = [PacketMetadata::EMPTY; 1];
    let mut socket_intern = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );
    // socket.set_reuse_address(true)?;
    socket_intern
        .bind(socket_to_listenendpoint(MDNS_SOCKET_DEFAULT_BIND_ADDR))
        .expect("ERROR");

    let mut socket = SocketNetwork {
        inner: &mut socket_intern,
        stack: &stack,
    };

    let ipv4address = stack
        .config_v4()
        .expect("Error due to no ipv4 addr")
        .address
        .address();

    BuiltinMdns::new()
        .run(
            &socket,
            &socket,
            &Host {
                hostname: "001122334455", //"rs-matter-demo",
                ip: ipv4address,
                ipv6: Ipv6Addr::UNSPECIFIED,
            },
            Some(ipv4address),
            None,
            matter,
            crypto,
        )
        .await
}
