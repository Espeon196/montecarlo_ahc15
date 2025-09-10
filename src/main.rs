#![allow(unused_imports, dead_code)]

mod time_keeper {
    use std::time::{Instant, Duration};

    pub struct TimeKeeper {
        start_time: Instant,
        before_time: Instant,
        time_threshold: Duration,
        end_turn: i64,
        turn: i64,
    }

    impl TimeKeeper {
        /// 全ターン含めての制限時間と最大ターン数を指定してTimeKeeperを作成する
        /// * `time_threshold` - 全体の時間制限(msec)
        /// * `end_turn` - 最大ターン数
        pub fn new(time_threshold: u64, end_turn: i64) -> Self {
            Self {
                start_time: Instant::now(),
                before_time: Instant::now(),
                time_threshold: Duration::from_millis(time_threshold),
                end_turn,
                turn: 0,
            }
        }

        /// ターンとターン開始時間を更新する
        pub fn set_turn(&mut self, turn: i64) {
            self.turn = turn;
            self.before_time = Instant::now();
        }

        /// 各ターンに割り振られた制限時間を超過したか判定
        pub fn is_time_over(&self) -> bool {
            let now = Instant::now();
            let whole_diff = now - self.start_time;
            let last_diff = now - self.before_time;
            let remaining_time = self.time_threshold - whole_diff;
            let now_threshold = remaining_time / (self.end_turn - self.turn) as u32;
            last_diff >= now_threshold
        }
    }
}

use rand::{rngs::StdRng, Rng, SeedableRng};
use once_cell::sync::Lazy;

use proconio::input;
use proconio::source::line::LineSource;

use std::io::{BufReader, Write};
use std::sync::Mutex;
use std::collections::VecDeque;

use time_keeper::TimeKeeper;

const H: usize = 10;
const W: usize = 10;
const END_TURN: i64 = 100;

pub static FUTURE_CANDIES: Lazy<Mutex<[u8; END_TURN as usize]>> = Lazy::new(|| Mutex::new([0u8; END_TURN as usize]));
pub static RAND_FOR_ACTION: Lazy<Mutex<StdRng>> = Lazy::new(|| {
    Mutex::new(StdRng::seed_from_u64(80))
});

const SIMULATION_MAX: usize = 14000;
pub static RANDOM_FOR_SIMULATION: Lazy<Mutex<Vec<Vec<i64>>>> = Lazy::new(|| {
    Mutex::new(vec![vec![0i64; END_TURN as usize]; SIMULATION_MAX])
});


#[derive(Clone, Copy)]
pub enum Action {
    Forward,
    Back,
    Left,
    Right,
}

pub fn action_to_char(action: Action) -> char {
    match action {
        Action::Forward => 'F',
        Action::Back => 'B',
        Action::Left => 'L',
        Action::Right => 'R',
    }
}

#[derive(Clone)]
pub struct State {
    board: [[u8; W]; H],
    turn: i64,
}

impl State {
    pub fn new() -> Self {
        Self { board: [[0u8; W]; H], turn: 0i64 }
    }

    pub fn is_done(&self) -> bool {
        self.turn >= END_TURN
    }

    pub fn advance(&mut self, action: Action) {
        match action {
            Action::Forward => {
                for x in 0..W {
                    let mut dest = 0usize;
                    for y in 0..H {
                        if self.board[y][x] == 0 {
                            continue;
                        }
                        (self.board[y][x], self.board[dest][x]) = (self.board[dest][x], self.board[y][x]);
                        dest += 1;
                    }
                }
            },
            Action::Back => {
                for x in 0..W {
                    let mut dest = H - 1;
                    for y in (0..H).rev() {
                        if self.board[y][x] == 0 {
                            continue;
                        }
                        (self.board[y][x], self.board[dest][x]) = (self.board[dest][x], self.board[y][x]);
                        dest -= 1;
                    }
                }
            },
            Action::Left => {
                for y in 0..H {
                    let mut dest = 0;
                    for x in 0..W {
                        if self.board[y][x] == 0 {
                            continue;
                        }
                        (self.board[y][x], self.board[y][dest]) = (self.board[y][dest], self.board[y][x]);
                        dest += 1;
                    }
                }
            },
            Action::Right => {
                for y in 0..H {
                    let mut dest = W - 1;
                    for x in (0..W).rev() {
                        if self.board[y][x] == 0 {
                            continue;
                        }
                        (self.board[y][x], self.board[y][dest]) = (self.board[y][dest], self.board[y][x]);
                        dest -= 1;
                    }
                }
            }
        }
        self.turn += 1;
    }

    pub fn random_update(&mut self) {
        let remain_turn = END_TURN - self.turn;
        let p = RAND_FOR_ACTION.lock().unwrap().gen_range(1..=remain_turn);
        self.update(p);
    }

    pub fn simulation_update(&mut self, simulation_cnt: usize) {
        let p = RANDOM_FOR_SIMULATION.lock().unwrap()[simulation_cnt][self.turn as usize];
        self.update(p);
    }

    pub fn update(&mut self, pt: i64) {
        let mut cnt = 0i64;
        let candies = FUTURE_CANDIES.lock().unwrap();
        for y in 0..H {
            for x in 0..W {
                if self.board[y][x] != 0 {
                    continue;
                }
                cnt += 1;
                if cnt == pt {
                    self.board[y][x] = candies[self.turn as usize];
                }
            }
        }
    }

    fn get_group_size(&self, y: usize, x: usize, checked: &mut [[bool; W]; H]) -> i64 {
        const DX: [isize; 4] = [1, -1, 0, 0];
        const DY: [isize; 4] = [0, 0, 1, -1];
        let candy = self.board[y][x];
        checked[y][x] = true;

        let mut q = VecDeque::new();
        q.push_back((y, x));
        let mut cnt = 0;
        while !q.is_empty() {
            cnt += 1;
            let (now_y, now_x) = q.pop_front().unwrap();
            for i in 0..4usize { 
                let ty = now_y as isize + DY[i];
                let tx = now_x as isize + DX[i];

                if (0..H as isize).contains(&ty) && (0..W as isize).contains(&tx) {
                    let new_y = ty as usize;
                    let new_x = tx as usize;
                    if !checked[new_y][new_x] && self.board[new_y][new_x] == candy {
                        checked[new_y][new_x] = true;
                        q.push_back((new_y, new_x));
                    }
                }
            }
        }
        cnt
    }

    pub fn get_score(&self) -> f64 {
        let mut score = 0.;
        let mut checked = [[false; W]; H];
        for y in 0..H {
            for x in 0..W {
                if self.board[y][x] != 0 && !checked[y][x] {
                    let group_size = self.get_group_size(y, x, &mut checked);
                    score += (group_size * group_size) as f64;
                }
            }
        }
        score
    }
}

pub const LEGAL_ACTIONS: [Action; 4] = [Action::Forward, Action::Back, Action::Left, Action::Right];

pub fn random_action(_state: &State) -> Action {
    let random_idx = RAND_FOR_ACTION.lock().unwrap().gen_range(0..LEGAL_ACTIONS.len());
    LEGAL_ACTIONS[random_idx]
}

pub fn rulebase_action(state: &State) -> Action {
    let rule = [
        [Action::Forward, Action::Back, Action::Back],
        [Action::Forward, Action::Left, Action::Right],
        [Action::Forward, Action::Left, Action::Right],
    ];
    let candies = FUTURE_CANDIES.lock().unwrap();
    let turn = state.turn;
    if turn >= END_TURN - 1 {
        return Action::Forward;
    }
    let now_candy_idx = candies[state.turn as usize] as usize - 1;
    let next_candy_idx = candies[state.turn as usize + 1] as usize - 1;
    return rule[now_candy_idx][next_candy_idx];
}

mod montecalro {
    use crate::SIMULATION_MAX;

    use super::{State, LEGAL_ACTIONS, random_action, rulebase_action, Action};
    use super::time_keeper::TimeKeeper;

    fn playout(state: &mut State, simulation_cnt: usize) -> f64 {
        while !state.is_done() {
            state.simulation_update(simulation_cnt);
            //state.advance(random_action(state));
            state.advance(rulebase_action(state));
        }
        state.get_score()
    }

    pub fn primitive_monteralro(time_keeper: &TimeKeeper, base_state: &State) -> Action {
        let mut w = [0.; LEGAL_ACTIONS.len()];
        for simulation_cnt in 0..SIMULATION_MAX {
            if time_keeper.is_time_over() {
                break;
            }
            for d in 0..LEGAL_ACTIONS.len() {
                let mut state = base_state.clone();
                state.advance(LEGAL_ACTIONS[d]);
                w[d] += playout(&mut state, simulation_cnt);
            }
        }
        let mut best_score = 0.;
        let mut best_action_idx = 0usize;
        for (d, wd) in w.iter().enumerate() {
            if *wd > best_score { 
                best_action_idx = d;
                best_score = *wd;
            }
        }
        LEGAL_ACTIONS[best_action_idx]
    }
}

fn main() {
    {
        let mut rand_for_simulation = RANDOM_FOR_SIMULATION.lock().unwrap();
        let mut rng = StdRng::seed_from_u64(0);
        for simulation_cnt in 0..SIMULATION_MAX {
            for turn in 0..END_TURN {
                let remain_turn = END_TURN - turn;
                let p = rng.gen_range(1..=remain_turn);
                rand_for_simulation[simulation_cnt][turn as usize] = p;
            }
        }
    }

    let mut source = LineSource::new(BufReader::new(std::io::stdin()));
    input! {
        from &mut source,
        future: [u8; END_TURN],
    }
    {
        let mut candies = FUTURE_CANDIES.lock().unwrap();
        for (t, &f) in future.iter().enumerate() {
            candies[t] = f;
        }
    }

    let mut state = State::new();
    let mut time_keeper = TimeKeeper::new(1950, END_TURN);

    for turn in 0..END_TURN {
        time_keeper.set_turn(turn);
        input! {
            from &mut source,
            pt: i64,
        }
        state.update(pt);
        let action = montecalro::primitive_monteralro(&time_keeper, &state);
        // let action = rulebase_action(&state);
        println!("{}", action_to_char(action));
        std::io::stdout().flush().unwrap();
        state.advance(action);
    }
}
