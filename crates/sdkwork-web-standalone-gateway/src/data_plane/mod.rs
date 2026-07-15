mod connection_limit;
mod error;
mod handler;
mod proxy;
mod server;
mod static_files;

pub use error::DataPlaneError;
pub use server::run_data_plane_until;

use std::{collections::HashMap, sync::Arc};

use sdkwork_webserver_core::CompiledWebServerApp;
use tokio::sync::Semaphore;

use self::proxy::ProxyUpstream;

struct DataPlaneRuntime {
    app: Arc<CompiledWebServerApp>,
    upstreams: HashMap<String, ProxyUpstream>,
    connection_permits: Arc<Semaphore>,
}

impl DataPlaneRuntime {
    fn build(app: CompiledWebServerApp) -> Result<Arc<Self>, DataPlaneError> {
        let app = Arc::new(app);
        let upstreams = app
            .config()
            .upstreams
            .iter()
            .map(|upstream| {
                ProxyUpstream::build(upstream).map(|runtime| (upstream.id.clone(), runtime))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;
        let connection_permits = Arc::new(Semaphore::new(app.config().limits.max_connections));
        Ok(Arc::new(Self {
            app,
            upstreams,
            connection_permits,
        }))
    }
}

#[derive(Clone)]
struct ListenerState {
    runtime: Arc<DataPlaneRuntime>,
    listener_id: String,
    is_tls: bool,
}
