













































use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use rand::Rng;
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


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    
    pub created_at: DateTime<Utc>,
    
    pub ruwa_version: String,
    
    pub wa_version: String,
    
    pub phone_masked: Option<String>,
    
    pub size_bytes: u64,
    
    pub checksum: String,
    
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

    
    pub fn mask_phone(phone: &str) -> String {
        if phone.len() > 8 {
            format!("{}****{}", &phone[..4], &phone[phone.len() - 4..])
        } else {
            "***".to_string()
        }
    }

    
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


#[derive(Debug, Clone)]
pub struct BackupConfig {
    
    pub destination: PathBuf,
    
    pub password: Option<String>,
    
    pub tags: Vec<String>,
    
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


#[derive(Debug, Clone)]
pub struct RestoreConfig {
    
    pub source: PathBuf,
    
    pub password: Option<String>,
    
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


#[derive(Debug, Clone)]
pub struct AutoBackupConfig {
    
    pub interval: Duration,
    
    pub directory: PathBuf,
    
    pub keep_last: u32,
    
    pub filename_prefix: String,
    
    pub compress: bool,
    
    pub password: Option<String>,
}

impl Default for AutoBackupConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(3600), 
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


#[derive(Debug, Clone)]
pub struct BackupResult {
    
    pub file_path: PathBuf,
    
    pub metadata: BackupMetadata,
    
    pub size_bytes: u64,
    
    pub encrypted: bool,
}


#[derive(Debug, Clone)]
pub struct RestoreResult {
    
    pub success: bool,
    
    pub items_restored: usize,
    
    pub backup_metadata: BackupMetadata,
    
    pub warnings: Vec<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    
    pub identity_key_pair: Option<Vec<u8>>,
    
    pub registration_id: Option<u32>,
    
    pub prekeys: Vec<(u32, Vec<u8>)>,
    
    pub signed_prekeys: Vec<(u32, Vec<u8>)>,
    
    pub sessions: Vec<(String, Vec<u8>)>,
    
    pub sender_keys: Vec<(String, Vec<u8>)>,
    
    pub device_info: Option<Vec<u8>>,
    
    pub app_state_keys: Vec<(Vec<u8>, Vec<u8>)>,
    
    pub lid_pn_mappings: Vec<(String, String)>,
    
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


pub struct SessionBackupManager {
    
    auto_backup_config: RwLock<Option<AutoBackupConfig>>,
    
    auto_backup_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
    
    last_backup: RwLock<Option<DateTime<Utc>>>,
    
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

    
    fn encrypt_data(data: &[u8], password: &str) -> Result<Vec<u8>, anyhow::Error> {
        
        use pbkdf2::pbkdf2_hmac;
        use sha2::Sha256;

        let mut salt = [0u8; 16];
        rand::thread_rng().fill(&mut salt);

        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, 100_000, &mut key);

        let cipher = Aes256Gcm::new_from_slice(&key)?;
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill(&mut nonce_bytes);

        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, data)?;

        
        let mut result = Vec::with_capacity(salt.len() + nonce.len() + ciphertext.len());
        result.extend_from_slice(&salt);
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    
    fn decrypt_data(encrypted: &[u8], password: &str) -> Result<Vec<u8>, anyhow::Error> {
        use pbkdf2::pbkdf2_hmac;
        use sha2::Sha256;

        if encrypted.len() < 28 {
            return Err(anyhow::anyhow!("Encrypted data too short"));
        }

        let salt = &encrypted[0..16];
        let nonce_bytes = &encrypted[16..28];
        let ciphertext = &encrypted[28..];

        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100_000, &mut key);

        let cipher = Aes256Gcm::new_from_slice(&key)?;
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher.decrypt(nonce, ciphertext)?;
        Ok(plaintext)
    }

    
    fn compress_data(data: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    
    fn decompress_data(data: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    
    pub async fn collect_session_data(
        &self,
        client: &crate::Client,
    ) -> Result<SessionData, anyhow::Error> {
        debug!("Collecting session data for backup...");

        let mut session_data = SessionData::new();

        
        let device = client.persistence_manager.get_device_arc().await;
        let device_guard = device.read().await;
        
        
        if let Some(pn) = device_guard.pn.as_ref() {
            session_data.device_info = Some(pn.to_string().into_bytes());
        }

        drop(device_guard);

        info!(
            "Collected session data for backup (phone: {})",
            session_data.device_info.as_ref().map(|d| String::from_utf8_lossy(d)).unwrap_or_default()
        );

        Ok(session_data)
    }

    
    pub async fn backup_to_file(
        &self,
        client: &crate::Client,
        config: BackupConfig,
    ) -> Result<BackupResult, anyhow::Error> {
        info!("Starting session backup to {:?}", config.destination);

        
        let session_data = self.collect_session_data(client).await?;

        
        let mut data = serde_json::to_vec(&session_data)
            .map_err(|e| anyhow::anyhow!("Failed to serialize session data: {}", e))?;

        
        if config.compress {
            debug!("Compressing backup data...");
            data = Self::compress_data(&data)?;
        }

        
        let encrypted = if let Some(password) = &config.password {
            debug!("Encrypting backup data...");
            Self::encrypt_data(&data, password)?
        } else {
            data
        };

        
        let mut metadata = BackupMetadata::new();
        metadata.size_bytes = encrypted.len() as u64;
        metadata.checksum = BackupMetadata::calculate_checksum(&encrypted);
        metadata.tags = config.tags;

        
        let device = client.persistence_manager.get_device_arc().await;
        let device_guard = device.read().await;
        if let Some(pn) = device_guard.pn.as_ref() {
            metadata.phone_masked = Some(BackupMetadata::mask_phone(&pn.to_string()));
        }
        drop(device_guard);

        
        if let Some(parent) = config.destination.parent() {
            fs::create_dir_all(parent).await?;
        }

        
        fs::write(&config.destination, &encrypted).await?;

        info!(
            "Backup completed: {:?} ({} bytes, encrypted: {})",
            config.destination,
            encrypted.len(),
            config.password.is_some()
        );

        
        *self.last_backup.write().await = Some(Utc::now());
        *self.backup_count.lock().await += 1;

        Ok(BackupResult {
            file_path: config.destination,
            metadata,
            size_bytes: encrypted.len() as u64,
            encrypted: config.password.is_some(),
        })
    }

    
    pub async fn restore_from_file(
        &self,
        client: &crate::Client,
        config: RestoreConfig,
    ) -> Result<RestoreResult, anyhow::Error> {
        info!("Starting session restore from {:?}", config.source);

        
        let encrypted = fs::read(&config.source).await?;

        
        let mut data = if let Some(password) = &config.password {
            debug!("Decrypting backup data...");
            Self::decrypt_data(&encrypted, password)?
        } else {
            encrypted
        };

        
        if data.starts_with(&[0x1f, 0x8b]) {
            debug!("Decompressing backup data...");
            data = Self::decompress_data(&data)?;
        }

        
        let session_data: SessionData = serde_json::from_slice(&data)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize session data: {}", e))?;

        
        let mut warnings = Vec::new();
        let mut items_restored = 0;

        
        if let Some(device_info) = session_data.device_info {
            
            
            if let Ok(phone) = String::from_utf8(device_info.clone()) {
                info!("Restored session for phone: {}", BackupMetadata::mask_phone(&phone));
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

    
    pub async fn start_auto_backup(
        &self,
        client: Arc<crate::Client>,
        config: AutoBackupConfig,
    ) -> Result<(), anyhow::Error> {
        info!(
            "Starting auto-backup scheduler (interval: {:?}, keep_last: {})",
            config.interval, config.keep_last
        );

        
        fs::create_dir_all(&config.directory).await?;

        
        *self.auto_backup_config.write().await = Some(config.clone());

        
        let handle = tokio::spawn({
            let backup_manager = Arc::new(self.clone());
            async move {
                let mut interval = time::interval(config.interval);
                loop {
                    interval.tick().await;

                    info!("Auto-backup triggered...");

                    
                    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
                    let filename = format!(
                        "{}_{}.enc",
                        config.filename_prefix, timestamp
                    );
                    let backup_path = config.directory.join(&filename);

                    
                    let backup_config = BackupConfig {
                        destination: backup_path.clone(),
                        password: config.password.clone(),
                        tags: vec!["auto-backup".to_string()],
                        compress: config.compress,
                    };

                    
                    match backup_manager.backup_to_file(&client, backup_config).await {
                        Ok(result) => {
                            info!(
                                "Auto-backup completed: {:?} ({} bytes)",
                                result.file_path, result.size_bytes
                            );

                            
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

    
    pub async fn stop_auto_backup(&self) {
        info!("Stopping auto-backup scheduler...");

        if let Some(handle) = self.auto_backup_handle.lock().await.take() {
            handle.abort();
        }

        *self.auto_backup_config.write().await = None;
    }

    
    async fn cleanup_old_backups(
        &self,
        directory: &Path,
        keep_last: u32,
    ) -> Result<(), anyhow::Error> {
        debug!("Cleaning up old backups, keeping last {}", keep_last);

        
        let entries = std::fs::read_dir(directory)?;
        let mut backup_files: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| {
                path.extension()
                    .map_or(false, |ext| ext == "enc")
            })
            .collect();

        
        backup_files.sort_by(|a: &PathBuf, b: &PathBuf| {
            let a_time = a.metadata().ok().and_then(|m| m.modified().ok()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let b_time = b.metadata().ok().and_then(|m| m.modified().ok()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            b_time.cmp(&a_time)
        });

        
        for path in backup_files.iter().skip(keep_last as usize) {
            if let Err(e) = fs::remove_file(path).await {
                warn!("Failed to delete old backup {:?}: {}", path, e);
            } else {
                debug!("Deleted old backup: {:?}", path);
            }
        }

        Ok(())
    }

    
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


#[derive(Debug, Clone)]
pub struct BackupStats {
    
    pub total_backups: u64,
    
    pub last_backup: Option<DateTime<Utc>>,
    
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
        assert!(encrypted.len() > data.len()); 
    }

    #[test]
    fn test_compress_decompress_roundtrip() {
        let data = b"Hello, World! This is a test message that should compress well.";

        let compressed = SessionBackupManager::compress_data(data).unwrap();
        let decompressed = SessionBackupManager::decompress_data(&compressed).unwrap();

        assert_eq!(data.to_vec(), decompressed);
        assert!(compressed.len() < data.len()); 
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

        assert_eq!(checksum1, checksum1_again); 
        assert_ne!(checksum1, checksum2); 
    }
}
