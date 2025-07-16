# 🚀 How to Run CHONKER & SNYFTER

The application is now running! Check your dock/taskbar for the CHONKER & SNYFTER window.

## For Future Launches

### Option 1: Simple Python Script
```bash
source /Users/jack/chonksnyft-env/bin/activate
python run.py
```

### Option 2: Shell Script
```bash
./chonker-snyfter.sh
```

### Option 3: Direct Launch
```bash
source /Users/jack/chonksnyft-env/bin/activate
python chonker_snyfter_enhanced.py
```

### Option 4: Background Launch
```bash
source /Users/jack/chonksnyft-env/bin/activate
nohup python chonker_snyfter_enhanced.py > app.log 2>&1 &
```

## Current Status
✅ The app is currently running (PID: 52926)
✅ Look for the window in your dock or use Cmd+Tab to switch to it
✅ Logs are being written to `app.log`

## To Stop the App
- Use the GUI close button (X)
- Or from terminal: `pkill -f chonker_snyfter`

## Troubleshooting
- If you don't see the window, check `app.log` for errors
- Make sure you're in the virtual environment
- Try minimizing other windows - it might be behind them

Enjoy using CHONKER & SNYFTER! 🐹🐁