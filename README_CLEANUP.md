# Continuous Cleanup System for chonker5.rs

This directory contains three cleanup systems for maintaining code quality as you develop:

## 1. 🔄 Basic Continuous Cleanup (`continuous_cleanup.sh`)

**Purpose**: Real-time monitoring with automatic quality checks

**Features**:
- Monitors chonker5.rs for changes every 5 seconds
- Runs compilation checks
- Executes clippy linting
- Auto-formats code with `cargo fmt`
- Checks for `unwrap()` calls and TODOs
- Runs tests automatically
- Logs all activities to `cleanup.log`

**Usage**:
```bash
./continuous_cleanup.sh
```

## 2. 🤖 Smart Cleanup with Sub-Agent Integration (`smart_cleanup.sh`)

**Purpose**: Intelligent cleanup with escalation to AI assistance

**Features**:
- Quick cleanup on every change (formatting, basic fixes)
- Deep cleanup every 10 changes
- Triggers AI sub-agent for large changes (>100 lines)
- Tracks change history
- Progressive cleanup strategy

**Usage**:
```bash
./smart_cleanup.sh
```

The AI sub-agent will be triggered automatically when:
- More than 100 lines are changed at once
- Complex refactoring is detected
- Manual trigger via `.cleanup_request.md`

## 3. 🛡️ Cleanup Daemon (`cleanup_daemon.sh`)

**Purpose**: Background service for passive monitoring

**Features**:
- Runs as a background process
- Silent operation with logging
- Automatic formatting fixes
- Continuous quality monitoring
- Minimal resource usage

**Commands**:
```bash
./cleanup_daemon.sh start    # Start the daemon
./cleanup_daemon.sh stop     # Stop the daemon
./cleanup_daemon.sh status   # Check status
./cleanup_daemon.sh logs     # View live logs
./cleanup_daemon.sh restart  # Restart daemon
```

## Choosing the Right System

| Use Case | Recommended System |
|----------|-------------------|
| Active development with immediate feedback | `continuous_cleanup.sh` |
| Long coding sessions with AI assistance | `smart_cleanup.sh` |
| Background quality assurance | `cleanup_daemon.sh` |
| CI/CD integration | `quality_check.sh` (already created) |

## Quick Start

1. **For interactive development**:
   ```bash
   ./continuous_cleanup.sh
   ```

2. **For AI-assisted development**:
   ```bash
   ./smart_cleanup.sh
   ```

3. **For background monitoring**:
   ```bash
   ./cleanup_daemon.sh start
   ```

## Integration with Development Workflow

1. **Pre-commit Hook**: Use `quality_check.sh` before commits
2. **Live Monitoring**: Run `continuous_cleanup.sh` in a terminal
3. **Background QA**: Keep `cleanup_daemon.sh` running
4. **Major Changes**: Let `smart_cleanup.sh` trigger AI cleanup

## Files Created

- `continuous_cleanup.sh` - Real-time monitoring with immediate feedback
- `smart_cleanup.sh` - Intelligent cleanup with AI escalation
- `cleanup_daemon.sh` - Background daemon service
- `cleanup.log` - Activity log for all cleanup operations
- `.change_counter` - Tracks changes for smart cleanup
- `.last_cleanup_hash` - Tracks file state
- `.cleanup_request.md` - AI sub-agent trigger file

## Customization

Each script can be customized by editing the configuration variables at the top:
- `CLEANUP_INTERVAL` - How often to check for changes
- `CHANGES_THRESHOLD` - Number of changes before deep cleanup
- `COMPLEXITY_THRESHOLD` - Lines changed before AI assistance

## Stopping the Services

- **Continuous/Smart Cleanup**: Press `Ctrl+C`
- **Daemon**: Run `./cleanup_daemon.sh stop`