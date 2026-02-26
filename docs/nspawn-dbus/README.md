# Fix: systemd-nspawn D-Bus Registration in Docker

## Problem
Container workspace creation failed in Docker with the error:

```
Failed to open system bus: No such file or directory
```

## Root Cause
systemd-nspawn attempts to register containers with systemd via D-Bus. Docker
containers typically don't run D-Bus, causing the registration to fail and the
container creation to abort.

## Solution
Added D-Bus avoidance flags to systemd-nspawn commands in `src/nspawn.rs`:

```rust
cmd.arg("--register=no");  // Skip registration with systemd
cmd.arg("--keep-unit");    // Don't require systemd unit management
```

These flags are added to both:
- `execute_in_container()` - Used for init script execution
- `execute_in_container_streaming()` - Used for streaming init scripts

## Technical Details

### `--register=no`
Tells systemd-nspawn not to register the container with the systemd machine
registry. This bypasses the D-Bus requirement for container registration.

### `--keep-unit`
Prevents systemd-nspawn from trying to create a new systemd unit for the
container. Useful when running inside an existing service (like Docker).

## Files Changed
- `src/nspawn.rs` - Added flags to nspawn command builder

## Impact
- Container workspaces now work correctly in Docker environments
- No functional impact on container isolation or functionality
- Init scripts can execute without D-Bus dependency

## References
- systemd-nspawn documentation: https://www.freedesktop.org/software/systemd/man/systemd-nspawn.html
