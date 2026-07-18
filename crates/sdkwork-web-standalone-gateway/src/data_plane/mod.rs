mod active_health;
mod connection_limit;
mod dns;
mod error;
mod fixed_histogram;
mod handler;
mod http1_wire;
mod http2_wire;
mod io_timeout;
mod keep_alive_timeout;
mod metrics;
mod operations;
mod proxy;
mod proxy_body;
mod proxy_protocol;
mod real_ip;
mod request_admission;
mod request_body_timeout;
mod request_gate;
mod request_uri;
mod resource_pressure;
mod runtime;
mod server;
mod smooth_weighted;
mod static_files;
mod tls;
mod tunnel;
mod upstream_admission;
mod upstream_client;
mod upstream_tls;
mod watch;

pub use error::DataPlaneError;
pub use operations::DataPlaneOperationsConfig;
pub use runtime::DataPlaneReloadReport;
pub use server::{run_data_plane_until, run_data_plane_with_operations_until};
pub use watch::{
    run_data_plane_from_config_until, run_data_plane_from_config_with_operations_until,
};

use std::sync::Arc;

use self::runtime::DataPlaneRuntime;

#[derive(Clone)]
struct ListenerState {
    runtime: Arc<DataPlaneRuntime>,
    listener_id: String,
    is_tls: bool,
}
