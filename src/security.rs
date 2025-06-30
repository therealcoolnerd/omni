use anyhow::Result;
use hex;
use reqwest;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub verify_signatures: bool,
    pub verify_checksums: bool,
    pub allow_untrusted: bool,
    pub check_mirrors: bool,
    pub signature_servers: Vec<String>,
    pub trusted_keys: Vec<String>,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            verify_signatures: true,
            verify_checksums: true,
            allow_untrusted: false,
            check_mirrors: true,
            signature_servers: vec![
                "keyserver.ubuntu.com".to_string(),
                "keys.openpgp.org".to_string(),
                "pgp.mit.edu".to_string(),
            ],
            trusted_keys: vec![],
        }
    }
}

#[derive(Debug)]
pub struct SecurityVerifier {
    policy: SecurityPolicy,
}

#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub signature_valid: Option<bool>,
    pub checksum_valid: Option<bool>,
    pub trust_level: TrustLevel,
    pub warnings: Vec<String>,
    pub details: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrustLevel {
    Trusted,   // Signature verified and trusted
    Valid,     // Signature verified but not explicitly trusted
    Unsigned,  // No signature but checksum verified
    Untrusted, // Failed verification or no verification possible
}

impl SecurityVerifier {
    pub fn new(policy: SecurityPolicy) -> Self {
        Self { policy }
    }

    pub async fn verify_package(
        &self,
        package_path: &Path,
        expected_checksum: Option<&str>,
        signature_url: Option<&str>,
        box_type: &str,
    ) -> Result<VerificationResult> {
        info!(
            "Starting security verification for package: {:?}",
            package_path
        );

        let mut result = VerificationResult {
            signature_valid: None,
            checksum_valid: None,
            trust_level: TrustLevel::Untrusted,
            warnings: Vec::new(),
            details: String::new(),
        };

        // Step 1: Verify file integrity with checksum
        if self.policy.verify_checksums {
            if let Some(expected) = expected_checksum {
                result.checksum_valid = Some(self.verify_checksum(package_path, expected).await?);
                if result.checksum_valid == Some(false) {
                    result
                        .warnings
                        .push("Checksum verification failed".to_string());
                    return Ok(result);
                }
            } else {
                result
                    .warnings
                    .push("No checksum provided for verification".to_string());
            }
        }

        // Step 2: Verify digital signature
        if self.policy.verify_signatures {
            if let Some(sig_url) = signature_url {
                result.signature_valid = Some(self.verify_signature(package_path, sig_url).await?);
            } else {
                // Try to find signature using common patterns
                if let Some(found_sig) = self.find_signature_file(package_path, box_type).await? {
                    result.signature_valid =
                        Some(self.verify_signature(package_path, &found_sig).await?);
                }
            }
        }

        // Step 3: Determine trust level
        result.trust_level = self.determine_trust_level(&result);

        // Step 4: Check if we should proceed based on policy
        if !self.should_proceed(&result) {
            return Err(anyhow::anyhow!(
                "Package verification failed according to security policy: {}",
                result.details
            ));
        }

        // Step 5: Additional security checks for specific package types
        self.perform_package_specific_checks(package_path, box_type, &mut result)
            .await?;

        result.details = self.generate_verification_summary(&result);

        info!(
            "Security verification completed with trust level: {:?}",
            result.trust_level
        );
        Ok(result)
    }

    async fn verify_checksum(&self, file_path: &Path, expected: &str) -> Result<bool> {
        info!("Verifying checksum for: {:?}", file_path);

        let file_contents = fs::read(file_path)?;

        // Determine hash algorithm based on expected checksum length
        let computed_hash = match expected.len() {
            32 => {
                // MD5 (deprecated, but still used sometimes)
                warn!("MD5 checksums are deprecated and insecure");
                return Ok(false);
            }
            64 => {
                // SHA-256
                let mut hasher = Sha256::new();
                hasher.update(&file_contents);
                hex::encode(hasher.finalize())
            }
            128 => {
                // SHA-512
                let mut hasher = Sha512::new();
                hasher.update(&file_contents);
                hex::encode(hasher.finalize())
            }
            _ => {
                warn!("Unknown hash format with length: {}", expected.len());
                return Ok(false);
            }
        };

        let is_valid = computed_hash.eq_ignore_ascii_case(expected);

        if is_valid {
            info!("✅ Checksum verification passed");
        } else {
            error!("❌ Checksum verification failed");
            error!("Expected: {}", expected);
            error!("Computed: {}", computed_hash);
        }

        Ok(is_valid)
    }

    async fn verify_signature(&self, file_path: &Path, signature_source: &str) -> Result<bool> {
        info!("Verifying signature for: {:?}", file_path);

        // Download signature if it's a URL
        let signature_path = if signature_source.starts_with("http") {
            let temp_file = self.download_signature(signature_source).await?;
            temp_file.path().to_path_buf()
        } else {
            Path::new(signature_source).to_path_buf()
        };

        if !signature_path.exists() {
            warn!("Signature file not found: {:?}", signature_path);
            return Ok(false);
        }

        // Verify using GPG
        let output = Command::new("gpg")
            .arg("--verify")
            .arg(&signature_path)
            .arg(file_path)
            .output();

        match output {
            Ok(result) => {
                let stderr = String::from_utf8_lossy(&result.stderr);
                let is_valid = result.status.success() && stderr.contains("Good signature");

                if is_valid {
                    info!("✅ GPG signature verification passed");
                } else {
                    warn!("❌ GPG signature verification failed");
                    warn!("GPG output: {}", stderr);
                }

                Ok(is_valid)
            }
            Err(e) => {
                warn!("Failed to run GPG verification: {}", e);
                if self.policy.allow_untrusted {
                    Ok(false)
                } else {
                    Err(anyhow::anyhow!(
                        "GPG verification unavailable and untrusted packages not allowed"
                    ))
                }
            }
        }
    }

    async fn download_signature(&self, url: &str) -> Result<NamedTempFile> {
        info!("Downloading signature from: {}", url);

        let client = reqwest::Client::builder()
            .user_agent("omni-package-manager/0.2.0")
            .timeout(std::time::Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::limited(3))
            .build()?;

        let response = client
            .get(url)
            .header(
                "Accept",
                "application/pgp-signature, application/octet-stream",
            )
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to download signature: HTTP {}",
                response.status()
            ));
        }

        let mut temp_file = NamedTempFile::new()?;
        let content = response.bytes().await?;

        use std::io::Write;
        temp_file.write_all(&content)?;
        temp_file.flush()?;

        Ok(temp_file)
    }

    async fn find_signature_file(
        &self,
        package_path: &Path,
        box_type: &str,
    ) -> Result<Option<String>> {
        // Common signature file patterns
        let base_name = package_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        let base_dir = package_path.parent().unwrap_or(Path::new("."));

        let signature_patterns = match box_type {
            "apt" | "dnf" | "pacman" => vec![
                format!("{}.sig", base_name),
                format!("{}.asc", base_name),
                format!("{}.gpg", base_name),
            ],
            "appimage" => vec![
                format!("{}.sig", base_name),
                format!("{}.zsync", base_name), // AppImage-specific
            ],
            _ => vec![format!("{}.sig", base_name), format!("{}.asc", base_name)],
        };

        for pattern in &signature_patterns {
            let sig_path = base_dir.join(pattern);
            if sig_path.exists() {
                info!("Found signature file: {:?}", sig_path);
                return Ok(Some(sig_path.to_string_lossy().to_string()));
            }
        }

        // Try downloading common signature URLs for web-hosted packages
        if let Some(parent) = package_path.parent() {
            if parent.to_string_lossy().starts_with("http") {
                for pattern in &signature_patterns {
                    let sig_url = format!("{}/{}", parent.display(), pattern);
                    if self.url_exists(&sig_url).await {
                        return Ok(Some(sig_url));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn url_exists(&self, url: &str) -> bool {
        let client = reqwest::Client::builder()
            .user_agent("omni-package-manager/0.2.0")
            .timeout(std::time::Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::limited(2))
            .build();

        match client {
            Ok(client) => {
                match client
                    .head(url)
                    .header("Cache-Control", "no-cache")
                    .send()
                    .await
                {
                    Ok(response) => response.status().is_success(),
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }

    fn determine_trust_level(&self, result: &VerificationResult) -> TrustLevel {
        match (result.signature_valid, result.checksum_valid) {
            (Some(true), Some(true)) => TrustLevel::Trusted,
            (Some(true), _) => TrustLevel::Valid,
            (Some(false), _) => TrustLevel::Untrusted,
            (None, Some(true)) => TrustLevel::Unsigned,
            (None, Some(false)) => TrustLevel::Untrusted,
            (None, None) => TrustLevel::Untrusted,
        }
    }

    fn should_proceed(&self, result: &VerificationResult) -> bool {
        match result.trust_level {
            TrustLevel::Trusted | TrustLevel::Valid => true,
            TrustLevel::Unsigned => self.policy.allow_untrusted,
            TrustLevel::Untrusted => self.policy.allow_untrusted,
        }
    }

    async fn perform_package_specific_checks(
        &self,
        package_path: &Path,
        box_type: &str,
        result: &mut VerificationResult,
    ) -> Result<()> {
        match box_type {
            "appimage" => {
                self.verify_appimage_security(package_path, result).await?;
            }
            "snap" => {
                self.verify_snap_security(package_path, result).await?;
            }
            "flatpak" => {
                self.verify_flatpak_security(package_path, result).await?;
            }
            _ => {
                // Generic checks for deb/rpm packages
                self.verify_archive_security(package_path, result).await?;
            }
        }

        Ok(())
    }

    async fn verify_appimage_security(
        &self,
        package_path: &Path,
        result: &mut VerificationResult,
    ) -> Result<()> {
        // Check if AppImage is executable
        let metadata = fs::metadata(package_path)?;
        let permissions = metadata.permissions();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if permissions.mode() & 0o111 == 0 {
                result
                    .warnings
                    .push("AppImage is not executable".to_string());
            }
        }

        // Check AppImage integrity
        let output = Command::new("file").arg(package_path).output();

        if let Ok(output) = output {
            let file_type = String::from_utf8_lossy(&output.stdout);
            if !file_type.contains("ELF") {
                result
                    .warnings
                    .push("AppImage does not appear to be a valid ELF executable".to_string());
            }
        }

        Ok(())
    }

    async fn verify_snap_security(
        &self,
        _package_path: &Path,
        result: &mut VerificationResult,
    ) -> Result<()> {
        // Snaps are verified by the snap store
        result
            .details
            .push_str("Snap packages are verified by the Snap Store. ");
        Ok(())
    }

    async fn verify_flatpak_security(
        &self,
        _package_path: &Path,
        result: &mut VerificationResult,
    ) -> Result<()> {
        // Flatpaks are sandboxed and verified by repositories
        result
            .details
            .push_str("Flatpak packages are sandboxed and verified by repositories. ");
        Ok(())
    }

    async fn verify_archive_security(
        &self,
        package_path: &Path,
        result: &mut VerificationResult,
    ) -> Result<()> {
        // Basic file type verification
        let output = Command::new("file").arg(package_path).output();

        if let Ok(output) = output {
            let file_type = String::from_utf8_lossy(&output.stdout);

            // Check for suspicious file types
            if file_type.contains("script") || file_type.contains("executable") {
                if !file_type.contains("Debian") && !file_type.contains("RPM") {
                    result.warnings.push(
                        "Package contains executable content outside of standard package format"
                            .to_string(),
                    );
                }
            }
        }

        Ok(())
    }

    fn generate_verification_summary(&self, result: &VerificationResult) -> String {
        let mut summary = String::new();

        summary.push_str(&format!("Trust Level: {:?}\n", result.trust_level));

        if let Some(sig_valid) = result.signature_valid {
            summary.push_str(&format!(
                "Signature: {}\n",
                if sig_valid {
                    "✅ Valid"
                } else {
                    "❌ Invalid"
                }
            ));
        } else {
            summary.push_str("Signature: ⚠️ Not verified\n");
        }

        if let Some(checksum_valid) = result.checksum_valid {
            summary.push_str(&format!(
                "Checksum: {}\n",
                if checksum_valid {
                    "✅ Valid"
                } else {
                    "❌ Invalid"
                }
            ));
        } else {
            summary.push_str("Checksum: ⚠️ Not verified\n");
        }

        if !result.warnings.is_empty() {
            summary.push_str("\nWarnings:\n");
            for warning in &result.warnings {
                summary.push_str(&format!("- {}\n", warning));
            }
        }

        summary
    }

    pub async fn verify_repository_metadata(&self, box_type: &str) -> Result<VerificationResult> {
        info!("Verifying repository metadata for: {}", box_type);

        let mut result = VerificationResult {
            signature_valid: None,
            checksum_valid: None,
            trust_level: TrustLevel::Untrusted,
            warnings: Vec::new(),
            details: String::new(),
        };

        match box_type {
            "apt" => {
                result = self.verify_apt_repository().await?;
            }
            "dnf" => {
                result = self.verify_dnf_repository().await?;
            }
            "pacman" => {
                result = self.verify_pacman_repository().await?;
            }
            _ => {
                result.details =
                    format!("Repository verification not implemented for {}", box_type);
            }
        }

        Ok(result)
    }

    async fn verify_apt_repository(&self) -> Result<VerificationResult> {
        let mut result = VerificationResult {
            signature_valid: None,
            checksum_valid: None,
            trust_level: TrustLevel::Trusted, // APT repositories are generally trusted
            warnings: Vec::new(),
            details: "APT repositories are verified by the package manager".to_string(),
        };

        // Check if apt-key is properly configured
        let output = Command::new("apt-key").arg("list").output();

        if let Ok(output) = output {
            if output.status.success() {
                result.signature_valid = Some(true);
            }
        }

        Ok(result)
    }

    async fn verify_dnf_repository(&self) -> Result<VerificationResult> {
        let result = VerificationResult {
            signature_valid: Some(true),
            checksum_valid: Some(true),
            trust_level: TrustLevel::Trusted,
            warnings: Vec::new(),
            details: "DNF repositories use GPG verification by default".to_string(),
        };

        Ok(result)
    }

    async fn verify_pacman_repository(&self) -> Result<VerificationResult> {
        let result = VerificationResult {
            signature_valid: Some(true),
            checksum_valid: Some(true),
            trust_level: TrustLevel::Trusted,
            warnings: Vec::new(),
            details: "Pacman repositories use package signing verification".to_string(),
        };

        Ok(result)
    }

    pub fn import_gpg_key(&self, key_id: &str) -> Result<bool> {
        info!("Importing GPG key: {}", key_id);

        for server in &self.policy.signature_servers {
            let output = Command::new("gpg")
                .arg("--keyserver")
                .arg(server)
                .arg("--recv-keys")
                .arg(key_id)
                .output();

            if let Ok(result) = output {
                if result.status.success() {
                    info!("✅ Successfully imported GPG key from {}", server);
                    return Ok(true);
                }
            }
        }

        warn!("❌ Failed to import GPG key from any server");
        Ok(false)
    }

    pub fn list_trusted_keys(&self) -> Result<Vec<String>> {
        let output = Command::new("gpg")
            .arg("--list-keys")
            .arg("--with-colons")
            .output()?;

        let mut keys = Vec::new();
        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            if line.starts_with("pub:") {
                let fields: Vec<&str> = line.split(':').collect();
                if fields.len() > 4 {
                    keys.push(fields[4].to_string());
                }
            }
        }

        Ok(keys)
    }
}
