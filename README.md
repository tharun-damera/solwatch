# Solwatch 🛰️ (Still in Development)
A minimal and fast lazy indexer for the Solana blockchain.

## Technologies
- **Backend:** Rust (Axum, Tokio)
- **Frontend:** React
- **Database:** MongoDB
- **Deployment:** Docker Compose

## Getting Started
1. Clone the repository
   ```bash
   git clone https://github.com/tharun-damera/solwatch.git
   ```
2. Go to the project directory
   ```bash
   cd solwatch
   ```
3. Run the services using docker compose in detached mode
   ```bash
   docker compose up -d
   ```
5. Check the frontend running [here](http://localhost:8000/)
6. To shutdown the services run
   ```bash
   docker compose down
   ```

# Architecture

## System Flow Overview
```mermaid
sequenceDiagram
    actor C as Client
    participant B as Backend
    participant S as Solana RPC
    participant D as Database
    
    C->>B: Enter Solana Address
    B->>D: Check Indexing State
    
    alt Not Indexed
        Note over C,D: See: Initial Indexing Flow
        C->>B: SSE: Index Account
        B->>S: Fetch Account & Transactions
        B->>D: Store Data
        B->>C: SSE: Progress Updates
    
    else Already Indexed
        Note over C,D: See: Cached Data Flow
        C->>B: Get Account Data API
        B->>D: Fetch from DB
        B->>C: Return Data
        
        opt Refresh
            Note over C,D: See: Refresh Flow
            C->>B: SSE: Refresh Account
            B->>S: Fetch Latest Data
            B->>D: Update DB
            B->>C: SSE: Progress Updates
        end
    end
```
The system handles three main scenarios:
1. **Initial Indexing** - When an address is queried for the first time
2. **Cached Data Access** - When data already exists in the database
3. **Refresh/Sync** - When updating an indexed address with latest data

## Detailed Flows

### 1. Initial Indexing Flow
When a new address is indexed, the system:
- Validates the address
- Fetches account data from Solana RPC
- Stores the account in the database
- Fetches the latest 20 transactions immediately for quick display
- Continues fetching remaining transactions in batches of 1000
- Sends real-time progress updates via SSE
```mermaid
sequenceDiagram
    actor C as Client
    participant B as Backend
    participant S as Solana RPC
    participant D as Database
    
    C->>B: GET /api/accounts/{address}/status
    B->>D: Check Address State
    D->>B: Address Not Found
    B->>C: error: Address Not Found
    
    C->>B: SSE /api/accounts/{address}/index/sse
    
    alt Invalid Address
        B->>C: error: Invalid address
    else Valid Address
        B->>S: get_account(address)
        
        alt Account Not Found
            S->>B: Error
            B->>C: error: Account not found
        
        else Account Found
            S->>B: Account data
            B->>D: Store Address State (state='indexing')
            B->>C: event: indexing
            B->>D: Insert Account data
            B->>C: event: account-data
            
            B->>S: get_signatures_for_address(limit=20)
            S->>B: 20 signatures
            B->>D: Store signatures
            B->>C: event: signatures-fetched
            
            Note over C,B: Client can now call:<br/>GET /api/accounts/{address}/signatures
            
            loop For each signature
                B->>S: get_transaction(signature)
                S->>B: Transaction
            end
            B->>D: Store 20 transactions
            B->>C: event: transactions-fetched
            
            loop Batches of 1000
                B->>S: get_signatures_for_address(limit=1000)
                S->>B: Signatures
                B->>D: Store signatures
                B->>C: event: signatures-fetched
                
                loop For each batch
                    B->>S: get_transaction(signature)
                    S->>B: Transaction
                end
                B->>D: Store transactions
                B->>C: event: transactions-fetched
            end
            
            B->>D: Update Address State (state='idle')
            B->>C: event: close
        end
    end
```

### 2. Cached Data Access
When accessing already-indexed data:
- All data is served directly from MongoDB
- No RPC calls needed
- Fast response times
```mermaid
sequenceDiagram
    actor C as Client
    participant B as Backend
    participant D as Database
    
    C->>B: Enter Solana Address <br/>GET /api/accounts/{address}/status
    B->>D: Check Address State
    D->>B: { state: "idle" }
    B->>C: Already indexed
    
    C->>B: Get Indexer Stats <br/>GET /api/accounts/{address}/indexer/stats
    B->>D: Get state, signatures, transactions
    D->>B: Stats
    B->>C: { state: "idle", signatures: 125, transactions: 125 }
    
    C->>B: Get Account Data <br/>GET /api/accounts/{address}
    B->>D: Get Account
    D->>B: Account
    B->>C: { lamports, owner, ... }
    
    C->>B: Get Transaction History (Signatures) <br/>GET /api/accounts/{address}/signatures?skip=0&limit=20
    B->>D: Get 20 Signatures
    D->>B: Signatures
    B->>C: [{ signature, slot, ... }]
    
    opt Get Full Transaction
        C->>B: Get Full Transaction for given signature <br/>GET /api/accounts/{address}/transactions/{signature}
        B->>D: Get specific transaction
        D->>B: Transaction
        B->>C: Full transaction data
    end
```

### 3. Refresh Flow
When refreshing an indexed address:
- Fetches only new data since last sync
- Updates account information
- Fetches new transactions incrementally
```mermaid

```


## Sequence Diagram
```mermaid
sequenceDiagram
    actor C as Client
    participant B as Backend
    participant S as Solana RPC
    participant D as Database

    C->>B: Enter Solana Address
    B->>D: Check The Address Indexing State

    alt Address Not Found Or Indexing
        C->>B: Index Account SSE
        alt Invalid Address
            B->>C: Invalid Address
        else Valid Address
            B->>S: Fetch Account Data
            alt Account Not Found
                S->>B: Account Not Found
                B->>C: Account Not Found
            else Account Found
                S->>B: Account Data
                B->>D: Store Address State as Indexing
                B->>D: Store Account Data
                B->>C: Send Account Data
                B->>S: Fetch Latest 20 Transaction Signatures
                S->>B: Latest 20 Signatures
                B->>D: Store the 20 Signatures
                B->>C: Send Signature Count
                C->>B: Get Signatures API
                B->>D: Fetch Signatures
                B->>C: Signatures
                loop for each 20 Signatures
                    B->>S: Fetch Transaction
                    S->>B: Transaction
                end
                B->>D: Store 20 Transactions
                B->>C: Send Transaction Count
                loop For each batch of 1000
                    B->>S: Fetch the next 1000 Signatures
                    S->>B: Signatures
                    B->>D: Store Signatures
                    B->>C: Send Signature Count
                    loop for each 1000 Signatures
                        B->>S: Fetch Transaction
                        S->>B: Transaction
                    end
                    B->>D: Store 1000 Transactions
                    B->>C: Send Transaction Count
                end
                B->>D: Update Address State as Idle
            end
        end
    else Idle or Syncing
        C->>B: Get Indexer Stats API
        B->>D: Fetch Indexer Stats
        B->>C: Indexer Stats
        C->>B: Get Account Data API
        B->>D: Fetch Account Data
        B->>C: Account Data
        C->>B: Get Transaction Signatures API
        B->>D: Fetch Transaction Signatures
        B->>C: Transaction Signatures
        C->>B: Get Transaction API
        B->>D: Fetch Transaction
        B->>C: Transaction

        C->>B: Refresh Account SSE
        B->>D: Update Address State as Syncing
        B->>S: Fetch Latest Account Data
        S->>B: Account Data
        B->>D: Update Account Data
        B->>C: Latest Account Data
        loop For each batch of 1000
            B->>S: Fetch the next 1000 Signatures
            S->>B: Signatures
            B->>D: Store Signatures
            B->>C: Send Signature Count
            loop for each 1000 Signatures
                B->>S: Fetch Transaction
                S->>B: Transaction
            end
            B->>D: Store 1000 Transactions
            B->>C: Send Transaction Count
        end
        B->>D: Update Address State as Idle
    end
```
