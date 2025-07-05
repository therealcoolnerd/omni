/// Branding and visual elements for Omni
/// 
/// This module provides both ASCII art logos for terminal use and
/// an adaptive SVG logo (assets/logo.svg) that automatically adapts
/// to dark/light themes for GUI and web interfaces.
use std::fmt;

pub struct OmniBranding;

impl OmniBranding {
    /// Path to the adaptive SVG logo
    pub fn svg_logo_path() -> &'static str {
        "assets/logo.svg"
    }

    /// Load the adaptive SVG logo content
    /// The original logo includes embedded CSS for automatic dark/light theme adaptation
    pub fn svg_logo() -> Result<String, std::io::Error> {
        std::fs::read_to_string(Self::svg_logo_path())
    }

    /// Get SVG logo with theme class applied
    /// The original adaptive logo includes embedded CSS for automatic theme detection,
    /// but manual theme classes can be added for programmatic control
    pub fn svg_logo_with_theme(dark_mode: bool) -> Result<String, std::io::Error> {
        let mut svg_content = Self::svg_logo()?;
        let theme_class = if dark_mode { "dark-theme" } else { "light-theme" };
        
        // The original logo already includes adaptive CSS with media queries
        // This function adds explicit theme classes for programmatic control
        if let Some(pos) = svg_content.find("<svg") {
            if let Some(end_pos) = svg_content[pos..].find('>') {
                let insert_pos = pos + end_pos;
                svg_content.insert_str(insert_pos, &format!(" class=\"{}\"", theme_class));
            }
        }
        
        Ok(svg_content)
    }

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

    /// Generate HTML with embedded SVG logo
    pub fn html_with_logo(title: &str, dark_mode: bool) -> Result<String, std::io::Error> {
        let svg_logo = Self::svg_logo_with_theme(dark_mode)?;
        let theme_class = if dark_mode { "dark" } else { "light" };
        
        Ok(format!(r#"<!DOCTYPE html>
<html lang="en" class="{}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{ 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: var(--bg-color, {});
            color: var(--text-color, {});
            margin: 0;
            padding: 20px;
        }}
        .logo-container {{ 
            text-align: center; 
            margin: 20px 0; 
        }}
        .logo-container svg {{ 
            max-width: 200px; 
            height: auto; 
        }}
        .dark {{ --bg-color: #000000; --text-color: #ffffff; }}
        .light {{ --bg-color: #ffffff; --text-color: #000000; }}
    </style>
</head>
<body>
    <div class="logo-container">
        {}
    </div>
    <h1>{}</h1>
</body>
</html>"#, 
            theme_class,
            title,
            if dark_mode { "#000000" } else { "#ffffff" },
            if dark_mode { "#ffffff" } else { "#000000" },
            svg_logo,
            title
        ))
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
    
    /// Dark theme for SVG/GUI
    pub fn dark_theme() -> SvgTheme {
        SvgTheme {
            bg_color: "#000000",
            text_color: "#ffffff", 
            accent_color: "#00bcd4",
            box_color: "#333333",
        }
    }
    
    /// Light theme for SVG/GUI
    pub fn light_theme() -> SvgTheme {
        SvgTheme {
            bg_color: "#ffffff",
            text_color: "#000000",
            accent_color: "#0077aa", 
            box_color: "#f0f0f0",
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

pub struct SvgTheme {
    pub bg_color: &'static str,
    pub text_color: &'static str,
    pub accent_color: &'static str,
    pub box_color: &'static str,
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