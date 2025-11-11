use serde::Serialize;

use crate::models::{Account, TransactionSignature};

// Enum that is used in Websocket communication messages
// Server sends the messages to client based on the IndexingMessage enum variants
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum IndexingMessage<'a> {
    Started { address: &'a str },
    AccountData(Account),
    TransactionSignatures { data: &'a [TransactionSignature] },
    Completed { address: &'a str },
    Error { address: &'a str, message: &'a str },
}
