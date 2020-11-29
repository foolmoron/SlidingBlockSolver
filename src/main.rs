use std::{collections::HashMap, fs::OpenOptions, fmt, fs};
use std::io::Write;
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
    let mut states: Vec<(Vec<Board>, Board)> = Vec::with_capacity(1000);
    states.push((Vec::new(), Board {
        size: board_config.size,
        state: board_config.board.iter()
            .map(|(name, block)| (name.to_owned(), block.to_owned()))
            .collect()
    }));
    // store only the best path
    let mut best_length = usize::MAX;
    // DFS for all board states, skip ones already seen with a shorter path, store best path
    let mut shortest_path_to = HashMap::new();
    while let Some(state) = states.pop() {
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
            if new_length < best_length {
                best_length = new_length;
                // save results to file
                println!("Found new winner of path length {}", new_length);
                fs::create_dir_all("./results/").expect("mkdir failed");
                let mut file = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create(true)
                    .open(format!("./results/{}.txt", new_length))
                    .unwrap();
                writeln!(file, "Winner!\n").expect("write failed");
                for (i, s) in new_path.iter().enumerate() {
                    writeln!(file, "{}:\n{}", i, s).expect("write failed");
                }
            }
            continue;
        }
        // track shortest path to already-seen ones, skipping ones that are too slow
        let existing_length = shortest_path_to.entry(state.1.clone()).or_insert(new_length);
        if new_length <= *existing_length {
            *existing_length = new_length;
        } else {
            continue;
        }
        // queue up the next moves, using the newly extended path
        for new_state in state.1.get_moves() {
            let existing_length = *shortest_path_to.get(&new_state).unwrap_or(&usize::MAX);
            if new_length < existing_length {
                states.push((new_path.clone(), new_state));
            }
        }
    }
    // print final
    if best_length != usize::MAX {
        println!("Winner! After {} moves, path of {}. See ./results/{}.txt", moves, best_length, best_length);
        return;
    } else {
        println!("FAILED after {} moves", moves);
    }
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