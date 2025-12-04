// SyncStatus enum is used in SSE API communication
pub enum SyncStatus {
    Started,
    AccountData(String),
    TransactionSignatures(String),
    TransactionDetails(String),
    Completed,
    Error(String),
}
