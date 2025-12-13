use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use mongodb::Database;
use solana_client::nonblocking::rpc_client::RpcClient;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub rpc: Arc<RpcClient>,
    pub address_lock: Arc<DashMap<String, Arc<Mutex<()>>>>,
}

impl AppState {
    pub fn new(db: Database, rpc: Arc<RpcClient>) -> Self {
        AppState {
            db,
            rpc,
            address_lock: Arc::new(DashMap::new()),
        }
    }

    pub fn get_address_lock(&self, address: &str) -> Arc<Mutex<()>> {
        self.address_lock
            .entry(address.to_string())
            .or_insert(Arc::new(Mutex::new(())))
            .clone()
    }
}
