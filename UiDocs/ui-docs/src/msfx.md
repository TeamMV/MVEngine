# MSFX (Mvengine Shape Format DeluXe) Tool
This is a remake of the popular MVEngine shape anguage MSF. This remake uses the .msfx file extension.

## Overview 😱🚨❗

With this deluxe ✨💎👑🥂, fancy 🎩💅🌈🕶️, special 🌟🎁🎀🍾, elegant 🕊️🎻🧼💃, preformant 🚀💨📈⚙️, blazingly fast 🔥⚡🚒💨 (and memory safe too 🦀🛡️🧠📦🔒), this elaborate 🧠🧵📐🎛️🎯 language empowers you to craft arbitrary shapes 🔷🔺🌀🔶🌐🧊 ready to be wielded in the UI system 🖼️📲🧩🎛️💻📐 or summoned into other mighty parts of the engine 🛠️🏗️🎮💾🎯💣.

---

## Syntax
Here we discuss the syntactical shenanigans of this deluxe, fancy, special, elegant, preformant, blazingly fast ⚡🔥💨🏎️🚀🌠💥🌋 and memory safe 🧠🛡️🔒🧬🚫👾🔍🧯💉🩹‼️🆘, elaborate language.

### Comments
Because we are too lazy to make the lexer support multiline comments, you have to bear with the good old single line ones.
They are much more superior as well (source: trust me bro)
```
//This is a demonstration on how to create a so-called "comment"
```

### Variables
Variables can be defined using the `let` keyword.
```
let a = 1.0;
```
Variable shadowing is supported, so the following is perfectly fine:
```
let a = 1.0;
smth[arg: a];
let a = 2.0;
smth[arg: a];
```

### Blocks
Certain blocks of code can be encapsulated by using the [colon symbol](https://simple.wikipedia.org/wiki/Colon_(punctuation))
and the `end` keyword.
```
if true:
    //...
end;
```

### Loops
There is a for and while loop built into the language natively.
```
while expr:
    //...
end;

for i in begin[start: 0, end: 5, step: 1]:
    //...
end;
```
The begin function for for-loops requires a start and end parameter, but the step defaults to 1.
The `break` and `continue` keywords can be used like in any other language.

## Shape expressions
Finally we go over how shapes can be made.
There are builtin functions to create some default shapes such as rectangles, circles, triangles and so on.

<br>
<br>
<br>
<br>

For anyone wondering, yes we did have a ton of fun making this file.