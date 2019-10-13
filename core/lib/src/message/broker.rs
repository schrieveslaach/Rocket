use crate::message::{Message, Receiver};
use futures_core::future::Future;
use futures_util::future::FutureExt;
use std::sync::Arc;
use crate::http::hyper;
use tokio::io::AsyncWriteExt;
use futures_core::{Poll, Stream};
use futures_core::task::Context;
use std::pin::Pin;
// TODO use futures_core::task::__internal::AtomicWaker;
use futures_util::lock::Mutex;

pub(crate) struct Broker {
    upgrades: Arc<Mutex<Vec<Arc<Mutex<hyper::Upgraded>>>>>,
    receivers: Vec<Receiver>,
    // TODO waker: AtomicWaker,
}

impl Broker {
    pub fn new(upgrades: Arc<Mutex<Vec<Arc<Mutex<hyper::Upgraded>>>>>) -> Self {
        Broker {
            upgrades,
            receivers: Vec::new(),
        }
    }

    pub fn empty() -> Self {
        Broker {
            upgrades: Arc::new(Mutex::new(Vec::new())),
            receivers: Vec::new(),
        }
    }

    pub fn extend_with(&mut self, receivers: Vec<Receiver>) {
        self.receivers.extend(receivers);
    }

    fn send_message(upgrades: Arc<Mutex<Vec<Arc<Mutex<hyper::Upgraded>>>>>, msg: Message) -> impl Future<Output = ()> {

        async move {
            for upgraded in upgrades.lock().await.iter_mut() {
                let upgraded = upgraded.clone();

                let payload = format!("{:?}", msg);
                tokio::spawn(async move {
                    let mut upgraded = upgraded.lock().await;

                    // TODO: handle error correctly
                    upgraded.write_all(payload.as_bytes()).map(|_| ()).await;
                });
            }
        }
    }
}

impl Stream for Broker {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let broker = self.get_mut();

        // TODO: ensure round robbing
        for receiver in broker.receivers.iter_mut() {

            let msg = match receiver.try_next() {
                Ok(msg) => msg,
                Err(err) => {
                    trace!("{}", err);
                    continue;
                }
            };

            if let Some(msg) = msg {
                tokio::spawn(Broker::send_message(broker.upgrades.clone(), msg));
                return Poll::Ready(Some(()));
            }
        }

        // TODO: the stream never is pending and therefore eats a full cpu core... :see_no_evil:
        Poll::Ready(Some(()))
    }
}