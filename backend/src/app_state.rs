use dashmap::DashMap;
use std::sync::{Arc, atomic::AtomicBool};
use tokio::sync::{RwLock, broadcast};

use mongodb::Database;
use solana_client::nonblocking::rpc_client::RpcClient;
use tracing::{error, warn};

use crate::message::SyncStatus;

// A global AddressSession for each address whenever the account indexing or syncing tasks are running.
// Here the sender is of the broadcast channel which is used for subscribing
// all the receiver or clients to this channel specific to the address to receive real-time updates.
// started is an AtomicBool that is a thread-safe boolean variable to prevent data race
// in case of multiple concurrent requests try to index or refresh the same address.
#[derive(Debug)]
pub struct AddressSession {
    pub sender: broadcast::Sender<SyncStatus>,
    pub started: AtomicBool,
    pub past_events: RwLock<Vec<SyncStatus>>,
}

impl AddressSession {
    pub async fn emit_event(&self, event: SyncStatus) {
        {
            let mut events = self.past_events.write().await;
            events.push(event.clone());
        }
        if let Err(err) = self.sender.send(event) {
            error!(
                "Error occcured while sending event to channel: {}",
                err.to_string()
            );
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    // The standard mongodb database
    pub db: Database,
    // The Solana Json-Rpc Client wrapped inside an Arc to be shared across threads
    pub rpc: Arc<RpcClient>,
    // A dashmap that stores the address(String) as its key and an AddressSession (wrapped inside
    // an Arc for sharing across threads)
    // Why dashmap and not hashmap? Well, dashmap has built-in fine-grained locks for its sharded
    // map regions that allows multiple threads to write to different keys concurrently.
    // If we use hashmap and locks that is a coarse-grained locking which is harder and slower to
    // manage across threads while dashmap is built for high-performance and multithreaded systems.
    pub session: Arc<DashMap<String, Arc<AddressSession>>>,
}

impl AppState {
    pub fn new(db: Database, rpc: Arc<RpcClient>) -> Self {
        AppState {
            db,
            rpc,
            session: Arc::new(DashMap::new()),
        }
    }

    // Session creation or retrieval when indexing or refreshing an address
    // It involves adding the address into the DashMap along with the broadcast channel sender and
    // started atomic bool for handling multiple such requests
    pub fn get_or_create_session(&self, address: &str) -> Arc<AddressSession> {
        warn!("Session data: {:?}", self.session);
        self.session
            .entry(address.to_string())
            .or_insert_with(|| {
                let (sender, _) = broadcast::channel(10);
                Arc::new(AddressSession {
                    sender,
                    started: AtomicBool::new(false),
                    past_events: RwLock::new(Vec::new()),
                })
            })
            .clone()
    }

    // Once the indexing or refreshing is done
    // making sure to remove the address from the DashMap
    pub fn remove_session(&self, address: &str) -> bool {
        self.session.remove(address).is_some()
    }
}
