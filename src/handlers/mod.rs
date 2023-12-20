use crate::jupyter::response::Response;
use std::fmt::Debug;

pub mod debug;
pub mod msg_count;

// export Handlers
pub use debug::DebugHandler;
pub use msg_count::MessageCountHandler;

#[async_trait::async_trait]
pub trait Handler: Debug + Send + Sync {
    async fn handle(&self, msg: &Response);
}
