
use rs_matter::{crypto::Crypto, dm::{DataModel, clusters::net_comm, networks::wireless::NoopWirelessNetCtl}, error::Error, im::{IMBuffer, InteractionModel, events::DEFAULT_MAX_EVENTS_BUF_SIZE, subscriptions::DEFAULT_MAX_SUBSCRIPTIONS}, persist::KvBlobStoreAccess, respond::{BusyExchangeHandler, DefaultExchangeHandler, Responder}, utils::{select, storage::pooled::Buffers}};

use core::pin::pin;

use rs_matter::utils::select::Coalesce;
use embassy_futures::select::select;

const RESPOND_BUSY_MS: u32 = 5000;

pub struct CustomResponder<
    'd,
    'a,
    C,
    B,
    T,
    K,
    N,
    NC = NoopWirelessNetCtl,
    const NS: usize = DEFAULT_MAX_SUBSCRIPTIONS,
    const NE: usize = DEFAULT_MAX_EVENTS_BUF_SIZE,
> where
    B: Buffers<IMBuffer>,
{
    responder: Responder<'a, DefaultExchangeHandler<'d, 'a, C, B, T, K, N, NC, NS, NE>>,
    busy_responder: Responder<'a, BusyExchangeHandler>,
}

impl<'d, 'a, C, B, T, K, N, NC, const NS: usize, const NE: usize>
    CustomResponder<'d, 'a, C, B, T, K, N, NC, NS, NE>
where
    C: Crypto,
    B: Buffers<IMBuffer>,
    T: DataModel,
    K: KvBlobStoreAccess,
    N: net_comm::Networks,
{
    /// Creates the responder composition.
    #[inline(always)]
    pub const fn new(data_model: &'d InteractionModel<'a, C, B, T, K, N, NC, NS, NE>) -> Self {
        Self {
            responder: Responder::new_default(data_model),
            busy_responder: Responder::new_busy(data_model.matter(), RESPOND_BUSY_MS),
        }
    }

    /// Run the responder.
    pub async fn run<const A: usize, const O: usize>(&self) -> Result<(), Error> {
        let mut actual = pin!(self.responder.run::<A>());
        let mut busy = pin!(self.busy_responder.run::<O>());

        select(&mut actual, &mut busy).coalesce().await
    }
}