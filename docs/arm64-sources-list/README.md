# Fix: arm64 Ubuntu Container Sources List

## Problem
On arm64 systems (Apple Silicon, AWS Graviton, etc.), Ubuntu container workspaces
failed during init script execution with apt-get 404 errors:

```
E: Failed to fetch http://archive.ubuntu.com/ubuntu/dists/noble/main/binary-arm64/Packages
  404  Not Found
```

## Root Cause
The `base` init script in the library was writing `archive.ubuntu.com` to
`/etc/apt/sources.list`, but `archive.ubuntu.com` doesn't host arm64 packages.
Arm64 packages are hosted at `ports.ubuntu.com/ubuntu-ports`.

## Solution
Updated the default library to use a fork that includes the arm64 fix:

**Library:** `https://github.com/writepavel/sandboxed-library-template.git`

The `init-script/base/SCRIPT.sh` in the library now detects architecture:

```bash
APT_ARCH=$(dpkg --print-architecture 2>/dev/null || echo "amd64")
if [ "$APT_ARCH" = "arm64" ]; then
  # arm64 uses ports.ubuntu.com
  cat > /etc/apt/sources.list <<'EOF'
deb http://ports.ubuntu.com/ubuntu-ports noble main restricted universe multiverse
deb http://ports.ubuntu.com/ubuntu-ports noble-updates main restricted universe multiverse
deb http://ports.ubuntu.com/ubuntu-ports noble-security main restricted universe multiverse
EOF
else
  # amd64 and other architectures use archive.ubuntu.com
  cat > /etc/apt/sources.list <<'EOF'
deb http://archive.ubuntu.com/ubuntu noble main restricted universe multiverse
deb http://archive.ubuntu.com/ubuntu noble-updates main restricted universe multiverse
deb http://archive.ubuntu.com/ubuntu noble-security main restricted universe multiverse
EOF
fi
```

## Files Changed
- `src/settings.rs` - Default library_remote URL
- `src/config.rs` - Documentation comment
- `.env.example` - Updated comment

## Related
- Library fork: https://github.com/writepavel/sandboxed-library-template
