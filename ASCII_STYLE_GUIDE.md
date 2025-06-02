# popup-mcp ASCII Art Style Guide

Since the ImGui renderer doesn't support Unicode/emoji, use classic ASCII art and emoticons!

## Quick Reference

### Emoticons
```
Happy:     :)  :D  ^_^  \\o/
Sad:       :(  :'(  T_T  ;_;
Confused:  :?  o_O  O.o  ???
Angry:     >:(  >.<  -_-
Love:      <3  <3<3  ~<3~
Cool:      B)  8)  BD
Wink:      ;)  ;D  ~_^
Surprised: :O  :o  O_O  0_0
```

### Status Indicators
```
Success:   [OK]  [+]  [✓]  (/)  :)
Error:     [X]   [-]  [!]   (X)  :(
Warning:   /!\   [!]  <!>   (!)  
Info:      [i]   (?)  [?]   ...
Loading:   ...   [ ]  [~]   <->
```

### Arrows & Pointers
```
Right:     ->  -->  ==>  >>>  ▶
Left:      <-  <--  <==  <<<  ◀
Up:        ^   /\   ^^^  
Down:      v   \/   vvv
Both:      <-> <=>  <--> 
```

### Emphasis & Decoration
```
Stars:     *  **  ***  (*)  [*]
Bullets:   •  ·  -  *  >  >>
Boxes:     [ ]  [x]  [v]  { }  < >
Lines:     ---  ===  ***  ___  ~~~
Emphasis:  *text*  **text**  ==text==  |text|  
```

### ASCII Art Headers

#### Box Style
```
+--------------+
|   WARNING    |
+--------------+

+--[ALERT]-----+
| System Error |
+--------------+

╔═══════════╗
║  CRITICAL  ║
╚═══════════╝
```

#### Banner Style
```
=== NEURAL SPIKE ===

*** FOG PROTOCOL ***

>>> SYSTEM CHECK <<<

--- Status Report ---
```

#### Dividers
```
=====================================
-------------------------------------
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
*************************************
. . . . . . . . . . . . . . . . . . .
```

### ASCII Art Icons

#### Brain/Mind
```
Simple:    {@}  {o}  [@]  (%)
Complex:   
    .---.
   /     \
  | () () |
   \  ~  /
    '---'
```

#### Lightning/Energy
```
Simple:    /|\  ^\  |/  ⚡
Complex:
    \  |  /
     \ | /
      \|/
      /|\
     / | \
```

#### Fire
```
Simple:    ^^^  /|\  ^v^
Complex:
     /\
    /  \
   / /\ \
  /_/  \_\
```

#### Heart
```
Simple:    <3  <3<3  {♥}
Complex:
   /\_/\
  <  3  >
   \/ \/
```

### Progress Bars
```
[=====>    ] 50%
[■■■■■□□□□□] 50%
<********--> 80%
{oooooo....} 60%
```

### Special Effects

#### Glitch/Corruption
```
S¥st3m_3rr0r
C0GN!T!V3_CR@SH
F0G_PR0T0C0L
```

#### Emphasis Frames
```
!!!!! ALERT !!!!!
***** URGENT *****
<<<<< NOTE >>>>>
))))) PING (((((
```

## Example Popups

### Neural Interface Style
```
popup "[!] NEURAL SPIKE" [
    text "=== FOG PROTOCOL ACTIVATED ==="
    text ""
    text "Cognitive load: [■■■■■■■□□□] 70%"
    text "Energy reserve: [■■□□□□□□□□] 20%"
    text ""
    text "/!\ INTERVENTION REQUIRED /!\"
    checkbox ">>> Acknowledge alert"
    buttons ["[ENGAGE]", "[DEFER]"]
]
```

### Retro Terminal Style
```
popup "SYSTEM_ALERT" [
    text "+-----------------------+"
    text "| CRITICAL_STATE_DETECT |"
    text "+-----------------------+"
    text ""
    text "> Status: DEGRADED"
    text "> Action: REQUIRED"
    text "> Time:   NOW"
    text ""
    choice "SELECT_PROTOCOL" ["[1] RESET", "[2] CONTINUE", "[3] ABORT"]
    buttons ["EXECUTE", "CANCEL"]
]
```

### Friendly Assistant Style
```
popup "Hey there! :)" [
    text "\\o/ Quick check-in time! \\o/"
    text ""
    text "How are you feeling?"
    slider "Energy" 0..10 default=5
    text ""
    text "Need anything? :)"
    checkbox "[] Water break"
    checkbox "[] Stretch session"
    checkbox "[] Fresh air"
    buttons ["Thanks! :D", "Later ^_^"]
]
```

## Tips

1. **Test your ASCII** - What looks good in monospace might not work in proportional fonts
2. **Keep it simple** - Complex ASCII art can be hard to read at small sizes
3. **Use consistent style** - Pick a theme (retro, friendly, technical) and stick to it
4. **Mind the spacing** - ASCII art needs proper spacing to look right
5. **Fallback gracefully** - If in doubt, use simple text indicators like [!] or (*)

Remember: We're going for that classic terminal/BBS aesthetic! Think 1980s computer interfaces, MUDs, and IRC. Keep it retro! 

                              _____
                             |     |
                             | :-) |
                             |_____|
                              |||||
                              |||||
