use std::collections::HashMap;
use std::{collections::HashSet, fmt, collections::VecDeque, fs};
use std::env;
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct BoardConfig {
    size: (i32, i32),
    board: BTreeMap<char, Block>,
    goal_block: char,
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
    let mut states: VecDeque<(Vec<(char, char)>, Board)> = VecDeque::with_capacity(1000);
    let initial_board = Board {
        size: board_config.size,
        state: board_config.board.iter()
            .map(|(name, block)| (name.to_owned(), block.to_owned()))
            .collect()
    };
    states.push_back((vec![], initial_board.clone()));
    // BFS for all board states, skip ones already seen
    let mut already_seen = HashSet::new();
    while let Some(state) = states.pop_front() {
        moves += 1;
        if moves % 1000000 == 0 {
            println!("Processed {} moves, {} in queue", moves, states.len());
        }
        // find the one that matches the goal
        if state.1.wins(&board_config.goal_block, &board_config.goal_pos) {
            println!("Winner! After {} moves, with path of {}", moves, state.0.len());
            println!("");
            print_moves(initial_board.clone(), state.0.clone());
            return;
        }
        // track shortest path to already-seen ones, skipping ones that are too slow
        already_seen.insert(state.1.clone());
        // queue up the next moves, using the newly extended path
        for (m, new_state) in state.1.get_moves() {
            if !already_seen.contains(&new_state) {
                let mut new_path = state.0.clone();
                new_path.push(m);
                states.push_back((new_path, new_state));
            }
        }
    }
    // failed
    println!("FAILED after {} moves", moves);
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
struct Board {
    size: (i32, i32),
    state: Vec<(char, Block)>,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let grid = self.get_grid('.', |name, _| name.clone());
        let rows: Vec<String> = grid.iter().map(|row| row.iter().collect()).collect();
        writeln!(f, "{}", rows.join("\n"))
    }
}

impl Board {

    fn get_grid<T: Clone>(&self, default_value: T, new_value: impl Fn(&char, &Block) -> T) -> Vec<Vec<T>> {
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

    fn get_moves(&self) -> Vec<((char, char), Board)> {
        // calculate blanks by filling in all the block spaces with false
        let blanks = self.get_grid(true, |_, _| false);
        // use blanks to check all 4 dirs for any valid moves
        let mut moves = Vec::new();
        for (index, (name, block)) in self.state.iter().enumerate() {
            let dirs:[(i32, i32, char); 4] = [(0, 1, 'v'), (0, -1, '^'), (1, 0, '>'), (-1, 0, '<')];
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
                    new_state[index] = (*name, new_block);

                    moves.push((
                        (*name, dir.2),
                        Board {
                            size: self.size,
                            state: new_state,
                        }
                    ));
                }
            }
        }
        return moves;
    }

    fn wins(&self, block: &char, pos: &(i32, i32)) -> bool {
        return self.state.iter()
            .find(|(n, _)| n == block)
            .map(|(_, b)| b.pos == *pos)
            .unwrap_or(false);
    }
}

fn print_moves(mut board: Board, moves: Vec<(char, char)>) {
    let mut dirs: HashMap<char, (i32, i32)> = HashMap::new();
    dirs.insert('v', (0, 1));
    dirs.insert('^', (0, -1));
    dirs.insert('>', (1, 0));
    dirs.insert('<', (-1, 0));
    for (i, m) in moves.into_iter().enumerate() {
        let mut pos = &mut board.state.iter_mut().find(|item| item.0 == m.0).unwrap().1.pos;
        pos.0 += dirs[&m.1].0;
        pos.1 += dirs[&m.1].1;
        println!("{}:\n{} {}\n", i, m.0, m.1);
        println!("{}", board);
    }
}
