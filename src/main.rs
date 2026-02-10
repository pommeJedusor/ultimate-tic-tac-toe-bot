const FULL_BOARD: u16 = 0b111111111;
const MAX_DEPTH: usize = 7;
const IS_BOARD_WINNING: [bool; 512] = are_boards_winning();
const SCORE_BY_SQUARE: [i32; 9] = [3, 2, 3, 2, 4, 2, 3, 2, 3];

const fn are_boards_winning() -> [bool; 512] {
    let mut results = [false; 512];
    let mut board = 0;
    while board < 512 {
        let is_board_winning = (board & board >> 1 & board >> 2) & 0b1001001 != 0
            || (board & board >> 3 & board >> 6) != 0
            || board & 0b100010001 == 0b100010001
            || board & 0b1010100 == 0b1010100;
        results[board] = is_board_winning;
        board += 1;
    }
    results
}

// max score = 128, min score = -128
fn scores_by_position() -> [i32; 262144] {
    let score_by_number_of_square_controlled = [0, 1, 4, 16];
    let masks = [
        0b111,
        0b111000,
        0b111000000,
        0b1001001,
        0b10010010,
        0b100100100,
        0b100010001,
        0b1010100,
    ];
    let mut scores: [i32; 262144] = [0; 262144];
    let mut position = 0;
    while position < 262144 {
        let player: usize = position >> 9;
        let opponent: usize = position & 0b111111111;
        let mut i = 0;
        while i < masks.len() {
            let mask = masks[i];
            if opponent & mask == 0 {
                scores[position] +=
                    score_by_number_of_square_controlled[(player & mask).count_ones() as usize];
            }
            if player & mask == 0 {
                scores[position] -=
                    score_by_number_of_square_controlled[(opponent & mask).count_ones() as usize];
            }
            i += 1;
        }
        position += 1;
    }
    scores
}

#[derive(Debug, Copy, Clone)]
struct MoveStruct {
    moves: [(u8, i32); 81],
    index: u8,
}

impl MoveStruct {
    fn init() -> MoveStruct {
        return MoveStruct {
            moves: [(0, 0); 81],
            index: 0,
        };
    }

    fn reset(&mut self) {
        self.index = 0;
    }

    fn push(&mut self, r#move: u8) {
        self.moves[self.index as usize].0 = r#move;
        self.index += 1;
    }
}

struct Board {
    players_mini_boards: [[u16; 9]; 2],
    players_big_board: [u16; 2],
    mini_board_can_play: u16,
    finished_played_boards: u16,
    player_turn: usize,
}

impl Board {
    fn init() -> Board {
        Board {
            players_mini_boards: [[0; 9]; 2],
            players_big_board: [0; 2],
            mini_board_can_play: 0b111111111,
            finished_played_boards: 0,
            player_turn: 0,
        }
    }

    fn get_moves_bitmap(&self, board_index: usize) -> u16 {
        let bitmap_board =
            self.players_mini_boards[0][board_index] | self.players_mini_boards[1][board_index];
        !bitmap_board & 0b111111111
    }

    fn get_moves(&self, moves: &mut MoveStruct) {
        moves.reset();
        let mut available_boards_bitmap = self.mini_board_can_play;
        while available_boards_bitmap != 0 {
            let board_index = available_boards_bitmap.trailing_zeros();

            let mut moves_bitmap = self.get_moves_bitmap(board_index as usize);
            while moves_bitmap != 0 {
                let move_index = moves_bitmap.trailing_zeros();
                moves.push((board_index << 4 | move_index) as u8);
                moves_bitmap ^= 1 << move_index;
            }

            available_boards_bitmap ^= 1 << board_index;
        }
    }

    fn is_mini_board_winning(&self, board_index: usize) -> bool {
        let board = self.players_mini_boards[self.player_turn][board_index];
        IS_BOARD_WINNING[board as usize]
    }

    fn is_losing(&self) -> bool {
        let board = self.players_big_board[self.player_turn ^ 1];
        IS_BOARD_WINNING[board as usize]
    }

    fn play_move(&mut self, r#move: u8) {
        let board_index = r#move as usize >> 4;
        let move_index = r#move & 0b1111;
        self.players_mini_boards[self.player_turn][board_index] ^= 1 << move_index;

        let board = self.players_mini_boards[self.player_turn][board_index]
            | self.players_mini_boards[self.player_turn ^ 1][board_index];
        if self.is_mini_board_winning(board_index) {
            self.finished_played_boards |= 1 << board_index;
            self.players_big_board[self.player_turn] |= 1 << board_index;
        } else if board == FULL_BOARD {
            self.finished_played_boards |= 1 << board_index;
        }

        self.mini_board_can_play = 1 << move_index;
        if self.finished_played_boards & self.mini_board_can_play != 0 {
            self.mini_board_can_play = !self.finished_played_boards & 0b111111111;
        }

        self.player_turn ^= 1;
    }

    fn cancel_move(&mut self, r#move: u8, mini_board_can_play: u16) {
        let board_index = r#move as usize >> 4;
        let move_index = r#move & 0b1111;

        self.player_turn ^= 1;

        self.players_mini_boards[self.player_turn][board_index] ^= 1 << move_index;

        self.finished_played_boards &= self.finished_played_boards ^ 1 << board_index;
        self.players_big_board[self.player_turn] &=
            self.players_big_board[self.player_turn] ^ 1 << board_index;

        self.mini_board_can_play = mini_board_can_play;
    }

    fn eval(&self, scores_by_position: &[i32; 262144]) -> i32 {
        let position = (self.players_big_board[self.player_turn] as usize) << 9
            | (self.players_big_board[self.player_turn ^ 1] as usize);
        let mut score = scores_by_position[position] * 1000;
        for i in (0..9).filter(|i| 1 << i & self.finished_played_boards == 0) {
            let position = (self.players_mini_boards[self.player_turn][i] as usize) << 9
                | (self.players_mini_boards[self.player_turn ^ 1][i] as usize);
            score += scores_by_position[position] * SCORE_BY_SQUARE[i];
        }
        score
    }

    fn minimax(
        &mut self,
        moves_by_depth: &mut [MoveStruct; MAX_DEPTH],
        scores_by_position: &[i32; 262144],
        depth: usize,
        mut alpha: i32,
        beta: i32,
    ) -> (u8, i32) {
        self.get_moves(&mut moves_by_depth[depth]);
        if moves_by_depth[depth].index == 0 {
            let squares_controlled_by_player =
                self.players_big_board[self.player_turn].count_ones();
            let squares_controlled_by_opponent =
                self.players_big_board[self.player_turn ^ 1].count_ones();
            if squares_controlled_by_player == squares_controlled_by_opponent {
                return (0, 0);
            }
            if squares_controlled_by_player < squares_controlled_by_opponent {
                return (0, (MAX_DEPTH - depth) as i32 * 1_000_000);
            }
            return (0, (MAX_DEPTH - depth) as i32 * -1_000_000);
        }

        for i in 0..moves_by_depth[depth].index {
            let (r#move, _) = moves_by_depth[depth].moves[i as usize];
            let previous_mini_board_can_play = self.mini_board_can_play;

            self.play_move(r#move);
            if self.is_losing() {
                let score = (MAX_DEPTH - depth) as i32 * 1_000_000;
                moves_by_depth[depth].moves[i as usize].1 = score;
            } else if depth + 1 == MAX_DEPTH {
                let score = -self.eval(scores_by_position);
                moves_by_depth[depth].moves[i as usize].1 = score;
            } else {
                let (_, best_score) =
                    self.minimax(moves_by_depth, scores_by_position, depth + 1, -beta, -alpha);
                moves_by_depth[depth].moves[i as usize].1 = -best_score;
            }
            self.cancel_move(r#move, previous_mini_board_can_play);
            alpha = alpha.max(moves_by_depth[depth].moves[i as usize].1);
            if alpha > beta {
                break;
            }
        }
        let best_move_index = (0..moves_by_depth[depth].index)
            .max_by_key(|i| moves_by_depth[depth].moves[*i as usize].1)
            .unwrap();
        let (best_move, best_score) = moves_by_depth[depth].moves[best_move_index as usize];
        (best_move, best_score)
    }
}

fn move_u16_to_row_col(r#move: u8) -> (u8, u8) {
    let board_index = r#move >> 4;
    let move_index = r#move & 0b1111;
    let x = board_index % 3 * 3 + move_index % 3;
    let y = board_index / 3 * 3 + move_index / 3;
    (y, x)
}

fn row_col_move_to_u16(row: u8, col: u8) -> u8 {
    let big_board_y = row / 3;
    let big_board_x = col / 3;
    let board_index = big_board_y * 3 + big_board_x;
    let mini_board_y = row % 3;
    let mini_board_x = col % 3;
    let move_index = mini_board_y * 3 + mini_board_x;
    board_index << 4 | move_index
}

fn main() {
    let board = Board::init();
    let mut moves_by_depth = [MoveStruct::init(); MAX_DEPTH];
    board.get_moves(&mut moves_by_depth[0]);
    let move_index = 80;
    println!("{:?}", moves_by_depth[0]);
    println!("{:?}", moves_by_depth[0].moves[move_index]);
    let scores_by_position: [i32; 262144] = scores_by_position();
    println!("{}", scores_by_position[1]);
}
