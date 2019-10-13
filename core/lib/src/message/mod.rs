mod broker;
mod message;

pub use self::message::Message;
pub(crate) use self::broker::Broker;

pub type Receiver = futures_channel::mpsc::UnboundedReceiver<Message>;
pub type Sender = futures_channel::mpsc::UnboundedSender<Message>;

pub fn channel() -> (Sender, Receiver) {
    futures_channel::mpsc::unbounded()
}