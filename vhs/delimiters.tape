# TSV and custom delimiters
Output vhs/delimiters.gif

Set FontSize 16
Set Width 900
Set Height 400
Set Theme "Catppuccin Mocha"
Set TypingSpeed 40ms
Set Padding 20

Type "# TSV input with --tsv"
Enter
Sleep 500ms

Type `printf "Alice\t30\nBob\t25\n" | csvfmt --tsv "{1}: {2}"`
Enter
Sleep 2s

Type "# Custom delimiter with -d"
Enter
Sleep 500ms

Type `echo "Alice;30;Berlin" | csvfmt -d ';' "{1} lives in {3}"`
Enter
Sleep 2s
