# Named fields with -H header mode
Output vhs/named-fields.gif

Set FontSize 16
Set Width 900
Set Height 350
Set Theme "Catppuccin Mocha"
Set TypingSpeed 40ms
Set Padding 20

Type "# Named fields using -H (header row)"
Enter
Sleep 500ms

Type `printf "first,last,email\nAlice,Smith,alice@example.com\nBob,Jones,bob@example.com\n" | csvfmt -H "Dear {first} {last} <{email}>,"`
Enter
Sleep 3s
