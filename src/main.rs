use std::{collections::HashSet, fmt, collections::VecDeque, fs};
use std::env;
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct BoardConfig {
    size: (i32, i32),
    board: BTreeMap<String, Block>,
    goal_block: String,
    goal_pos: (i32, i32),
}

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Clone, Debug)]
struct Block {
    pos: (i32, i32),
    size: (i32, i32),
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let file = fs::read_to_string(&args[1]).unwrap();
    let board_config: BoardConfig = serde_json::from_str(&file).unwrap();

    // state = path to current state, board of block positions
    let mut moves: i64 = 0;
    let mut states: VecDeque<(Vec<Board>, Board)> = VecDeque::with_capacity(1000);
    states.push_back((Vec::new(), Board {
        size: board_config.size,
        state: board_config.board.iter()
            .map(|(name, block)| (name.to_owned(), block.to_owned()))
            .collect()
    }));
    // BFS for all board states, skip ones already seen
    let mut already_seen = HashSet::new();
    while let Some(state) = states.pop_front() {
        moves += 1;
        if moves % 10000 == 0 {
            println!("Processed {} moves", moves);
        }
        // add to path
        let mut new_path = state.0;
        new_path.push(state.1.clone());
        let new_length = new_path.len();
        // find the one that matches the goal
        if state.1.wins(&board_config.goal_block, &board_config.goal_pos) {
            println!("Winner! After {} moves, with path of {}", moves, new_length);
            println!("");
            for (i, s) in new_path.iter().enumerate() {
                println!("{}:\n{}", i, s);
            }
            return;
        }
        // track shortest path to already-seen ones, skipping ones that are too slow
        already_seen.insert(state.1.clone());
        // queue up the next moves, using the newly extended path
        for new_state in state.1.get_moves() {
            if !already_seen.contains(&new_state) {
                states.push_back((new_path.clone(), new_state));
            }
        }
    }
    // failed
    println!("FAILED after {} moves", moves);
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
struct Board {
    size: (i32, i32),
    state: Vec<(String, Block)>,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let grid = self.get_grid(".".to_owned(), |name, _| name.clone());
        let rows: Vec<String> = grid.iter().map(|row| row.join("")).collect();
        writeln!(f, "{}", rows.join("\n"))
    }
}

impl Board {

    fn get_grid<T: Clone>(&self, default_value: T, new_value: impl Fn(&String, &Block) -> T) -> Vec<Vec<T>> {
        let mut grid = vec![vec![default_value; self.size.0 as usize]; self.size.1 as usize];
        for (name, block) in &self.state {
            for i in 0..block.size.0 {
                for j in 0..block.size.1 {
                    // 0, 0 is top-left logically
                    let x = block.pos.0 + i;
                    let y = block.pos.1 + j;
                    // and x and y are reversed compared to the array layout
                    grid[y as usize][x as usize] = new_value(name, block);
                }
            }
        }
        return grid;
    }

    fn get_moves(&self) -> Vec<Board> {
        // calculate blanks by filling in all the block spaces with false
        let blanks = self.get_grid(true, |_, _| false);
        // use blanks to check all 4 dirs for any valid moves
        let mut moves = Vec::new();
        for (index, (name, block)) in self.state.iter().enumerate() {
            let dirs:[(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];
            for dir in &dirs {
                let mut all_new_coords_valid = true;
                // use horiz/vert block size depending on dir
                let size = if dir.0 == 0 { block.size.0 } else { block.size.1 };
                for s in 0..size {
                    // use near/far block edge based on dir
                    let edge_x = block.pos.0 + (block.size.0 - 1) * (if dir.0 == 1 { 1 } else { 0 });
                    let edge_y = block.pos.1 + (block.size.1 - 1) * (if dir.1 == 1 { 1 } else { 0 });
                    // final motion coordinate to check for blank
                    let x = edge_x + (s * dir.1.abs()) + dir.0;
                    let y = edge_y + (s * dir.0.abs()) + dir.1;
                    all_new_coords_valid &= blanks
                        .get(y as usize)
                            .map(|row| row.get(x as usize))
                            .unwrap_or(Some(&false))
                        .unwrap_or(&false);
                }
                if all_new_coords_valid {
                    let mut new_block = block.clone();
                    new_block.pos.0 += dir.0;
                    new_block.pos.1 += dir.1;

                    let mut new_state = self.state.clone();
                    new_state[index] = (name.to_owned(), new_block);

                    moves.push(Board {
                        size: self.size,
                        state: new_state,
                    });
                }
            }
        }
        return moves;
    }

    fn wins(&self, block: &String, pos: &(i32, i32)) -> bool {
        return self.state.iter()
            .find(|(n, _)| n == block)
            .map(|(_, b)| b.pos == *pos)
            .unwrap_or(false);
    }
}