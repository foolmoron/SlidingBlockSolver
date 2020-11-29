# Sliding Block Solver (in Rust!)

This was made to solve a very old but surprisingly ingenious and difficult sliding puzzle that I was challenged to solve:
https://www.google.com/search?q=drueke+%26+sons+grand+rapids+puzzle

However, it should work with any rectangle-based sliding block configuration.

The code is quite ugly, since it's my first time using Rust. It uses a DFS search (due to memory constraints when using BFS) and gradually finds shorter and shorter solutions, which are saved to text files in the `results` folder. It will only terminate after checking all paths so you know for sure that you have the shortest solution, but you can end it early and just use shortest text file if that's good enough.

## Example

Set up the puzzle in JSON format (see `puzzles` folder). Currently `board_text` does not do anything, you have to manually set up the board config.
```json
{
    "board_text": [
        "A..",
        "...",
        "..C"
    ],
    "size": [3, 3],
    "board": {
        "A": { "pos": [0, 0], "size": [1, 1] },
        "C": { "pos": [2, 2], "size": [1, 1] }
    },
    "goal_block": "A",
    "goal_pos": [2, 2]
}
```

Run the solver:
```sh
cargo build --release
./target/release/sliding-block-solver.exe ./puzzles/test.json
```

You will get a progressive list of logs indicating every 100000th move processed, and any shorters paths found. At the end of the program, it will print the winner, and the full path of the solution:
```
Found better path of length 9
Found better path of length 8
Found better path of length 7
Found better path of length 6
Found better path of length 5
Winner! After 2870 moves, with path of 5

0:
C <

A..
...
.C.

1:
A >

.A.
...
.C.

2:
A >

..A
...
.C.

3:
A v

...
..A
.C.

4:
A v

...
...
.CA
```

Each step in the path shows the move required, with the block's name (A, B, C, etc) and the direction (^, v, <, >).

It also shows the state of the board after that move.
