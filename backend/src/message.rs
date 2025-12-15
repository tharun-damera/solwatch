// SyncStatus enum is used in SSE API communication
#[derive(Clone, Debug)]
pub enum SyncStatus {
    Indexing,
    Syncing,
    AccountData(String),
    TransactionSignatures(String),
    TransactionDetails(String),
    Completed,
    Error(String),
}
