# Marble Architecture Diagram

## Component Interaction Diagram

```mermaid
graph TD
    subgraph Clients
        ObsidianClient[Obsidian Client]
        WebClient[Web Browser]
    end
    
    subgraph MarbleWebDAV[WebDAV Server]
        WebDAVRaw[Raw WebDAV Endpoint]
        WebDAVProcessed[Processed WebDAV Endpoint]
        Auth[Authentication]
    end
    
    subgraph Storage
        RawStorage[Raw Storage Backend]
        ProcessedStorage[Processed Storage Backend]
        S3[(S3 Content Storage)]
        DB[(PostgreSQL Metadata)]
    end
    
    subgraph ProcessingPipeline
        WriteProcessor[Write Processor]
        ReadProcessor[Read Processor]
        ProcessingQueue[Processing Queue]
    end
    
    subgraph ReadSide
        HTMLGenerator[HTML Generator]
        TemplateEngine[Template Engine]
    end
    
    % Client connections
    ObsidianClient -->|WebDAV Sync| WebDAVRaw
    WebClient -->|Read Published Content| WebDAVProcessed
    WebClient -->|View Website| HTMLGenerator
    
    % WebDAV Server connections
    WebDAVRaw -->|Authenticate| Auth
    WebDAVProcessed -->|No Auth Required| ProcessedStorage
    Auth -->|Verify| DB
    
    % Storage connections
    WebDAVRaw -->|Read/Write| RawStorage
    RawStorage -->|Store Content| S3
    RawStorage -->|Store Metadata| DB
    ProcessedStorage -->|Read Content| S3
    ProcessedStorage -->|Read Metadata| DB
    
    % Processing connections
    RawStorage -->|Notify Changes| ProcessingQueue
    ProcessingQueue -->|Process| WriteProcessor
    WriteProcessor -->|Extract Metadata| DB
    WriteProcessor -->|Trigger| ReadProcessor
    ReadProcessor -->|Generate| ProcessedStorage
    
    % Read Side
    WebDAVProcessed -->|Fetch Content| HTMLGenerator
    HTMLGenerator -->|Use| TemplateEngine
    
    classDef server fill:#f9f,stroke:#333,stroke-width:2px;
    classDef storage fill:#bbf,stroke:#333,stroke-width:2px;
    classDef processor fill:#bfb,stroke:#333,stroke-width:2px;
    classDef db fill:#ff9,stroke:#333,stroke-width:2px;
    classDef client fill:#ddd,stroke:#333,stroke-width:1px;
    
    class WebDAVRaw,WebDAVProcessed,Auth server;
    class RawStorage,ProcessedStorage storage;
    class WriteProcessor,ReadProcessor,ProcessingQueue processor;
    class S3,DB db;
    class ObsidianClient,WebClient,HTMLGenerator,TemplateEngine client;
```

## Data Flow Diagram

```mermaid
sequenceDiagram
    participant Client as Obsidian Client
    participant WebDAV as WebDAV Server
    participant Raw as Raw Storage
    participant S3 as S3 Storage
    participant DB as PostgreSQL
    participant WP as Write Processor
    participant RP as Read Processor
    participant Processed as Processed Storage
    
    Client->>WebDAV: Authenticate (username/password)
    WebDAV->>DB: Verify credentials
    DB-->>WebDAV: Authentication result
    
    Client->>WebDAV: PUT file.md
    WebDAV->>Raw: Store file
    Raw->>S3: Store content with hash
    Raw->>DB: Update metadata
    
    Raw->>WP: Notify change
    WP->>DB: Extract references & metadata
    WP->>DB: Queue for processing
    
    Note over WP,RP: Wait 5 seconds for more changes
    
    DB->>RP: Process queued items
    RP->>DB: Fetch published content
    RP->>S3: Fetch content by hash
    RP->>S3: Store processed content
    RP->>DB: Update published content metadata
    
    Client->>WebDAV: GET /processed/{username}/permalink/index.md
    WebDAV->>Processed: Fetch content
    Processed->>DB: Lookup path
    Processed->>S3: Get content by hash
    Processed-->>WebDAV: Return processed content
    WebDAV-->>Client: Deliver content
```
