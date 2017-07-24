# ultimate-tictactoe-ai
Monte Carlo tree search based AI for Ultimate TicTacToe

# Running
You will need:
1. [Rust](https://www.rust-lang.org/en-US/) (If you are using Windows, make sure to use the GNU ABI)
2. [GTK+](https://www.gtk.org/download/index.php)

To run, just run `cargo run --release` in the project root. The `release` flag is suggested to improve the speed of the program, which in turn improves the performance of the AI.

# Configuration
Currently, there is no configuration file (I plan to add this at some point in the future). If you want to adjust the lenght of time given to the AI, modify the `AI_TURN_TIME` constant in `src/main.rs`. The `HUMAN_PLAYER` constant can also be set to `false` to make the AI play against itself.
