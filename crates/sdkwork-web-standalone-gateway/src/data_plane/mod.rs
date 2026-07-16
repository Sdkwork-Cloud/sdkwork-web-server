mod connection_limit;
mod dns;
mod error;
mod handler;
mod http1_wire;
mod http2_wire;
mod io_timeout;
mod keep_alive_timeout;
mod proxy;
mod proxy_body;
mod request_admission;
mod request_body_timeout;
mod request_uri;
mod runtime;
mod server;
mod static_files;
mod tls;
mod watch;

pub use error::DataPlaneError;
pub use runtime::DataPlaneReloadReport;
pub use server::run_data_plane_until;
pub use watch::run_data_plane_from_config_until;

use std::sync::Arc;

use self::runtime::DataPlaneRuntime;

#[derive(Clone)]
struct ListenerState {
    runtime: Arc<DataPlaneRuntime>,
    listener_id: String,
    is_tls: bool,
}
