use std::time::Duration;

const TM_BASE_MULT: f64 = 0.05;
const TM_INC_MULT: f64 = 0.08;
const TM_SOFT_MULT: f64 = 0.66;
const TM_HARD_MULT: f64 = 2.0;
const TM_UCI_OVERHEAD: f64 = 50.0;

pub struct SearchLimits {
    pub hard_time: Option<Duration>,
    pub soft_time: Option<Duration>,
    pub soft_nodes: Option<u64>,
    pub hard_nodes: Option<u64>,
    pub depth: Option<u64>,
}

pub enum LimitType {
    Soft,
    Hard,
}

pub type FischerTime = (u64, u64);

impl SearchLimits {
    pub fn new(
        fischer: Option<FischerTime>,
        movetime: Option<u64>,
        soft_nodes: Option<u64>,
        hard_nodes: Option<u64>,
        depth: Option<u64>,
    ) -> SearchLimits {
        let (soft_time, hard_time) = match (fischer, movetime) {
            (Some(f), _) => {
                let (soft, hard) = Self::calc_time_limits(f);
                (Some(soft), Some(hard))
            }
            (None, Some(mt)) => {
                let duration = Duration::from_millis(mt);
                (Some(duration), Some(duration))
            }
            (None, None) => (None, None),
        };

        SearchLimits {
            hard_time,
            soft_time,
            soft_nodes,
            hard_nodes,
            depth,
        }
    }

    fn calc_time_limits(fischer: FischerTime) -> (Duration, Duration) {
        let (time, inc) = (fischer.0 as f64, fischer.1 as f64);
        let base = time * TM_BASE_MULT + inc * TM_INC_MULT;
        let soft_time = base * TM_SOFT_MULT;
        let hard_time = base * TM_HARD_MULT;
        let soft = soft_time.min(time - TM_UCI_OVERHEAD);
        let hard = hard_time.min(time - TM_UCI_OVERHEAD);
        (
            Duration::from_millis(soft as u64),
            Duration::from_millis(hard as u64),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_fischer() {
        let fischer = Some((100_000, 10_000));

        let limits = SearchLimits::new(fischer, None, Some(1000), Some(2000), Some(10));

        assert_eq!(limits.soft_time, Some(Duration::from_millis(3828)));
        assert_eq!(limits.hard_time, Some(Duration::from_millis(11600)));
        assert_eq!(limits.soft_nodes, Some(1000));
        assert_eq!(limits.hard_nodes, Some(2000));
        assert_eq!(limits.depth, Some(10));
    }

    #[test]
    fn test_movetime() {
        let movetime = Some(5000);

        let limits = SearchLimits::new(None, movetime, None, None, None);

        assert_eq!(limits.soft_time, Some(Duration::from_millis(5000)));
        assert_eq!(limits.hard_time, Some(Duration::from_millis(5000)));
        assert_eq!(limits.soft_nodes, None);
        assert_eq!(limits.hard_nodes, None);
        assert_eq!(limits.depth, None);
    }
}
