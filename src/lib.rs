use actix::prelude::*;
use tokio::sync::mpsc;

pub mod messages;
pub use messages::*;
pub mod print;
pub use print::*;
pub mod yfetch;
pub use yfetch::*;
pub mod transform;
pub use transform::*;
pub mod ticker;
pub use ticker::*;

#[inline]
pub fn subscribe<A, M, C>(addr: Addr<A>, mut rx: mpsc::Receiver<M>)
where
    M: Message<Result = ()> + Send + 'static,
    A: Actor<Context = C> + Handler<M>,
    C: actix::dev::ToEnvelope<A, M>,
{
    actix::spawn(async move {
        while let Some(msg) = rx.recv().await {
            addr.send(msg).await.unwrap();
        }
    });
}
