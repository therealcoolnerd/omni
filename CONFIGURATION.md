<div align="center">

```
 ██████╗ ███╗   ███╗███╗   ██╗██╗
██╔═══██╗████╗ ████║████╗  ██║██║
██║   ██║██╔████╔██║██╔██╗ ██║██║
██║   ██║██║╚██╔╝██║██║╚██╗██║██║
╚██████╔╝██║ ╚═╝ ██║██║ ╚████║██║
 ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═══╝╚═╝
```

**Configuration Reference**

</div>

---

## Config File Location

| Platform | Path |
|----------|------|
| Linux    | `~/.config/omni/config.yaml` |
| macOS    | `~/Library/Application Support/omni/config.yaml` |
| Windows  | `%APPDATA%\omni\config.yaml` |

The file is created automatically on first run with defaults. You can view, edit, or reset it at any time:

```bash
omni config show      # Print the active configuration
omni config edit      # Open in $EDITOR (Linux/macOS) or the default editor
omni config reset     # Restore all settings to defaults
```

---

## General Settings

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `auto_update` | bool | `false` | Refresh package lists before installing. |
| `parallel_installs` | bool | `true` | Allow concurrent installs when supported. |
| `max_parallel_jobs` | usize | `4` | Maximum concurrent jobs (clamped 1–16). |
| `confirm_installs` | bool | `true` | Prompt before installing packages. |
| `log_level` | string | `"info"` | Logging verbosity: `error` · `warn` · `info` · `debug` · `trace`. |
| `fallback_enabled` | bool | `true` | Try the next manager in the preferred list if one fails. |

---

## Package Manager Settings

| Field | Type | Description |
|-------|------|-------------|
| `preferred_order` | list | Order in which package managers are tried. |
| `disabled_boxes` | list | Package managers to skip completely. |
| `apt_options` | list | Extra flags appended to every `apt` call. |
| `dnf_options` | list | Extra flags appended to every `dnf` call. |
| `pacman_options` | list | Extra flags appended to every `pacman` call. |
| `snap_options` | list | Extra flags appended to every `snap` call. |
| `flatpak_options` | list | Extra flags appended to every `flatpak` call. |

**Default preferred order (Linux):** `apt` → `dnf` → `pacman` → `flatpak` → `snap` → `appimage`

On Windows, Omni automatically selects from `winget` → `choco` → `scoop`. On macOS from `brew` → `mas`. The preferred_order list is used as a tiebreaker when a package is found in multiple managers.

---

## Security Settings

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `verify_signatures` | bool | `true` | Require valid GPG signatures on packages. |
| `verify_checksums` | bool | `true` | Verify SHA-256 checksums on downloaded files. |
| `allow_untrusted` | bool | `false` | Permit packages that fail signature or checksum checks. |
| `check_mirrors` | bool | `true` | Test mirror availability before downloading. |
| `signature_servers` | list | (see below) | GPG keyservers used for signature lookup. |
| `trusted_keys` | list | `[]` | Additional GPG fingerprints to trust. |
| `interactive_prompts` | bool | `true` | Prompt before risky operations. |

Default signature servers:
- `keyserver.ubuntu.com`
- `keys.openpgp.org`
- `pgp.mit.edu`

---

## UI Settings

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `show_progress` | bool | `true` | Display progress bars during operations. |
| `use_colors` | bool | `true` | Colorize terminal output. |
| `compact_output` | bool | `false` | Reduce whitespace in output. |
| `gui_theme` | string | `"dark"` | Theme for the GUI: `dark` · `light` · `auto`. |

---

## Examples

### Disable a specific package manager

```yaml
boxes:
  disabled_boxes:
    - snap
```

### Set a custom preferred order

```yaml
boxes:
  preferred_order:
    - flatpak
    - apt
    - appimage
```

### Require strict signature verification

```yaml
security:
  verify_signatures: true
  verify_checksums: true
  allow_untrusted: false
```

### Allow unsigned packages (lab / dev environments)

```yaml
security:
  verify_signatures: false
  allow_untrusted: true
```

### Reduce log verbosity

```yaml
general:
  log_level: warn
```

### Disable install confirmation prompts

```yaml
general:
  confirm_installs: false
```

### Full example config

```yaml
general:
  auto_update: false
  parallel_installs: true
  max_parallel_jobs: 4
  confirm_installs: true
  log_level: info
  fallback_enabled: true

boxes:
  preferred_order:
    - apt
    - dnf
    - pacman
    - flatpak
    - snap
    - appimage
  disabled_boxes: []
  apt_options:
    - "-y"
  dnf_options:
    - "-y"
  pacman_options:
    - "--noconfirm"
  snap_options: []
  flatpak_options:
    - "-y"

security:
  verify_signatures: true
  verify_checksums: true
  allow_untrusted: false
  check_mirrors: true
  signature_servers:
    - keyserver.ubuntu.com
    - keys.openpgp.org
    - pgp.mit.edu
  trusted_keys: []
  interactive_prompts: true

ui:
  show_progress: true
  use_colors: true
  compact_output: false
  gui_theme: dark
```
