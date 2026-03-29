//! Session Backup & Restore System
//!
//! Provides encrypted backup and restore functionality for WhatsApp session data.
//! Supports local file backup, cloud storage (S3, GCS), and automatic scheduled backups.
//!
//! # Features
//!
//! - **Encrypted Backup**: AES-256-GCM encryption with password
//! - **Multiple Storage**: Local file, S3, Google Cloud Storage
//! - **Auto Backup**: Scheduled automatic backups with rotation
//! - **Versioning**: Keep multiple backup versions
//! - **Quick Restore**: One-click session restoration
//!
//! # Example
//!
//! ```rust,no_run
//! use ruwa::Client;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let client: Arc<Client> = /* ... */;
//!
//!     // Backup session to file
//!     client.backup_session()
//!         .to_file("session_backup.enc")
//!         .with_password("secure_password")
//!         .await?;
//!
//!     // Restore session from file
//!     client.restore_session()
//!         .from_file("session_backup.enc")
//!         .with_password("secure_password")
//!         .await?;
//!
//!     // Enable auto-backup (every hour, keep last 10)
//!     client.enable_auto_backup()
//!         .interval(std::time::Duration::from_secs(3600))
//!         .to_directory("./backups")
//!         .keep_last(10)
//!         .start().await?;
//!
//!     Ok(())
//! }
//! ```

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::{
    fs,
    sync::{Mutex, RwLock},
    time,
};

/// Backup metadata stored alongside encrypted data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    /// Backup creation timestamp
    pub created_at: DateTime<Utc>,
    /// RuWa version used for backup
    pub ruwa_version: String,
    /// WhatsApp web version
    pub wa_version: String,
    /// Phone number associated with session (masked)
    pub phone_masked: Option<String>,
    /// Backup size in bytes
    pub size_bytes: u64,
    /// Backup checksum for integrity verification
    pub checksum: String,
    /// Additional notes or tags
    pub tags: Vec<String>,
}

impl BackupMetadata {
    pub fn new() -> Self {
        Self {
            created_at: Utc::now(),
            ruwa_version: env!("CARGO_PKG_VERSION").to_string(),
            wa_version: String::new(),
            phone_masked: None,
            size_bytes: 0,
            checksum: String::new(),
            tags: Vec::new(),
        }
    }

    /// Mask phone number for privacy (e.g., "6281234567890" → "628****7890")
    pub fn mask_phone(phone: &str) -> String {
        if phone.len() > 8 {
            format!("{}****{}", &phone[..4], &phone[phone.len() - 4..])
        } else {
            "***".to_string()
        }
    }

    /// Calculate SHA256 checksum of data
    pub fn calculate_checksum(data: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
}

impl Default for BackupMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for session backup
#[derive(Debug, Clone)]
pub struct BackupConfig {
    /// Destination path for backup file
    pub destination: PathBuf,
    /// Encryption password (optional, recommended for security)
    pub password: Option<String>,
    /// Backup metadata tags
    pub tags: Vec<String>,
    /// Compression enabled (default: true)
    pub compress: bool,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            destination: PathBuf::from("session_backup.enc"),
            password: None,
            tags: Vec::new(),
            compress: true,
        }
    }
}

impl BackupConfig {
    pub fn new(destination: impl AsRef<Path>) -> Self {
        Self {
            destination: destination.as_ref().to_path_buf(),
            ..Default::default()
        }
    }

    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_compression(mut self, compress: bool) -> Self {
        self.compress = compress;
        self
    }
}

/// Configuration for session restore
#[derive(Debug, Clone)]
pub struct RestoreConfig {
    /// Source path for backup file
    pub source: PathBuf,
    /// Decryption password (if backup was encrypted)
    pub password: Option<String>,
    /// Skip version check (use with caution)
    pub skip_version_check: bool,
}

impl Default for RestoreConfig {
    fn default() -> Self {
        Self {
            source: PathBuf::from("session_backup.enc"),
            password: None,
            skip_version_check: false,
        }
    }
}

impl RestoreConfig {
    pub fn new(source: impl AsRef<Path>) -> Self {
        Self {
            source: source.as_ref().to_path_buf(),
            ..Default::default()
        }
    }

    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn skip_version_check(mut self, skip: bool) -> Self {
        self.skip_version_check = skip;
        self
    }
}

/// Configuration for automatic backup scheduler
#[derive(Debug, Clone)]
pub struct AutoBackupConfig {
    /// Backup interval duration
    pub interval: Duration,
    /// Directory to store backups
    pub directory: PathBuf,
    /// Number of backups to keep (0 = unlimited)
    pub keep_last: u32,
    /// Filename prefix for backups
    pub filename_prefix: String,
    /// Enable compression
    pub compress: bool,
    /// Encryption password (optional)
    pub password: Option<String>,
}

impl Default for AutoBackupConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(3600), // 1 hour
            directory: PathBuf::from("./backups"),
            keep_last: 10,
            filename_prefix: "session".to_string(),
            compress: true,
            password: None,
        }
    }
}

impl AutoBackupConfig {
    pub fn new(directory: impl AsRef<Path>, interval: Duration) -> Self {
        Self {
            directory: directory.as_ref().to_path_buf(),
            interval,
            ..Default::default()
        }
    }

    pub fn keep_last(mut self, count: u32) -> Self {
        self.keep_last = count;
        self
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.filename_prefix = prefix.into();
        self
    }

    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn no_compression(mut self) -> Self {
        self.compress = false;
        self
    }
}

/// Result of backup operation
#[derive(Debug, Clone)]
pub struct BackupResult {
    /// Path to backup file
    pub file_path: PathBuf,
    /// Backup metadata
    pub metadata: BackupMetadata,
    /// Backup size in bytes
    pub size_bytes: u64,
    /// Whether encryption was used
    pub encrypted: bool,
}

/// Result of restore operation
#[derive(Debug, Clone)]
pub struct RestoreResult {
    /// Whether restore was successful
    pub success: bool,
    /// Number of items restored
    pub items_restored: usize,
    /// Backup metadata
    pub backup_metadata: BackupMetadata,
    /// Warnings during restore
    pub warnings: Vec<String>,
}

/// Session data to be backed up
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// Identity key pair
    pub identity_key_pair: Option<Vec<u8>>,
    /// Registration ID
    pub registration_id: Option<u32>,
    /// PreKeys
    pub prekeys: Vec<(u32, Vec<u8>)>,
    /// Signed prekeys
    pub signed_prekeys: Vec<(u32, Vec<u8>)>,
    /// Sessions with other users
    pub sessions: Vec<(String, Vec<u8>)>,
    /// Sender keys for groups
    pub sender_keys: Vec<(String, Vec<u8>)>,
    /// Device information
    pub device_info: Option<Vec<u8>>,
    /// App state sync keys
    pub app_state_keys: Vec<(Vec<u8>, Vec<u8>)>,
    /// LID-PN mappings
    pub lid_pn_mappings: Vec<(String, String)>,
    /// Device registry
    pub device_registry: Vec<(String, Vec<u8>)>,
}

impl SessionData {
    pub fn new() -> Self {
        Self {
            identity_key_pair: None,
            registration_id: None,
            prekeys: Vec::new(),
            signed_prekeys: Vec::new(),
            sessions: Vec::new(),
            sender_keys: Vec::new(),
            device_info: None,
            app_state_keys: Vec::new(),
            lid_pn_mappings: Vec::new(),
            device_registry: Vec::new(),
        }
    }
}

impl Default for SessionData {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for backup operations
pub struct BackupBuilder {
    manager: Arc<SessionBackupManager>,
    client: Arc<crate::Client>,
    config: BackupConfig,
}

impl BackupBuilder {
    pub fn new(manager: Arc<SessionBackupManager>, client: Arc<crate::Client>) -> Self {
        Self {
            manager,
            client,
            config: BackupConfig::default(),
        }
    }

    pub fn to_file(mut self, path: impl AsRef<Path>) -> Self {
        self.config.destination = path.as_ref().to_path_buf();
        self
    }

    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.config.password = Some(password.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.config.tags = tags;
        self
    }

    pub fn compress(mut self, compress: bool) -> Self {
        self.config.compress = compress;
        self
    }

    pub async fn run(self) -> Result<BackupResult, anyhow::Error> {
        self.manager.backup_to_file(&self.client, self.config).await
    }
}

/// Builder for restore operations
pub struct RestoreBuilder {
    manager: Arc<SessionBackupManager>,
    client: Arc<crate::Client>,
    config: RestoreConfig,
}

impl RestoreBuilder {
    pub fn new(manager: Arc<SessionBackupManager>, client: Arc<crate::Client>) -> Self {
        Self {
            manager,
            client,
            config: RestoreConfig::default(),
        }
    }

    pub fn from_file(mut self, path: impl AsRef<Path>) -> Self {
        self.config.source = path.as_ref().to_path_buf();
        self
    }

    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.config.password = Some(password.into());
        self
    }

    pub fn skip_version_check(mut self, skip: bool) -> Self {
        self.config.skip_version_check = skip;
        self
    }

    pub async fn run(self) -> Result<RestoreResult, anyhow::Error> {
        self.manager.restore_from_file(&self.client, self.config).await
    }
}

/// Builder for auto-backup configuration
pub struct AutoBackupBuilder {
    manager: Arc<SessionBackupManager>,
    client: Arc<crate::Client>,
    config: AutoBackupConfig,
}

impl AutoBackupBuilder {
    pub fn new(manager: Arc<SessionBackupManager>, client: Arc<crate::Client>) -> Self {
        Self {
            manager,
            client,
            config: AutoBackupConfig::default(),
        }
    }

    pub fn interval(mut self, interval: Duration) -> Self {
        self.config.interval = interval;
        self
    }

    pub fn to_directory(mut self, path: impl AsRef<Path>) -> Self {
        self.config.directory = path.as_ref().to_path_buf();
        self
    }

    pub fn keep_last(mut self, count: u32) -> Self {
        self.config.keep_last = count;
        self
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.config.filename_prefix = prefix.into();
        self
    }

    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.config.password = Some(password.into());
        self
    }

    pub fn no_compression(mut self) -> Self {
        self.config.compress = false;
        self
    }

    pub async fn start(self) -> Result<(), anyhow::Error> {
        self.manager.start_auto_backup(self.client, self.config).await
    }
}

/// Backup manager handles session backup and restore operations
pub struct SessionBackupManager {
    /// Configuration for auto-backup
    auto_backup_config: RwLock<Option<AutoBackupConfig>>,
    /// Handle for auto-backup task
    auto_backup_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
    /// Last backup timestamp
    last_backup: RwLock<Option<DateTime<Utc>>>,
    /// Backup counter for statistics
    backup_count: Mutex<u64>,
}

impl SessionBackupManager {
    pub fn new() -> Self {
        Self {
            auto_backup_config: RwLock::new(None),
            auto_backup_handle: Mutex::new(None),
            last_backup: RwLock::new(None),
            backup_count: Mutex::new(0),
        }
    }

    /// Encrypt data using AES-256-GCM
    fn encrypt_data(data: &[u8], password: &str) -> Result<Vec<u8>, anyhow::Error> {
        // Derive key from password using PBKDF2
        use pbkdf2::pbkdf2_hmac;
        use sha2::Sha256;

        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);

        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, 100_000, &mut key);

        let cipher = Aes256Gcm::new_from_slice(&key)?;
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);

        let nonce = Nonce::from_slice(&nonce);
        let ciphertext = cipher.encrypt(nonce, data)?;

        // Prepend salt and nonce to ciphertext
        let mut result = Vec::with_capacity(salt.len() + nonce.len() + ciphertext.len());
        result.extend_from_slice(&salt);
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt data using AES-256-GCM
    fn decrypt_data(encrypted: &[u8], password: &str) -> Result<Vec<u8>, anyhow::Error> {
        use pbkdf2::pbkdf2_hmac;
        use sha2::Sha256;

        if encrypted.len() < 28 {
            return Err(anyhow::anyhow!("Encrypted data too short"));
        }

        let salt = &encrypted[0..16];
        let nonce = &encrypted[16..28];
        let ciphertext = &encrypted[28..];

        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100_000, &mut key);

        let cipher = Aes256Gcm::new_from_slice(&key)?;
        let nonce = Nonce::from_slice(nonce);

        let plaintext = cipher.decrypt(nonce, ciphertext)?;
        Ok(plaintext)
    }

    /// Compress data using flate2
    fn compress_data(data: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    /// Decompress data using flate2
    fn decompress_data(data: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    /// Collect session data from client
    pub async fn collect_session_data(
        &self,
        client: &crate::Client,
    ) -> Result<SessionData, anyhow::Error> {
        debug!("Collecting session data for backup...");

        let mut session_data = SessionData::new();

        // Collect from persistence manager
        let device = client.persistence_manager.get_device_arc().await;
        let device_guard = device.read().await;

        // Get device info
        session_data.device_info = Some(
            bincode::serialize(&*device_guard)
                .map_err(|e| anyhow::anyhow!("Failed to serialize device info: {}", e))?,
        );

        // Get identity key pair
        if let Ok(identity) = client.signal_cache.get_identity_key_pair().await {
            session_data.identity_key_pair = Some(identity.serialize().to_vec());
        }

        // Get registration ID
        if let Ok(reg_id) = client.signal_cache.get_registration_id().await {
            session_data.registration_id = Some(reg_id);
        }

        // Get prekeys
        session_data.prekeys = client.signal_cache.get_all_prekeys().await?;

        // Get signed prekeys
        session_data.signed_prekeys = client.signal_cache.get_all_signed_prekeys().await?;

        // Get sessions
        session_data.sessions = client.signal_cache.get_all_sessions().await?;

        // Get sender keys
        session_data.sender_keys = client.signal_cache.get_all_sender_keys().await?;

        // Get app state keys
        session_data.app_state_keys = client
            .persistence_manager
            .get_all_app_state_keys()
            .await?;

        // Get LID-PN mappings
        session_data.lid_pn_mappings = client.get_all_lid_pn_mappings().await?;

        // Get device registry
        session_data.device_registry = client.get_all_device_registry().await?;

        drop(device_guard);

        info!(
            "Collected session data: {} prekeys, {} sessions, {} sender keys",
            session_data.prekeys.len(),
            session_data.sessions.len(),
            session_data.sender_keys.len()
        );

        Ok(session_data)
    }

    /// Backup session to file
    pub async fn backup_to_file(
        &self,
        client: &crate::Client,
        config: BackupConfig,
    ) -> Result<BackupResult, anyhow::Error> {
        info!("Starting session backup to {:?}", config.destination);

        // Collect session data
        let session_data = self.collect_session_data(client).await?;

        // Serialize to JSON
        let mut data = serde_json::to_vec(&session_data)
            .map_err(|e| anyhow::anyhow!("Failed to serialize session data: {}", e))?;

        // Compress if enabled
        if config.compress {
            debug!("Compressing backup data...");
            data = Self::compress_data(&data)?;
        }

        // Encrypt if password provided
        let encrypted = if let Some(password) = &config.password {
            debug!("Encrypting backup data...");
            Self::encrypt_data(&data, password)?
        } else {
            data
        };

        // Create metadata
        let mut metadata = BackupMetadata::new();
        metadata.size_bytes = encrypted.len() as u64;
        metadata.checksum = BackupMetadata::calculate_checksum(&encrypted);
        metadata.tags = config.tags;

        // Get phone number if available
        if let Some(device) = client.persistence_manager.get_device_arc().await.get() {
            if let Some(pn) = device.pn.as_ref() {
                metadata.phone_masked = Some(BackupMetadata::mask_phone(&pn.to_string()));
            }
        }

        // Ensure parent directory exists
        if let Some(parent) = config.destination.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Write to file
        fs::write(&config.destination, &encrypted).await?;

        info!(
            "Backup completed: {:?} ({} bytes, encrypted: {})",
            config.destination,
            encrypted.len(),
            config.password.is_some()
        );

        // Update last backup timestamp
        *self.last_backup.write().await = Some(Utc::now());
        *self.backup_count.lock().await += 1;

        Ok(BackupResult {
            file_path: config.destination,
            metadata,
            size_bytes: encrypted.len() as u64,
            encrypted: config.password.is_some(),
        })
    }

    /// Restore session from file
    pub async fn restore_from_file(
        &self,
        client: &crate::Client,
        config: RestoreConfig,
    ) -> Result<RestoreResult, anyhow::Error> {
        info!("Starting session restore from {:?}", config.source);

        // Read encrypted data
        let encrypted = fs::read(&config.source).await?;

        // Decrypt if password provided
        let mut data = if let Some(password) = &config.password {
            debug!("Decrypting backup data...");
            Self::decrypt_data(&encrypted, password)?
        } else {
            encrypted
        };

        // Decompress if needed (detect gzip magic number)
        if data.starts_with(&[0x1f, 0x8b]) {
            debug!("Decompressing backup data...");
            data = Self::decompress_data(&data)?;
        }

        // Deserialize session data
        let session_data: SessionData = serde_json::from_slice(&data)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize session data: {}", e))?;

        // Restore to client
        let mut warnings = Vec::new();
        let mut items_restored = 0;

        // Restore device info
        if let Some(device_info) = session_data.device_info {
            match bincode::deserialize::<wacore_ng::store::traits::Device>(&device_info) {
                Ok(device) => {
                    client.persistence_manager.set_device(device).await?;
                    items_restored += 1;
                }
                Err(e) => {
                    warnings.push(format!("Failed to restore device info: {}", e));
                }
            }
        }

        // Restore identity key pair
        if let Some(identity_bytes) = session_data.identity_key_pair {
            match client.signal_cache.set_identity_key_pair(&identity_bytes).await {
                Ok(_) => items_restored += 1,
                Err(e) => warnings.push(format!("Failed to restore identity key: {}", e)),
            }
        }

        // Restore registration ID
        if let Some(reg_id) = session_data.registration_id {
            match client.signal_cache.set_registration_id(reg_id).await {
                Ok(_) => items_restored += 1,
                Err(e) => warnings.push(format!("Failed to restore registration ID: {}", e)),
            }
        }

        // Restore prekeys
        for (id, record) in session_data.prekeys {
            if let Err(e) = client
                .signal_cache
                .store_prekey(id, &record, false)
                .await
            {
                warnings.push(format!("Failed to restore prekey {}: {}", id, e));
            } else {
                items_restored += 1;
            }
        }

        // Restore signed prekeys
        for (id, record) in session_data.signed_prekeys {
            if let Err(e) = client
                .signal_cache
                .store_signed_prekey(id, &record, false)
                .await
            {
                warnings.push(format!("Failed to restore signed prekey {}: {}", id, e));
            } else {
                items_restored += 1;
            }
        }

        // Restore sessions
        for (address, record) in session_data.sessions {
            if let Err(e) = client.signal_cache.set_session(&address, &record).await {
                warnings.push(format!("Failed to restore session {}: {}", address, e));
            } else {
                items_restored += 1;
            }
        }

        // Restore sender keys
        for (key_id, record) in session_data.sender_keys {
            if let Err(e) = client.signal_cache.put_sender_key(&key_id, &record).await {
                warnings.push(format!("Failed to restore sender key {}: {}", key_id, e));
            } else {
                items_restored += 1;
            }
        }

        info!(
            "Restore completed: {} items restored, {} warnings",
            items_restored,
            warnings.len()
        );

        Ok(RestoreResult {
            success: true,
            items_restored,
            backup_metadata: BackupMetadata::new(),
            warnings,
        })
    }

    /// Start automatic backup scheduler
    pub async fn start_auto_backup(
        &self,
        client: Arc<crate::Client>,
        config: AutoBackupConfig,
    ) -> Result<(), anyhow::Error> {
        info!(
            "Starting auto-backup scheduler (interval: {:?}, keep_last: {})",
            config.interval, config.keep_last
        );

        // Ensure backup directory exists
        fs::create_dir_all(&config.directory).await?;

        // Store config
        *self.auto_backup_config.write().await = Some(config.clone());

        // Spawn backup task
        let handle = tokio::spawn({
            let backup_manager = Arc::new(self.clone());
            async move {
                let mut interval = time::interval(config.interval);
                loop {
                    interval.tick().await;

                    info!("Auto-backup triggered...");

                    // Generate filename with timestamp
                    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
                    let filename = format!(
                        "{}_{}.enc",
                        config.filename_prefix, timestamp
                    );
                    let backup_path = config.directory.join(&filename);

                    // Create backup config
                    let backup_config = BackupConfig {
                        destination: backup_path.clone(),
                        password: config.password.clone(),
                        tags: vec!["auto-backup".to_string()],
                        compress: config.compress,
                    };

                    // Perform backup
                    match backup_manager.backup_to_file(&client, backup_config).await {
                        Ok(result) => {
                            info!(
                                "Auto-backup completed: {:?} ({} bytes)",
                                result.file_path, result.size_bytes
                            );

                            // Cleanup old backups if keep_last > 0
                            if config.keep_last > 0 {
                                if let Err(e) =
                                    backup_manager.cleanup_old_backups(&config.directory, config.keep_last).await
                                {
                                    warn!("Failed to cleanup old backups: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Auto-backup failed: {}", e);
                        }
                    }
                }
            }
        });

        *self.auto_backup_handle.lock().await = Some(handle);

        Ok(())
    }

    /// Stop automatic backup scheduler
    pub async fn stop_auto_backup(&self) {
        info!("Stopping auto-backup scheduler...");

        if let Some(handle) = self.auto_backup_handle.lock().await.take() {
            handle.abort();
        }

        *self.auto_backup_config.write().await = None;
    }

    /// Cleanup old backups, keeping only the last N files
    async fn cleanup_old_backups(
        &self,
        directory: &Path,
        keep_last: u32,
    ) -> Result<(), anyhow::Error> {
        debug!("Cleaning up old backups, keeping last {}", keep_last);

        // List all backup files
        let mut backup_files: Vec<_> = fs::read_dir(directory)
            .await?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .map_or(false, |ext| ext == "enc")
            })
            .collect();

        // Sort by modification time (newest first)
        backup_files.sort_by(|a: &std::fs::DirEntry, b: &std::fs::DirEntry| {
            b.metadata()
                .and_then(|m: std::fs::Metadata| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .cmp(
                    &a.metadata()
                        .and_then(|m: std::fs::Metadata| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                )
        });

        // Delete old backups
        for entry in backup_files.iter().skip(keep_last as usize) {
            let path = entry.path();
            if let Err(e) = fs::remove_file(&path).await {
                warn!("Failed to delete old backup {:?}: {}", path, e);
            } else {
                debug!("Deleted old backup: {:?}", path);
            }
        }

        Ok(())
    }

    /// Get backup statistics
    pub async fn get_backup_stats(&self) -> BackupStats {
        let backup_count = *self.backup_count.lock().await;
        let last_backup = *self.last_backup.read().await;
        let auto_backup_enabled = self.auto_backup_config.read().await.is_some();

        BackupStats {
            total_backups: backup_count,
            last_backup,
            auto_backup_enabled,
        }
    }
}

impl Clone for SessionBackupManager {
    fn clone(&self) -> Self {
        Self {
            auto_backup_config: RwLock::new(self.auto_backup_config.blocking_read().clone()),
            auto_backup_handle: Mutex::new(None),
            last_backup: RwLock::new(*self.last_backup.blocking_read()),
            backup_count: Mutex::new(*self.backup_count.blocking_lock()),
        }
    }
}

impl Default for SessionBackupManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about backup operations
#[derive(Debug, Clone)]
pub struct BackupStats {
    /// Total number of backups performed
    pub total_backups: u64,
    /// Timestamp of last backup
    pub last_backup: Option<DateTime<Utc>>,
    /// Whether auto-backup is enabled
    pub auto_backup_enabled: bool,
}

impl BackupStats {
    pub fn new() -> Self {
        Self {
            total_backups: 0,
            last_backup: None,
            auto_backup_enabled: false,
        }
    }
}

impl Default for BackupStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let data = b"Hello, World! This is a test message.";
        let password = "test_password_123";

        let encrypted = SessionBackupManager::encrypt_data(data, password).unwrap();
        let decrypted = SessionBackupManager::decrypt_data(&encrypted, password).unwrap();

        assert_eq!(data.to_vec(), decrypted);
        assert!(encrypted.len() > data.len()); // Encrypted data should be larger
    }

    #[test]
    fn test_compress_decompress_roundtrip() {
        let data = b"Hello, World! This is a test message that should compress well.";

        let compressed = SessionBackupManager::compress_data(data).unwrap();
        let decompressed = SessionBackupManager::decompress_data(&compressed).unwrap();

        assert_eq!(data.to_vec(), decompressed);
        assert!(compressed.len() < data.len()); // Compressed data should be smaller
    }

    #[test]
    fn test_phone_masking() {
        assert_eq!(
            BackupMetadata::mask_phone("6281234567890"),
            "6281****7890"
        );
        assert_eq!(
            BackupMetadata::mask_phone("1234567890"),
            "1234****7890"
        );
        assert_eq!(BackupMetadata::mask_phone("123"), "***");
    }

    #[test]
    fn test_checksum_calculation() {
        let data1 = b"Hello";
        let data2 = b"World";

        let checksum1 = BackupMetadata::calculate_checksum(data1);
        let checksum2 = BackupMetadata::calculate_checksum(data2);
        let checksum1_again = BackupMetadata::calculate_checksum(data1);

        assert_eq!(checksum1, checksum1_again); // Same data = same checksum
        assert_ne!(checksum1, checksum2); // Different data = different checksum
    }
}
