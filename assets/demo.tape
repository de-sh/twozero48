Output assets/demo.gif

# Capture the release binary because debug-mode terminal rendering can be slow
# enough to show visible tearing in the recording.
Require target/release/twozero48

Set Shell "zsh"
Set Theme "OneDark"
Set Width 1000
Set Height 930
Set LetterSpacing 0
Set Padding 4
# 50 FPS keeps the slide animation smooth while staying reasonable for a GIF.
Set Framerate 50
Set TypingSpeed 1ms

Hide
Type "env -u NO_COLOR COLORTERM=truecolor TERM=xterm-256color ./target/release/twozero48"
Enter
Sleep 2s

Show
Sleep 1.5s

Right
Sleep 750ms
Down
Sleep 750ms
Left
Sleep 750ms
Up
Sleep 750ms
Right
Sleep 750ms
Down
Sleep 1000ms
Left
Sleep 750ms
Up
Sleep 750ms
Right
Sleep 750ms
Down
Sleep 750ms
Left
Sleep 750ms
Up
Sleep 750ms
Right
Sleep 750ms
Down
Sleep 750ms
Left
Sleep 750ms
Up
Sleep 1000ms
Left
Sleep 750ms
Up
Sleep 750ms
Right
Sleep 750ms
Down
Sleep 750ms

Sleep 1.5s
Hide
Escape
Sleep 250ms
