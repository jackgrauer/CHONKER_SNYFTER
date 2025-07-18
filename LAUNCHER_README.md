# CHONKER SNYFTER Launcher System

A robust launcher system for managing the CHONKER SNYFTER process in the background.

## Prerequisites

```bash
# Activate virtual environment and install dependencies
source venv/bin/activate
pip install psutil
```

## Usage

### Using the bash wrapper (recommended)

```bash
# Launch CHONKER in background
./launch_chonker.sh launch

# Check status
./launch_chonker.sh status

# Stop CHONKER
./launch_chonker.sh stop

# Clean up old log files
./launch_chonker.sh cleanup

# Launch in foreground (for debugging)
./launch_chonker.sh launch --foreground
```

### Using the Python launcher directly

```bash
# Make sure virtual environment is activated
source venv/bin/activate

# Launch
python chonker_launcher.py launch

# With custom log location
python chonker_launcher.py launch --log-file /path/to/custom.log

# With custom PID file
python chonker_launcher.py launch --pid-file /path/to/custom.pid
```

## Features

- **Multiple instance prevention**: Won't start if CHONKER is already running
- **PID file management**: Tracks process ID and cleans up stale files
- **Log rotation**: Automatically rotates logs when they exceed size limit (default: 10MB)
- **Clean shutdown**: Sends SIGTERM for graceful shutdown, SIGKILL if needed
- **Process verification**: Verifies the running process is actually CHONKER
- **Status tracking**: Shows PID and uptime for running processes

## File Locations

Default locations:
- **PID file**: `/tmp/chonker.pid`
- **Log file**: `/tmp/chonker.log`
- **Process info**: `/tmp/chonker.info` (JSON metadata)

## Exit Codes

- `0`: Success
- `1`: Failure (check output message for details)

## Troubleshooting

### "psutil package is required"
Install psutil: `pip install psutil`

### "CHONKER already running"
Check status with `./launch_chonker.sh status` or stop with `./launch_chonker.sh stop`

### Stale PID file
The launcher automatically cleans up stale PID files when checking status or launching.