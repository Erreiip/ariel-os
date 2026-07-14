#![no_main]
#![no_std]

// use ariel_os::reexports::embassy_net::{IpAddress, IpListenEndpoint};
// use ariel_os::{log::*, net, reexports::embassy_net};
use ariel_os::{log::*, net};

use ariel_os_random::FastRng;

use core::net::{IpAddr, Ipv6Addr, SocketAddr, SocketAddrV6};
use core::pin::pin;

use embassy_futures::select::select4;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{IpAddress, IpListenEndpoint};

use rs_matter::crypto::{Crypto, default_crypto};
use rs_matter::dm::clusters::app::level_control::LevelControlHooks;
use rs_matter::dm::clusters::app::on_off::{self, test::TestOnOffDeviceLogic, OnOffHooks};
use rs_matter::dm::clusters::desc::{self, ClusterHandler as _};
use rs_matter::dm::clusters::groups::{self, ClusterHandler as _};
use rs_matter::dm::devices::test::{DAC_PRIVKEY, TEST_DEV_ATT, TEST_DEV_COMM, TEST_DEV_DET};
use rs_matter::dm::devices::DEV_TYPE_ON_OFF_LIGHT;
use rs_matter::dm::endpoints;
use rs_matter::dm::networks::eth::EthNetwork;
use rs_matter::dm::{Async, DataModel, Dataver, Endpoint, EpClMatcher, Node};
use rs_matter::im::{EthInteractionModelState, InteractionModel};
use rs_matter::pairing::qr::QrTextType;
use rs_matter::pairing::DiscoveryCapabilities;
use rs_matter::respond::DefaultResponder;
use rs_matter::sc::pase::MAX_COMM_WINDOW_TIMEOUT_SECS;
use rs_matter::transport::exchange::MatterBuffers;
use rs_matter::transport::MATTER_SOCKET_BIND_ADDR;
use rs_matter::utils::select::Coalesce;
use rs_matter::{clusters, devices, root_endpoint, Matter, MATTER_PORT};
use rs_matter::persist::DummyKvBlobStore;

mod mdns;
mod socket;

use socket::socket_to_listenendpoint;

#[ariel_os::task(autostart)]
async fn main() {

    let matter = Matter::new(&TEST_DEV_DET, TEST_DEV_COMM, &TEST_DEV_ATT, MATTER_PORT);

    let buffers: MatterBuffers = MatterBuffers::new();

    let state: EthInteractionModelState = EthInteractionModelState::new(EthNetwork::new_default());

    let store = DummyKvBlobStore;

    let kv = matter.kv(store);

    let mut c = CryptoRngWrapperP {
        inner: &mut ariel_os_random::fast_rng()
    };

    let crypto = default_crypto(&mut c, DAC_PRIVKEY);

    let mut rand = crypto.rand().expect("Fck");

    let on_off_handler = on_off::OnOffHandler::new_standalone(
        Dataver::new_rand(&mut rand),
        1,
        TestOnOffDeviceLogic::new(true),
    );

    let im = InteractionModel::new(
        &matter,
        &crypto,
        &buffers,
        data_model(rand, &on_off_handler),
        &kv,
        &state,
    );

    let responder = DefaultResponder::new(&im);

    let mut respond = pin!(responder.run::<4, 4>());

    let mut im_job = pin!(im.run());

    let stack = net::network_stack().await.unwrap();

    info!("waiting for interface to come up...");
    stack.wait_config_up().await;

    // Increase the buffer size if you want to send bigger packets.
    let mut rx_buffer = [0; 256];
    let mut tx_buffer = [0; 256];
    let mut rx_meta = [PacketMetadata::EMPTY; 1];
    let mut tx_meta = [PacketMetadata::EMPTY; 1];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );
    socket.bind(socket_to_listenendpoint(MATTER_SOCKET_BIND_ADDR)).expect("ARGHHH");

    let mut mdns = pin!(mdns::run_mdns(&matter, &crypto));
    let mut transport = pin!(matter.run(&crypto, &socket, &socket, &socket));

    if !matter.is_commissioned() {
        // If the device is not commissioned yet, print the QR text and code to the console
        // and enable basic commissioning

        matter.print_standard_qr_text(DiscoveryCapabilities::IP).expect("NO");
        matter.print_standard_qr_code(QrTextType::Unicode, DiscoveryCapabilities::IP).expect("NO");

        matter.open_basic_comm_window(MAX_COMM_WINDOW_TIMEOUT_SECS, &crypto, &()).expect("NO");
    }

    let all = select4(&mut transport, &mut mdns, &mut respond, &mut im_job).coalesce();

    info!("Started correctly");
    info!("x_x");

    // Run with a simple `block_on`. Any local executor would do.
    // futures_lite::future::block_on(all);

    info!("This is an error");

    loop {}
}



struct CryptoRngWrapperP<'a> {
    inner: &'a mut FastRng
}

impl rs_matter::crypto::CryptoRng for CryptoRngWrapperP<'_> {}

impl rs_matter::crypto::RngCore for CryptoRngWrapperP<'_> {
    fn next_u32(&mut self) -> u32 {
        self.inner.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.inner.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.inner.fill_bytes(dest)
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core_06::Error> {
        self.inner.fill_bytes(dest);
        Ok(())
    }
}

/// The Node meta-data describing our Matter device.
const NODE: Node<'static> = Node {
    endpoints: &[
        root_endpoint!(eth),
        Endpoint::new(
            1,
            devices!(DEV_TYPE_ON_OFF_LIGHT),
            clusters!(
                desc::DescHandler::CLUSTER,
                groups::GroupsHandler::CLUSTER,
                TestOnOffDeviceLogic::CLUSTER
            ),
        ),
    ],
};

/// The Data Model handler + meta-data for our Matter device.
/// The handler is the root endpoint 0 handler plus the on-off handler and its descriptor.
fn data_model<'a, OH: OnOffHooks, LH: LevelControlHooks>(
    mut rand: impl rs_matter::crypto::RngCore + Copy,
    on_off: &'a on_off::OnOffHandler<'a, OH, LH>,
) -> impl DataModel + 'a {
    (
        NODE,
        endpoints::EthSysHandlerBuilder::new()
            // .netif_diag(&SysNetifs)
            .build(rand)
            .chain(
                EpClMatcher::new(Some(1), Some(desc::DescHandler::CLUSTER.id)),
                Async(desc::DescHandler::new(Dataver::new_rand(&mut rand)).adapt()),
            )
            .chain(
                EpClMatcher::new(Some(1), Some(groups::GroupsHandler::CLUSTER.id)),
                Async(groups::GroupsHandler::new(Dataver::new_rand(&mut rand)).adapt()),
            )
            .chain(
                EpClMatcher::new(Some(1), Some(TestOnOffDeviceLogic::CLUSTER.id)),
                on_off::HandlerAsyncAdaptor(on_off),
            ),
    )
}