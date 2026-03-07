<div align="center">

```
 ██████╗ ███╗   ███╗███╗   ██╗██╗
██╔═══██╗████╗ ████║████╗  ██║██║
██║   ██║██╔████╔██║██╔██╗ ██║██║
██║   ██║██║╚██╔╝██║██║╚██╗██║██║
╚██████╔╝██║ ╚═╝ ██║██║ ╚████║██║
 ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═══╝╚═╝
```

**User Guide**

</div>

---

## Install

```bash
omni install <package>
omni install <package> --box-type <manager>   # Force a specific manager
omni install --from packages.yaml             # From a manifest
```

Omni detects the current platform, picks the best available manager, and runs the install. The result is logged to the local SQLite database.

**Examples:**

```bash
omni install git
omni install firefox --box-type flatpak
omni install --from dev-environment.yaml
```

---

## Remove

```bash
omni remove <package>
omni remove <package> --box-type <manager>
```

**Examples:**

```bash
omni remove firefox
omni remove vlc --box-type snap
```

---

## Search

```bash
omni search <query>
omni search <query> --limit <n>              # Default limit: 20
```

Searches all available package managers on the current platform and deduplicates results, preferring installed packages in the ranking.

**Examples:**

```bash
omni search browser
omni search "text editor" --limit 10
```

---

## Info

```bash
omni info <package>
omni info <package> --box-type <manager>
```

Fetches package metadata from the specified or detected package manager. Results are cached for one hour.

---

## Update

```bash
omni update                                  # List available updates
omni update <package>                        # Update a specific package
omni update --all                            # Apply all updates
omni update --all --refresh                  # Refresh repos then update
```

---

## List

```bash
omni list                                    # All installed packages
omni list --box-type <manager>               # Filter by manager
omni list --detailed                         # Include version and install date
```

---

## History and Rollback

```bash
omni history show                            # Last 20 operations
omni history show --limit 50                 # Custom limit
omni history undo                            # Reverse the last operation
```

Every install, update, and removal is recorded with package name, box type, version, timestamp, and status.

---

## Snapshots

Capture the full set of installed packages at a point in time and restore to it later.

```bash
omni snapshot create <name>
omni snapshot create <name> --description "Notes about this snapshot"
omni snapshot list
omni snapshot revert <name-or-id>
```

**Typical workflow:**

```bash
omni snapshot create "pre-experiment"
omni install some-new-tool
# Something breaks
omni snapshot revert "pre-experiment"
```

---

## Dependency Resolution

Inspect what would be installed before committing.

```bash
omni resolve <package>
omni resolve <package> --detailed            # Show dependency tree
omni resolve <package> --box-type apt        # Constrain to one manager
```

---

## Security Verification

```bash
omni verify <file> --checksum <sha256>
omni verify <file> --signature <sig-file-or-url>
omni verify <file> --checksum <sha256> --box-type apt
```

Trust levels returned: **Trusted** → **Valid** → **Unsigned** → **Untrusted**

---

## Hardware Detection

```bash
omni hardware detect                         # Identify hardware
omni hardware install                        # Auto-install optimal drivers
omni hardware vendor <name>                  # Vendor-specific drivers
```

Supported vendors: Dell · HP/HPE · Supermicro · Lenovo · Cisco UCS

---

## Repository Management

```bash
omni repository add <url>                    # Add a repository
omni repository add <url> --repo-type ppa --key-url <key-url>
omni repository remove <name>               # Remove a repository
omni repository list                         # List configured repositories
omni repository refresh                      # Refresh metadata
```

---

## Web Dashboard

```bash
omni web --port 3000
# Navigate to http://127.0.0.1:3000
```

The server binds on localhost only. REST API:

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/system/info` | OS, architecture, active managers |
| GET | `/api/packages/installed` | Tracked installed packages |
| GET | `/api/packages/search?q=<q>&limit=<n>` | Live search |
| POST | `/api/packages/install` | `{"package":"...", "box_type":"..."}` |
| POST | `/api/packages/remove` | `{"package":"...", "box_type":"..."}` |

---

## Configuration

```bash
omni config show                             # Print active config
omni config edit                             # Open in $EDITOR
omni config reset                            # Restore defaults
```

Config file location: `~/.config/omni/config.yaml`

See [CONFIGURATION.md](../CONFIGURATION.md) for all fields and examples.

---

## Mock Mode

Run any command without touching the system. Safe for CI, scripting, and testing.

```bash
omni --mock install firefox
omni --mock remove curl
omni --mock update --all
```

---

## GUI

```bash
omni gui
# Requires: cargo build --features gui
```

---

## Verbose Output

```bash
omni --verbose install firefox               # Extra log output to terminal
```

Log level can also be set in config (`log_level: debug`) for persistent verbosity.

---

## Manifest File Format

```yaml
version: "1.0"
name: "Development Environment"
description: "Standard dev tools"

packages:
  - name: git
  - name: curl
  - name: nodejs
  - name: docker
```

Install with:

```bash
omni install --from dev-environment.yaml
```

---

## Troubleshooting

**Package not found**

```bash
omni repository refresh
omni search <partial-name>              # Try a broader search term
```

**Permission denied**

Some package managers require elevated privileges. Run with `sudo` on Linux/macOS or an elevated terminal on Windows when necessary.

**Manager-specific failure with fallback**

If `fallback_enabled: true` in config (default), Omni will try the next manager in `preferred_order` automatically. Check `omni history show` to see which manager succeeded.

---

[Back to README](../README.md) · [Installation Guide](installation-guide.md) · [Configuration](../CONFIGURATION.md)
