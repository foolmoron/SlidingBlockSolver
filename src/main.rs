use std::{collections::HashSet, fmt, fs};
use std::env;
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct BoardConfig {
    size: (usize, usize),
    board: BTreeMap<String, Block>,
    goal_block: String,
    goal_pos: (i32, i32),
}

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Clone, Debug)]
struct Block {
    pos: (i32, i32),
    size: (usize, usize),
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let file = fs::read_to_string(&args[1]).unwrap();
    let board_config: BoardConfig = serde_json::from_str(&file).unwrap();

    // board state = positions of blocks & blanks
    let mut moves: i64 = 0;
    let mut states = Vec::with_capacity(1000);
    states.push(Board {
        size: board_config.size,
        state: board_config.board
    });
    // DFS for all board states, skip already seen ones
    let mut already_seen: HashSet<Board> = HashSet::new();
    while let Some(state) = states.pop() {
        moves += 1;
        // find the one that matches the goal
        if state.wins(&board_config.goal_block, &board_config.goal_pos) {
            states.push(state);
            println!("Winner! After {} moves, with stack of {}", moves, states.len());
            println!("");
            for (i, s) in states.iter().enumerate() {
                println!("{}:\n{}", i, s);
            }
            return;
        }
        // already seen
        already_seen.insert(state.clone());
        // possible moves = for all blocks, do blanks allow it to move in that dir
        for new_state in state.get_moves() {
            if !already_seen.contains(&new_state) {
                states.push(new_state);
            }
        }
    }
    println!("FAILED after {} moves", moves);
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
struct Board {
    size: (usize, usize),
    state: BTreeMap<String, Block>,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut grid = vec![vec!["."; self.size.0]; self.size.1];
        for (name, b) in &self.state {
            for i in 0..b.size.0 {
                for j in 0..b.size.1 {
                    // 0, 0 is top-left logically
                    // and x and y are reversed compared to the array layout
                    let x = b.pos.1 as usize + j;
                    let y = b.pos.0 as usize + i;
                    grid[x][y] = name;
                }
            }
        }
        let rows: Vec<String> = grid.iter().map(|row| row.join("")).collect();
        writeln!(f, "{}", rows.join("\n"))
    }
}

impl Board {


    fn get_moves(&self) -> Vec<Board> {
        return vec![
            // TODO
        ];
    }

    fn wins(&self, block: &String, pos: &(i32, i32)) -> bool {
        return self.state
            .get(block)
            .map(|b| b.pos == *pos)
            .unwrap_or(false);
    }
}