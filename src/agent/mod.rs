//! Agent protocol and integration layer.
//!
//! Provides the structured protocol for AI agents to connect to,
//! inspect, and control Dewey applications.

pub mod driver;
pub mod mcp;
pub mod protocol;
pub mod rpc;
pub mod session;
#[cfg(feature = "ws-transport")]
pub mod ws_transport;

pub use driver::HeadlessDriver;
pub use mcp::McpServer;
pub use protocol::{AgentEvent, AgentRequest, AgentResponse};
pub use rpc::RpcTransport;
pub use session::AgentSession;
#[cfg(feature = "ws-transport")]
pub use ws_transport::WsTransport;
