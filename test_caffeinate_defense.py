#!/usr/bin/env python3
"""Test the caffeinate defense system"""

import subprocess
import time
import os
import signal

def test_caffeinate():
    """Test starting and stopping caffeinate"""
    print("🛡️ Testing CAFFEINATE DEFENSE SYSTEM...")
    
    # Start caffeinate
    try:
        process = subprocess.Popen(
            ['caffeinate', '-diu'],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL
        )
        print(f"✅ Caffeinate started with PID: {process.pid}")
        
        # Check if it's running
        time.sleep(1)
        if process.poll() is None:
            print("✅ Caffeinate is running and defending against sleep!")
            
            # Check process details
            result = subprocess.run(['ps', '-p', str(process.pid), '-o', 'comm='], 
                                  capture_output=True, text=True)
            if result.stdout.strip() == 'caffeinate':
                print("✅ Process verified as caffeinate")
            
            # Let it run for a bit
            print("⏱️ Running for 5 seconds...")
            time.sleep(5)
            
            # Stop it
            process.terminate()
            print("🛑 Caffeinate terminated")
            
            # Verify it stopped
            time.sleep(1)
            if process.poll() is not None:
                print("✅ Caffeinate successfully stopped")
        else:
            print("❌ Caffeinate failed to start")
            
    except Exception as e:
        print(f"🐹 *cough* Error: {e}")

if __name__ == "__main__":
    test_caffeinate()