# Conditional blocks with {?N:text}
Output vhs/conditionals.gif

Set FontSize 16
Set Width 900
Set Height 400
Set Theme "Catppuccin Mocha"
Set TypingSpeed 40ms
Set Padding 20

Type "# Conditional blocks – include text only when a field has a value"
Enter
Sleep 500ms

Type `echo "Alice,30,Engineer" | csvfmt "Hello {1}{?3:, job: {3}}"`
Enter
Sleep 2s

Type `echo "Alice,30," | csvfmt "Hello {1}{?3:, job: {3}}"`
Enter
Sleep 2s

Type "# ↑ The job part is omitted when field 3 is empty"
Enter
Sleep 2s
