# ⚙️ **Omni Configuration Guide**

This guide explains the fields available in `OmniConfig` and how to customize them.

## 📄 Config File Location

The config file is located at `~/.config/omni/config.yaml` by default. When Omni first runs it will create this file with sensible defaults.

## 🗂️ **General Settings**

| Field | Type | Description |
|-------|------|-------------|
| `auto_update` | `bool` | Automatically run `omni update` before installing packages. |
| `parallel_installs` | `bool` | Allow concurrent package installations when supported. |
| `max_parallel_jobs` | `usize` | Maximum number of parallel install jobs. |
| `confirm_installs` | `bool` | Ask for confirmation before installing packages. |
| `log_level` | `string` | Logging verbosity (`error`, `warn`, `info`, `debug`, `trace`). |
| `fallback_enabled` | `bool` | Use the next package manager in the list if the preferred one fails. |

## 📦 **Package Manager Settings**

| Field | Type | Description |
|-------|------|-------------|
| `preferred_order` | `list<string>` | Order to try package managers. |
| `disabled_boxes` | `list<string>` | Package managers to completely disable. |
| `apt_options` | `list<string>` | Extra flags passed to `apt`. |
| `dnf_options` | `list<string>` | Extra flags passed to `dnf`. |
| `pacman_options` | `list<string>` | Extra flags passed to `pacman`. |
| `snap_options` | `list<string>` | Extra flags passed to `snap`. |
| `flatpak_options` | `list<string>` | Extra flags passed to `flatpak`. |

## 🔒 **Security Settings**

| Field | Type | Description |
|-------|------|-------------|
| `verify_signatures` | `bool` | Ensure downloaded packages have valid signatures. |
| `verify_checksums` | `bool` | Check file checksums after download. |
| `allow_untrusted` | `bool` | Permit installing packages that fail signature or checksum verification. |
| `check_mirrors` | `bool` | Test mirror availability before downloading. |
| `signature_servers` | `list<string>` | GPG key servers to query for package signatures. |
| `trusted_keys` | `list<string>` | Additional GPG key fingerprints to trust implicitly. |
| `interactive_prompts` | `bool` | Prompt before performing risky operations. |

## 🎨 **UI Settings**

| Field | Type | Description |
|-------|------|-------------|
| `show_progress` | `bool` | Display progress bars during operations. |
| `use_colors` | `bool` | Colorize output when supported. |
| `compact_output` | `bool` | Reduce whitespace in CLI output. |
| `gui_theme` | `string` | GUI theme: `dark`, `light`, or `auto`. |

## ✏️ **Examples**

### Disable a package manager

```yaml
boxes:
  disabled_boxes:
    - snap
```

### Enable a package manager

```yaml
boxes:
  disabled_boxes: []     # remove managers from this list to enable them
```

### Require signature verification

```yaml
security:
  verify_signatures: true
  allow_untrusted: false
```

### Allow installing unsigned packages

```yaml
security:
  verify_signatures: false
  allow_untrusted: true
```

