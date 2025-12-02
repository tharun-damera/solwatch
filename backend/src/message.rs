use serde::Serialize;

// SyncStatus enum is used in SSE API communication
#[derive(Debug, Serialize)]
pub enum SyncStatus {
    Started,
    AccountData { data: String },
    TransactionSignatures { fetched: u64 },
    TransactionDetails { fetched: u64 },
    Completed,
    Error { message: String },
}
