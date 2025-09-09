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

    pub fn set_turn(&mut self, turn: i64) {
        self.turn = turn;
        self.before_time = Instant::now();
    }

    pub fn is_time_over(&self) -> bool {
        let now = Instant::now();
        let whole_diff = now - self.start_time;
        let last_diff = now - self.before_time;
        let remaining_time = self.time_threshold - whole_diff;
        let now_threshold = remaining_time / (self.end_turn - self.turn) as u32;
        last_diff >= now_threshold
    }
}