use AppleScript version "2.4"
use scripting additions
use framework "Foundation"
use framework "AppKit"

property statusItem : missing value

on run
    set statusItem to current application's NSStatusBar's systemStatusBar's statusItemWithLength:(current application's NSVariableStatusItemLength)
    statusItem's setTitle:"📅"
    statusItem's setHighlightMode:true
    
    set newMenu to current application's NSMenu's alloc()'s init()
    newMenu's addItemWithTitle:"Open Calcurse" action:"openCalcurse:" keyEquivalent:""
    newMenu's addItemWithTitle:"Quit" action:"terminate:" keyEquivalent:"q"
    
    statusItem's setMenu:newMenu
end run

on openCalcurse:sender
    do shell script "open -a Terminal.app " & quoted form of "/Users/jack/chonker5/calcurse-tmux.sh"
end openCalcurse: