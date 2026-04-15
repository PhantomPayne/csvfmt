# Default values with {N:default}
Output vhs/defaults.gif

Set FontSize 16
Set Width 900
Set Height 350
Set Theme "Catppuccin Mocha"
Set TypingSpeed 40ms
Set Padding 20

Type "# Default values – provide a fallback when a field is empty"
Enter
Sleep 500ms

Type `echo "Alice,," | csvfmt "{1} is {2:unknown} years old"`
Enter
Sleep 2s

Type `echo "Alice,30," | csvfmt "{1} is {2:unknown} years old"`
Enter
Sleep 2s
