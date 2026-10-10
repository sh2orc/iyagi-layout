use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::geometry::{Action, Rect};

/// 창이 차지할 비율. 순환 옵션이 꺼져 있으면 항상 `Half`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Fraction {
    Half,
    TwoThirds,
    Third,
}

impl Fraction {
    pub fn ratio(self) -> f64 {
        match self {
            Fraction::Half => 0.5,
            Fraction::TwoThirds => 2.0 / 3.0,
            Fraction::Third => 1.0 / 3.0,
        }
    }

    fn next(self) -> Fraction {
        match self {
            Fraction::Half => Fraction::TwoThirds,
            Fraction::TwoThirds => Fraction::Third,
            Fraction::Third => Fraction::Half,
        }
    }
}

/// 이번 입력을 어떻게 계산할지.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    /// 화면 가용 영역을 `fraction`만큼 나눈다.
    FromScreen(Fraction),
    /// 직전 배치에 이어서 **현재 창**을 반으로 나눈다(왼쪽 → 아래 = 왼쪽 아래 1/4).
    Chained,
}

/// 연속 입력 판정. 창을 식별하지 않고, "직전에 우리가 둔 프레임에 창이 아직 그대로
/// 있는가"(±`TOLERANCE`pt)와 경과 시간으로 연속 여부를 정한다.
#[derive(Debug, Default)]
pub struct CycleState {
    last: Option<Last>,
}

#[derive(Debug)]
struct Last {
    action: Action,
    frame: Rect,
    fraction: Fraction,
    at: Instant,
}

/// 판정에 쓰는 설정 묶음.
#[derive(Debug, Clone, Copy)]
pub struct Rules {
    /// 같은 방향 반복 시 1/2 → 2/3 → 1/3 순환.
    pub cycle_thirds: bool,
    /// 축이 다른 방향키가 이 시간 안에 오면 현재 창을 다시 나눈다. 0이면 끈다.
    pub chain_window: Duration,
}

impl CycleState {
    pub const TOLERANCE: f64 = 1.0;

    fn continuation(&self, current: &Rect, now: Instant, window: Duration) -> Option<&Last> {
        let last = self.last.as_ref()?;
        if !last.frame.approx_eq(current, Self::TOLERANCE) {
            return None;
        }
        if now.saturating_duration_since(last.at) > window {
            return None;
        }
        Some(last)
    }

    /// 이번 입력의 계산 방식.
    /// - 축이 다른 키가 `chain_window` 안에 오면 `Chained`.
    /// - 같은 키가 이어지면(순환 옵션 on) 다음 비율. 순환의 시간 제한은 없다 —
    ///   창이 그 자리에 있는 한 이어진다.
    /// - 그 외는 화면 기준 1/2.
    pub fn plan(&self, action: Action, current: &Rect, now: Instant, rules: Rules) -> Plan {
        if rules.chain_window > Duration::ZERO {
            if let Some(last) = self.continuation(current, now, rules.chain_window) {
                if last.action.is_horizontal() != action.is_horizontal() {
                    return Plan::Chained;
                }
            }
        }
        if rules.cycle_thirds {
            if let Some(last) = self.continuation(current, now, Duration::MAX) {
                if last.action == action {
                    return Plan::FromScreen(last.fraction.next());
                }
            }
        }
        Plan::FromScreen(Fraction::Half)
    }

    /// 적용 **후** 실제로 읽은 프레임을 기록한다(쓰려던 값이 아니라 결과).
    pub fn record(&mut self, action: Action, applied: Rect, fraction: Fraction, at: Instant) {
        self.last = Some(Last {
            action,
            frame: applied,
            fraction,
            at,
        });
    }

    pub fn reset(&mut self) {
        self.last = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RULES: Rules = Rules {
        cycle_thirds: true,
        chain_window: Duration::from_secs(3),
    };
    const NO_CYCLE: Rules = Rules {
        cycle_thirds: false,
        chain_window: Duration::from_secs(3),
    };

    fn rect() -> Rect {
        Rect::new(0.0, 0.0, 10.0, 10.0)
    }

    #[test]
    fn first_press_is_half_from_screen() {
        let s = CycleState::default();
        assert_eq!(
            s.plan(Action::Left, &rect(), Instant::now(), RULES),
            Plan::FromScreen(Fraction::Half)
        );
    }

    #[test]
    fn perpendicular_key_within_window_chains() {
        let mut s = CycleState::default();
        let t0 = Instant::now();
        s.record(Action::Left, rect(), Fraction::Half, t0);
        assert_eq!(
            s.plan(Action::Down, &rect(), t0 + Duration::from_secs(2), NO_CYCLE),
            Plan::Chained
        );
        // 3초를 넘기면 화면 기준.
        assert_eq!(
            s.plan(Action::Down, &rect(), t0 + Duration::from_secs(4), NO_CYCLE),
            Plan::FromScreen(Fraction::Half)
        );
        // 창이 움직였으면 화면 기준.
        let moved = Rect::new(5.0, 0.0, 10.0, 10.0);
        assert_eq!(
            s.plan(Action::Down, &moved, t0, NO_CYCLE),
            Plan::FromScreen(Fraction::Half)
        );
        // 끄면 화면 기준.
        let off = Rules {
            cycle_thirds: false,
            chain_window: Duration::ZERO,
        };
        assert_eq!(
            s.plan(Action::Down, &rect(), t0, off),
            Plan::FromScreen(Fraction::Half)
        );
    }

    #[test]
    fn same_axis_never_chains() {
        let mut s = CycleState::default();
        let t0 = Instant::now();
        s.record(Action::Left, rect(), Fraction::Half, t0);
        assert_eq!(
            s.plan(Action::Right, &rect(), t0, NO_CYCLE),
            Plan::FromScreen(Fraction::Half)
        );
        assert_eq!(
            s.plan(Action::Left, &rect(), t0, NO_CYCLE),
            Plan::FromScreen(Fraction::Half)
        );
    }

    #[test]
    fn repeats_cycle_and_wrap_without_time_limit() {
        let mut s = CycleState::default();
        let t0 = Instant::now();
        s.record(Action::Left, rect(), Fraction::Half, t0);
        let later = t0 + Duration::from_secs(60);
        assert_eq!(
            s.plan(Action::Left, &rect(), later, RULES),
            Plan::FromScreen(Fraction::TwoThirds)
        );
        s.record(Action::Left, rect(), Fraction::TwoThirds, later);
        assert_eq!(
            s.plan(Action::Left, &rect(), later, RULES),
            Plan::FromScreen(Fraction::Third)
        );
        s.record(Action::Left, rect(), Fraction::Third, later);
        assert_eq!(
            s.plan(Action::Left, &rect(), later, RULES),
            Plan::FromScreen(Fraction::Half)
        );
        // 순환이 꺼져 있으면 항상 1/2.
        assert_eq!(
            s.plan(Action::Left, &rect(), later, NO_CYCLE),
            Plan::FromScreen(Fraction::Half)
        );
    }
}
