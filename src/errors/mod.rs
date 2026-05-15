use thiserror::Error;

/// Errors that can occur during project scanning.
#[derive(Error, Debug)]
pub enum ScanError {
    #[error("Failed to scan directory {path}: {details}")]
    DirectoryScan { path: String, details: String },

    #[error("Failed to detect project type: {0}")]
    ProjectTypeDetection(String),

    #[error("Failed to read file {path}: {details}")]
    FileRead { path: String, details: String },

    #[error("No supported project found at {0}")]
    NoSupportedProject(String),

    #[error("Scan depth exceeded maximum of {max} (current: {current})")]
    DepthExceeded { max: usize, current: usize },

    #[error("I/O error during scan: {0}")]
    Io(#[from] std::io::Error),
}

/// Errors that can occur during persistence operations.
#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Query execution failed: {query} — {details}")]
    QueryFailed { query: String, details: String },

    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    #[error("Schema version mismatch: expected {expected}, got {actual}")]
    SchemaVersionMismatch { expected: i32, actual: i32 },

    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(String),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

/// Errors that can occur during knowledge graph operations.
#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Failed to build knowledge graph: {0}")]
    BuildFailed(String),

    #[error("Module not found: {0}")]
    ModuleNotFound(String),

    #[error("Relationship not found between {src} and {tgt}")]
    RelationshipNotFound { src: String, tgt: String },

    #[error("Duplicate node detected: {0}")]
    DuplicateNode(String),

    #[error("Graph integrity check failed: {0}")]
    IntegrityCheckFailed(String),
}

/// Errors that can occur during parsing operations.
#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Failed to parse Cargo.toml at {path}: {details}")]
    CargoToml { path: String, details: String },

    #[error("Failed to parse package.json at {path}: {details}")]
    PackageJson { path: String, details: String },

    #[error("Failed to parse TOML: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),
}

/// Errors that can occur during configuration loading.
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to load config from {path}: {details}")]
    LoadFailed { path: String, details: String },

    #[error("Invalid configuration value for {key}: {value}")]
    InvalidValue { key: String, value: String },

    #[error("Missing required configuration key: {0}")]
    MissingKey(String),

    #[error("Failed to serialize config: {0}")]
    SerializeFailed(String),

    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
}

/// Errors that can occur during caching operations.
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Cache miss for key: {0}")]
    CacheMiss(String),

    #[error("Failed to read cache entry {key}: {details}")]
    ReadFailed { key: String, details: String },

    #[error("Failed to write cache entry {key}: {details}")]
    WriteFailed { key: String, details: String },

    #[error("Cache directory creation failed: {0}")]
    DirectoryCreation(String),

    #[error("Cache integrity check failed for {key}: expected hash {expected}, got {actual}")]
    IntegrityMismatch { key: String, expected: String, actual: String },
}

/// A unified error type that wraps all ChronoDrake-specific errors.
#[derive(Error, Debug)]
pub enum ChronoDrakeError {
    #[error("Scan error: {0}")]
    Scan(#[from] ScanError),

    #[error("Persistence error: {0}")]
    Persistence(#[from] PersistenceError),

    #[error("Graph error: {0}")]
    Graph(#[from] GraphError),

    #[error("Parser error: {0}")]
    Parser(#[from] ParserError),

    #[error("Config error: {0}")]
    Config(#[from] ConfigError),

    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("General error: {0}")]
    General(String),
}

impl From<String> for ChronoDrakeError {
    fn from(s: String) -> Self {
        ChronoDrakeError::General(s)
    }
}

impl From<&str> for ChronoDrakeError {
    fn from(s: &str) -> Self {
        ChronoDrakeError::General(s.to_string())
    }
}
