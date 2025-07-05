/// Branding and visual elements for Omni
use std::fmt;

pub struct OmniBranding;

impl OmniBranding {
    /// Main OMNI ASCII art logo
    pub fn logo() -> &'static str {
        r#"
██╗  ██╗ ███╗   ███╗ ███╗   ██╗ ██╗
██║  ██║ ████╗ ████║ ████╗  ██║ ██║
██║  ██║ ██╔████╔██║ ██╔██╗ ██║ ██║
██║  ██║ ██║╚██╔╝██║ ██║╚██╗██║ ██║
╚██████╔╝ ██║ ╚═╝ ██║ ██║ ╚████║ ██║
 ╚═════╝  ╚═╝     ╚═╝ ╚═╝  ╚═══╝ ╚═╝
                                    
    Universal Package Manager    
"#
    }

    /// Compact ASCII logo for headers
    pub fn compact_logo() -> &'static str {
        r#"
 ██████╗ ███╗   ███╗███╗   ██╗██╗
██╔═══██╗████╗ ████║████╗  ██║██║
██║   ██║██╔████╔██║██╔██╗ ██║██║
██║   ██║██║╚██╔╝██║██║╚██╗██║██║
╚██████╔╝██║ ╚═╝ ██║██║ ╚████║██║
 ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═══╝╚═╝
"#
    }

    /// Small ASCII for inline use
    pub fn mini_logo() -> &'static str {
        r#"
 ▄▄▄▄▄▄▄ ▄▄▄   ▄▄▄ ▄▄    ▄ ▄▄▄ 
█       █   █▄█   █  █  █ █   █
█   ▄   █       █   █   █▄█   █
█  █ █  █       █   █    ▄    █
█  █▄█  █ ██▄██ █   █   █ █   █
█       █       █   █   █ █   █
█▄▄▄▄▄▄▄█▄▄▄▄▄▄▄█▄▄▄█▄▄▄█ █▄▄▄█
"#
    }

    /// Package box ASCII art
    pub fn package_icon() -> &'static str {
        r#"
    ┌─────────────┐
   ╱│             │╲
  ╱ │    OMNI     │ ╲
 ╱  │   PACKAGE   │  ╲
╱   └─────────────┘   ╲
╲                     ╱
 ╲                   ╱
  ╲                 ╱
   ╲_______________╱
"#
    }

    /// Black background banner with white text
    pub fn welcome_banner() -> String {
        format!(
            "\x1b[40m\x1b[37m{}\x1b[0m",
            r#"
╔══════════════════════════════════════════════════════════════════════════════╗
║                                                                              ║
║  ██████╗ ███╗   ███╗███╗   ██╗██╗    Universal Package Manager             ║
║ ██╔═══██╗████╗ ████║████╗  ██║██║                                          ║
║ ██║   ██║██╔████╔██║██╔██╗ ██║██║    One command for all platforms         ║
║ ██║   ██║██║╚██╔╝██║██║╚██╗██║██║                                          ║
║ ╚██████╔╝██║ ╚═╝ ██║██║ ╚████║██║    Linux • macOS • Windows              ║
║  ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═══╝╚═╝                                          ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
"#
        )
    }

    /// Progress indicators for package operations
    pub fn progress_chars() -> ProgressChars {
        ProgressChars {
            installing: "▓",
            searching: "▒",
            downloading: "░",
            complete: "█",
        }
    }

    /// Color theme constants
    pub fn theme() -> Theme {
        Theme {
            background: "\x1b[40m",      // Black background
            foreground: "\x1b[37m",      // White text
            accent: "\x1b[36m",          // Cyan for highlights
            success: "\x1b[32m",         // Green for success
            error: "\x1b[31m",           // Red for errors
            warning: "\x1b[33m",         // Yellow for warnings
            reset: "\x1b[0m",            // Reset colors
        }
    }
}

pub struct ProgressChars {
    pub installing: &'static str,
    pub searching: &'static str,
    pub downloading: &'static str,
    pub complete: &'static str,
}

pub struct Theme {
    pub background: &'static str,
    pub foreground: &'static str,
    pub accent: &'static str,
    pub success: &'static str,
    pub error: &'static str,
    pub warning: &'static str,
    pub reset: &'static str,
}

impl fmt::Display for OmniBranding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::logo())
    }
}

/// Helper macro for colored output
#[macro_export]
macro_rules! omni_print {
    ($theme:expr, $color:ident, $($arg:tt)*) => {
        print!("{}{}{}", $theme.$color, format!($($arg)*), $theme.reset);
    };
}

/// Helper macro for colored println
#[macro_export]
macro_rules! omni_println {
    ($theme:expr, $color:ident, $($arg:tt)*) => {
        println!("{}{}{}", $theme.$color, format!($($arg)*), $theme.reset);
    };
}