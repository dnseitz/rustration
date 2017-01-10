# Rustration

## A simple Brainfuck interpreter in Rust

```bf
++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.
```

### What is Brainfuck?

Brainfuck is an esoteric programming language invented by Urban Muller in 1993. The language
operates on an array of memory cells, also called a tape. Every cell is intialized to 0. There
is a pointer that intially points to the first memory cell, and several commands are used to
manipulate the pointer and the data on the tape. The set of commands are `>`, `<`, `+`, `-`,
`[`, `]`, `.`, and `,`. 

`<` and `>` move the data pointer left and right respectively. `+` and `-` increment and
decrement the data in the cell being pointed at. `[` and `]` act as a looping mechanism for the
language. A `[` command means jump past the matching `]` if the cell under the data pointer is
0. A `]` command means jump back to the matching `[` if the cell under the data pointer is not
0. This looping construct is similar to a while loop in a C-like language. A C representation
would be something like:

```c
while *data != 0 {

}
```

The `,` and `.` commands act as input and output respectively. `,` inputs a character and
stores it at the data pointer, `.` outputs the character under the data pointer.

All other characters are considered comments and are ignored.

