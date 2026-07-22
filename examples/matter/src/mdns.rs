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
    log::info,
    net,
};
use embassy_net::{IpAddress, IpListenEndpoint};
use rs_matter::Matter;
use rs_matter::{crypto::Crypto, error::Error};

use embassy_net::udp::{PacketMetadata, UdpSocket};

use crate::socket_utils::socket_to_listenendpoint;
use crate::{socket_network::SocketNetwork, socket_utils::ipvaddr_to_embassy_ipaddr};

use rs_matter::transport::network::{Ipv6Addr};

use rs_matter::transport::network::mdns::builtin::{BuiltinMdns, Host};
use rs_matter::transport::network::mdns::{
    MDNS_IPV4_BROADCAST_ADDR, MDNS_PORT, MDNS_SOCKET_DEFAULT_BIND_ADDR, MDNS_IPV6_BROADCAST_ADDR
};

#[allow(unused)]
pub async fn run_mdns<C: Crypto>(matter: &Matter<'_>, crypto: C) -> Result<(), Error> {
    run_builtin_mdns(matter, crypto).await?;

    Ok(())
}

#[allow(unused)]
async fn run_builtin_mdns<C: Crypto>(matter: &Matter<'_>, crypto: C) -> Result<(), Error> {
    
    info!("MDNS start discovering");

    let stack = net::network_stack().await.unwrap();
    stack.wait_config_up().await;
    stack
        .join_multicast_group(ipvaddr_to_embassy_ipaddr(MDNS_IPV4_BROADCAST_ADDR.into()))
        .expect("IPV4 Group");

    stack
        .join_multicast_group(ipvaddr_to_embassy_ipaddr(MDNS_IPV6_BROADCAST_ADDR.into()))
        .expect("IPV6 Group");

    const RX_SIZE: usize = rs_matter::transport::MAX_RX_PAYLOAD_SIZE;
    const TX_SIZE: usize = rs_matter::transport::MAX_TX_PAYLOAD_SIZE;

    let mut rx_buffer = [0; RX_SIZE];
    let mut tx_buffer = [0; TX_SIZE];
    let mut rx_meta = [PacketMetadata::EMPTY; 1];
    let mut tx_meta = [PacketMetadata::EMPTY; 1];
    let mut socket_intern = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    let endpoint = IpListenEndpoint {
        addr: None,
        port: MDNS_PORT,
    };

    socket_intern
        .bind(endpoint)
        .expect("ERROR");

    let socket = SocketNetwork {
        inner: &mut &socket_intern,
        stack: &stack,
    };

    let ipv4address = stack
        .config_v4()
        .expect("Error due to no ipv4 addr")
        .address
        .address();

    let ipv6address = stack
        .config_v6()
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
                ipv6: ipv6address,
            },
            Some(ipv4address),
            None,
            matter,
            crypto,
        )
        .await
}
