use anyhow::{anyhow, Result};
use regex::Regex;
use std::path::{Path, PathBuf};
use url::Url;

/// Input validation utilities for security
pub struct InputValidator;

impl InputValidator {
    /// Validate package names to prevent injection attacks
    pub fn validate_package_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(anyhow!("Package name cannot be empty"));
        }

        if name.len() > 255 {
            return Err(anyhow!("Package name too long (max 255 characters)"));
        }

        // Allow alphanumeric, hyphens, underscores, dots, plus signs
        let valid_chars = Regex::new(r"^[a-zA-Z0-9._+-]+$").unwrap();
        if !valid_chars.is_match(name) {
            return Err(anyhow!("Package name contains invalid characters"));
        }

        // Prevent path traversal
        if name.contains("..") || name.contains("/") || name.contains("\\") {
            return Err(anyhow!("Package name cannot contain path separators"));
        }

        // Prevent reserved names
        let reserved = [".", "..", "con", "prn", "aux", "nul"];
        if reserved.contains(&name.to_lowercase().as_str()) {
            return Err(anyhow!("Package name is reserved"));
        }

        Ok(())
    }

    /// Validate URLs to prevent SSRF and other attacks
    pub fn validate_url(url_str: &str) -> Result<Url> {
        if url_str.is_empty() {
            return Err(anyhow!("URL cannot be empty"));
        }

        if url_str.len() > 2048 {
            return Err(anyhow!("URL too long (max 2048 characters)"));
        }

        let url = Url::parse(url_str).map_err(|_| anyhow!("Invalid URL format"))?;

        // Only allow HTTP/HTTPS
        match url.scheme() {
            "http" | "https" => {}
            _ => return Err(anyhow!("Only HTTP/HTTPS URLs are allowed")),
        }

        // Prevent access to private networks
        if let Some(host) = url.host_str() {
            Self::validate_host(host)?;
        }

        Ok(url)
    }

    /// Validate file paths to prevent directory traversal
    pub fn validate_file_path(path_str: &str) -> Result<PathBuf> {
        if path_str.is_empty() {
            return Err(anyhow!("Path cannot be empty"));
        }

        if path_str.len() > 4096 {
            return Err(anyhow!("Path too long (max 4096 characters)"));
        }

        let path = Path::new(path_str);

        // Check for null bytes
        if path_str.contains('\0') {
            return Err(anyhow!("Path contains null bytes"));
        }

        // Check for dangerous path patterns
        if path_str.contains("../") || path_str.contains("..\\") {
            return Err(anyhow!("Path traversal patterns not allowed"));
        }

        // Canonicalize to resolve .. and . components
        let canonical = path
            .canonicalize()
            .map_err(|_| anyhow!("Cannot resolve path"))?;

        // Ensure the path doesn't escape allowed directories
        let allowed_prefixes = ["/tmp/", "/var/tmp/", "/home/", "/opt/", "/usr/local/", "/var/cache/"];

        let path_str = canonical.to_string_lossy();
        if !allowed_prefixes
            .iter()
            .any(|prefix| path_str.starts_with(prefix))
        {
            return Err(anyhow!("Path outside allowed directories"));
        }

        Ok(canonical)
    }

    /// Validate command line arguments to prevent injection
    pub fn validate_command_args(args: &[&str]) -> Result<()> {
        for arg in args {
            // Check for dangerous characters
            if arg.contains(';') || arg.contains('|') || arg.contains('&') || arg.contains('`') {
                return Err(anyhow!("Dangerous shell characters found in argument: {}", arg));
            }

            // Check for command substitution
            if arg.contains("$(") || arg.contains("${") || arg.contains("\\`") {
                return Err(anyhow!("Command substitution not allowed in argument: {}", arg));
            }

            // Check for null bytes
            if arg.contains('\0') {
                return Err(anyhow!("Null bytes not allowed in argument: {}", arg));
            }

            // Limit argument length
            if arg.len() > 1024 {
                return Err(anyhow!("Argument too long (max 1024 characters): {}", arg));
            }
        }

        Ok(())
    }

    /// Validate repository URLs for security
    pub fn validate_repository_url(url_str: &str) -> Result<Url> {
        let url = Self::validate_url(url_str)?;

        // Additional checks for repository URLs
        if let Some(host) = url.host_str() {
            // Block known malicious hosts (this would be extended in production)
            let blocked_hosts = ["malware.com", "phishing.net"];
            if blocked_hosts.contains(&host) {
                return Err(anyhow!("Repository host is blocked: {}", host));
            }
        }

        // Only allow trusted schemes for repositories
        match url.scheme() {
            "https" => {} // Preferred
            "http" => {} // Allowed but not preferred
            _ => return Err(anyhow!("Only HTTP/HTTPS schemes allowed for repositories")),
        }

        Ok(url)
    }

    /// Validate version strings to prevent injection
    pub fn validate_version_string(version: &str) -> Result<()> {
        if version.is_empty() {
            return Err(anyhow!("Version cannot be empty"));
        }

        if version.len() > 64 {
            return Err(anyhow!("Version string too long (max 64 characters)"));
        }

        // Allow only alphanumeric, dots, hyphens, and plus signs
        let valid_chars = Regex::new(r"^[a-zA-Z0-9._+-]+$").unwrap();
        if !valid_chars.is_match(version) {
            return Err(anyhow!("Version contains invalid characters"));
        }

        Ok(())
    }

    /// Validate environment variable values
    pub fn validate_env_var(name: &str, value: &str) -> Result<()> {
        // Validate variable name
        if name.is_empty() || name.len() > 128 {
            return Err(anyhow!("Invalid environment variable name length"));
        }
        
        let name_regex = Regex::new(r"^[A-Z_][A-Z0-9_]*$").unwrap();
        if !name_regex.is_match(name) {
            return Err(anyhow!("Invalid environment variable name format"));
        }
        
        // Validate variable value
        if value.len() > 4096 {
            return Err(anyhow!("Environment variable value too long"));
        }
        
        // Check for dangerous patterns
        if value.contains('\0') || value.contains("$(") || value.contains("${") {
            return Err(anyhow!("Dangerous patterns in environment variable value"));
        }
        
        Ok(())
    }

    /// Validate checksums
    pub fn validate_checksum(checksum: &str) -> Result<()> {
        if checksum.is_empty() {
            return Err(anyhow!("Checksum cannot be empty"));
        }
        
        // Check for valid hex characters and length
        let hex_regex = Regex::new(r"^[a-fA-F0-9]+$").unwrap();
        if !hex_regex.is_match(checksum) {
            return Err(anyhow!("Checksum contains invalid characters"));
        }
        
        // Check common hash lengths
        match checksum.len() {
            32 => {}, // MD5
            40 => {}, // SHA-1
            64 => {}, // SHA-256
            128 => {}, // SHA-512
            _ => return Err(anyhow!("Checksum has invalid length")),
        }
        
        Ok(())
    }

    /// Validate box type names
    pub fn validate_box_type(box_type: &str) -> Result<()> {
        if box_type.is_empty() {
            return Err(anyhow!("Box type cannot be empty"));
        }

        let valid_box_types = [
            "apt",
            "dnf",
            "pacman",
            "snap",
            "flatpak",
            "appimage",
            "winget",
            "chocolatey",
            "scoop",
            "homebrew",
            "mas",
        ];

        if !valid_box_types.contains(&box_type) {
            return Err(anyhow!("Invalid box type: {}", box_type));
        }

        Ok(())
    }

    /// Validate input to prevent shell injection attacks
    pub fn validate_shell_safe(input: &str) -> Result<()> {
        if input.is_empty() {
            return Err(anyhow!("Input cannot be empty"));
        }

        if input.len() > 1024 {
            return Err(anyhow!("Input too long (max 1024 characters)"));
        }

        // Prevent all shell metacharacters that could be used for injection
        let dangerous_chars = [
            '&', '|', ';', '`', '$', '(', ')', '{', '}', '<', '>', '\'', '"', '\\', '\n', '\r',
            '\t', '\0', '!', '?', '*', '[', ']', '~', '#',
        ];

        for &ch in &dangerous_chars {
            if input.contains(ch) {
                return Err(anyhow!(
                    "Input contains dangerous shell metacharacter: '{}'",
                    ch
                ));
            }
        }

        // Check for common injection patterns
        let dangerous_patterns = [
            "&&",
            "||",
            ";;",
            "$(",
            "${",
            "`",
            "..",
            "/etc/",
            "/bin/",
            "/usr/bin/",
            "passwd",
            "shadow",
            "/dev/",
            "/proc/",
            "/sys/",
            "rm -rf",
            "dd if=",
            "curl",
            "wget",
            "nc ",
            "netcat",
            "telnet",
            "ssh ",
            "ftp ",
            "python",
            "perl",
            "ruby",
            "php",
            "bash",
            "sh ",
            "zsh",
            "csh",
        ];

        let input_lower = input.to_lowercase();
        for pattern in &dangerous_patterns {
            if input_lower.contains(pattern) {
                return Err(anyhow!(
                    "Input contains potentially dangerous pattern: '{}'",
                    pattern
                ));
            }
        }

        Ok(())
    }

    fn validate_host(host: &str) -> Result<()> {
        use std::net::IpAddr;

        // Try to parse as IP address
        if let Ok(ip) = host.parse::<IpAddr>() {
            match ip {
                IpAddr::V4(ipv4) => {
                    let octets = ipv4.octets();

                    // Block private networks
                    if octets[0] == 10
                        || (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31)
                        || (octets[0] == 192 && octets[1] == 168)
                        || octets[0] == 127
                    {
                        // localhost
                        return Err(anyhow!("Access to private networks not allowed"));
                    }
                }
                IpAddr::V6(ipv6) => {
                    // Block localhost and private networks
                    if ipv6.is_loopback() || ipv6.segments()[0] == 0xfc00 {
                        return Err(anyhow!("Access to private networks not allowed"));
                    }
                }
            }
        } else {
            // Validate hostname format
            let hostname_regex = Regex::new(r"^[a-zA-Z0-9.-]+$").unwrap();
            if !hostname_regex.is_match(host) {
                return Err(anyhow!("Invalid hostname format"));
            }

            // Block localhost variations
            let blocked_hosts = ["localhost", "127.0.0.1", "::1"];
            if blocked_hosts.contains(&host.to_lowercase().as_str()) {
                return Err(anyhow!("Access to localhost not allowed"));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_name_validation() {
        // Valid names
        assert!(InputValidator::validate_package_name("firefox").is_ok());
        assert!(InputValidator::validate_package_name("gcc-9").is_ok());
        assert!(InputValidator::validate_package_name("lib64.so.1").is_ok());
        assert!(InputValidator::validate_package_name("python3.9").is_ok());

        // Invalid names
        assert!(InputValidator::validate_package_name("").is_err());
        assert!(InputValidator::validate_package_name("../etc/passwd").is_err());
        assert!(InputValidator::validate_package_name("test/package").is_err());
        assert!(InputValidator::validate_package_name("test\\package").is_err());
        assert!(InputValidator::validate_package_name("con").is_err());
    }

    #[test]
    fn test_url_validation() {
        // Valid URLs
        assert!(InputValidator::validate_url("https://example.com/package.deb").is_ok());
        assert!(InputValidator::validate_url("http://packages.ubuntu.com/pool/main/").is_ok());

        // Invalid URLs
        assert!(InputValidator::validate_url("").is_err());
        assert!(InputValidator::validate_url("ftp://example.com/file").is_err());
        assert!(InputValidator::validate_url("https://localhost/evil").is_err());
        assert!(InputValidator::validate_url("https://127.0.0.1/evil").is_err());
        assert!(InputValidator::validate_url("https://192.168.1.1/evil").is_err());
    }

    #[test]
    fn test_command_args_validation() {
        // Valid args
        assert!(InputValidator::validate_command_args(&[
            "install".to_string(),
            "firefox".to_string()
        ])
        .is_ok());
        assert!(
            InputValidator::validate_command_args(&["-y".to_string(), "--force".to_string()])
                .is_ok()
        );

        // Invalid args
        assert!(InputValidator::validate_command_args(&["test; rm -rf /".to_string()]).is_err());
        assert!(InputValidator::validate_command_args(&["test && malicious".to_string()]).is_err());
        assert!(InputValidator::validate_command_args(&["test`whoami`".to_string()]).is_err());
    }

    #[test]
    fn test_box_type_validation() {
        // Valid box types
        assert!(InputValidator::validate_box_type("apt").is_ok());
        assert!(InputValidator::validate_box_type("dnf").is_ok());
        assert!(InputValidator::validate_box_type("snap").is_ok());

        // Invalid box types
        assert!(InputValidator::validate_box_type("").is_err());
        assert!(InputValidator::validate_box_type("invalid").is_err());
        assert!(InputValidator::validate_box_type("custom_manager").is_err());
    }

    #[test]
    fn test_shell_safe_validation() {
        // Valid inputs
        assert!(InputValidator::validate_shell_safe("firefox").is_ok());
        assert!(InputValidator::validate_shell_safe("package-name").is_ok());
        assert!(InputValidator::validate_shell_safe("package.name").is_ok());
        assert!(InputValidator::validate_shell_safe("123").is_ok());

        // Invalid inputs - shell metacharacters
        assert!(InputValidator::validate_shell_safe("test;rm -rf /").is_err());
        assert!(InputValidator::validate_shell_safe("test && malicious").is_err());
        assert!(InputValidator::validate_shell_safe("test`whoami`").is_err());
        assert!(InputValidator::validate_shell_safe("test$(id)").is_err());
        assert!(InputValidator::validate_shell_safe("test|grep").is_err());
        assert!(InputValidator::validate_shell_safe("test'injection'").is_err());
        assert!(InputValidator::validate_shell_safe("test\"injection\"").is_err());
        assert!(InputValidator::validate_shell_safe("test<input").is_err());
        assert!(InputValidator::validate_shell_safe("test>output").is_err());

        // Invalid inputs - dangerous patterns
        assert!(InputValidator::validate_shell_safe("/etc/passwd").is_err());
        assert!(InputValidator::validate_shell_safe("rm -rf test").is_err());
        assert!(InputValidator::validate_shell_safe("wget malicious.com").is_err());
        assert!(InputValidator::validate_shell_safe("curl evil.com").is_err());
        assert!(InputValidator::validate_shell_safe("python exploit.py").is_err());
        assert!(InputValidator::validate_shell_safe("../../../etc/passwd").is_err());

        // Edge cases
        assert!(InputValidator::validate_shell_safe("").is_err());
        assert!(InputValidator::validate_shell_safe(&"a".repeat(1025)).is_err());
    }
}
