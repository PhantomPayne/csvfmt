# Basic field substitution
Output vhs/basic.gif

Set FontSize 16
Set Width 800
Set Height 300
Set Theme "Catppuccin Mocha"
Set TypingSpeed 40ms
Set Padding 20

Type "# Basic substitution – fields are {1}, {2}, etc."
Enter
Sleep 500ms

Type `echo "Alice,30" | csvfmt "Hello {1}, you are {2} years old."`
Enter
Sleep 2s

Type `echo "Bob,25" | csvfmt "{1} is {2} years old."`
Enter
Sleep 2s
