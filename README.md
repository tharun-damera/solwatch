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
    
    C->>B: Enter Solana Address <br/>GET /api/accounts/{address}/status
    B->>D: Check Address State
    D->>B: Address Not Found
    B->>C: error: Address Not Found
    
    C->>B: Start Indexing <br/>SSE /api/accounts/{address}/index/sse
    
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
            
            Note over C,B: Client can now call:<br/>Get Transaction History (Signatures) <br/>GET /api/accounts/{address}/signatures
            
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
sequenceDiagram
    actor C as Client
    participant B as Backend
    participant S as Solana RPC
    participant D as Database
    
    C->>B: Refresh Account <br/>SSE /api/accounts/{address}/refresh/sse
    B->>D: Update Address State (state='syncing')
    B->>C: event: syncing
    
    B->>S: get_account(address)
    S->>B: Latest Account Data
    B->>D: Update Account
    B->>C: event: account-data
    
    B->>D: Get last slot signature
    D->>B: last_signature
    
    loop Until caught up
        B->>S: get_signatures_for_address(until=last_signature)
        S->>B: New signatures
        B->>D: Store signatures
        B->>C: event: signatures-fetched
        
        loop For each signature
            B->>S: get_transaction(signature)
            S->>B: Transaction
        end
        B->>D: Store transactions
        B->>C: event: transactions-fetched
    end
    
    B->>D: Update Address State (state='idle')
    B->>C: event: close
```

