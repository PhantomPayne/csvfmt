# Practical use cases
Output vhs/practical.gif

Set FontSize 16
Set Width 1000
Set Height 500
Set Theme "Catppuccin Mocha"
Set TypingSpeed 40ms
Set Padding 20

Type "# Generate SQL INSERT statements from CSV"
Enter
Sleep 500ms

Type `printf "Alice,30\nBob,25\nCharlie,35\n" | csvfmt "INSERT INTO users (name, age) VALUES ('{1}', {2});"`
Enter
Sleep 3s

Type "# Generate markdown links from CSV"
Enter
Sleep 500ms

Type `printf "Google,https://google.com\nGitHub,https://github.com\n" | csvfmt "[{1}]({2})"`
Enter
Sleep 3s

Type "# Skip blank rows & trim whitespace"
Enter
Sleep 500ms

Type `printf " Alice , 30 \n\n , \n Bob , 25 \n" | csvfmt --trim --skip-empty "{1}: {2}"`
Enter
Sleep 3s
