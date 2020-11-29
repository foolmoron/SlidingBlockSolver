use std::io::stdout;
use std::{fs::OpenOptions};
use std::collections::HashMap;
use std::io::Write;
use std::{fmt, fs};
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
    let initial_best_length = args
        .get(2).unwrap_or(&String::from(""))
        .parse::<usize>().unwrap_or(usize::MAX - 1)
        + 1;
    let board_config: BoardConfig = serde_json::from_str(&file).unwrap();

    // state = path to current state, board of block positions
    let mut moves: i64 = 0;
    let mut states: Vec<(Vec<(char, char)>, Board)> = Vec::with_capacity(1000);
    let initial_board = Board {
        size: board_config.size,
        state: board_config.board.iter()
            .map(|(name, block)| (name.to_owned(), block.to_owned()))
            .collect()
    };
    states.push((vec![], initial_board.clone()));
    // store only the best path
    let mut best_length = initial_best_length;
    let mut best_path: Vec<(char, char)> = Vec::new();
    // DFS for all board states, skip ones already seen with a shorter path, store best path
    let mut shortest_path_to = HashMap::new();
    while let Some(state) = states.pop() {
        moves += 1;
        if moves % 1000000 == 0 {
            println!("Processed {} moves, {} in queue", moves, states.len());
        }
        let path_len = state.0.len();
        // stop if longer than best length so far
        if path_len >= best_length {
            continue;
        }
        // find the one that matches the goal
        if state.1.wins(&board_config.goal_block, &board_config.goal_pos) {
            if path_len < best_length {
                println!("Found better path of length {}", path_len);
                best_length = path_len;
                best_path = state.0.clone();
                // write to file
                fs::create_dir_all("./results/").expect("mkdir failed");
                let mut file = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create(true)
                    .open(format!("./results/{}.txt", path_len))
                    .unwrap();
                writeln!(file, "Winner!\n").expect("write failed");
                print_moves(&mut file, initial_board.clone(), best_path.clone());
            }
        }
        // track shortest path to already-seen ones, skipping ones that are too slow
        let shortest = *shortest_path_to.get(&state.1).unwrap_or(&usize::MAX);
        if path_len < shortest {
            shortest_path_to.insert(state.1.clone(), path_len);
        }
        // queue up the next moves, using the newly extended path
        for (m, new_state) in state.1.get_moves() {
            if path_len < *shortest_path_to.get(&new_state).unwrap_or(&usize::MAX) {
                let mut new_path = state.0.clone();
                new_path.push(m);
                states.push((new_path, new_state));
            }
        }
    }
    // print result
    if best_path.len() > 0 {
        println!("Winner! After {} moves, with path of {}", moves, best_length);
        println!("");
        print_moves(&mut stdout(), initial_board.clone(), best_path.clone());

    } else {
        println!("FAILED after {} moves", moves);
    }
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

fn print_moves(out: &mut (impl Write + ?Sized), mut board: Board, moves: Vec<(char, char)>) {
    let mut dirs: HashMap<char, (i32, i32)> = HashMap::new();
    dirs.insert('v', (0, 1));
    dirs.insert('^', (0, -1));
    dirs.insert('>', (1, 0));
    dirs.insert('<', (-1, 0));
    for (i, m) in moves.into_iter().enumerate() {
        let mut pos = &mut board.state.iter_mut().find(|item| item.0 == m.0).unwrap().1.pos;
        pos.0 += dirs[&m.1].0;
        pos.1 += dirs[&m.1].1;
        out.write_fmt(format_args!("{}:\n{} {}\n\n", i, m.0, m.1)).expect("write failed");
        out.write_fmt(format_args!("{}\n", board)).expect("write failed");
    }
}
