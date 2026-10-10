use serde::{Deserialize, Serialize};

use crate::cycle::Fraction;

/// 사각형(CG 좌표, pt).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub const fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }

    pub fn right(&self) -> f64 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f64 {
        self.y + self.h
    }

    pub fn center(&self) -> (f64, f64) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    /// 겹치는 넓이(겹치지 않으면 0).
    pub fn intersection_area(&self, other: &Rect) -> f64 {
        let w = (self.right().min(other.right()) - self.x.max(other.x)).max(0.0);
        let h = (self.bottom().min(other.bottom()) - self.y.max(other.y)).max(0.0);
        w * h
    }

    /// 각 변이 `tolerance` 안에서 같은가(순환 판정용).
    pub fn approx_eq(&self, other: &Rect, tolerance: f64) -> bool {
        (self.x - other.x).abs() <= tolerance
            && (self.y - other.y).abs() <= tolerance
            && (self.w - other.w).abs() <= tolerance
            && (self.h - other.h).abs() <= tolerance
    }
}

/// 화면 하나. `frame`은 전체, `visible`은 메뉴 막대·Dock을 뺀 영역(둘 다 CG 좌표).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Screen {
    pub id: u32,
    pub frame: Rect,
    pub visible: Rect,
    pub is_primary: bool,
    pub scale: f64,
}

/// Cocoa 좌표(왼쪽 아래 원점, y 위 방향)의 사각형을 CG 좌표로 바꾼다.
/// `primary_height`는 주 화면(`NSScreen.screens[0]`) 전체 높이.
pub fn cocoa_to_cg(primary_height: f64, r: Rect) -> Rect {
    Rect {
        x: r.x,
        y: primary_height - (r.y + r.h),
        w: r.w,
        h: r.h,
    }
}

/// 단축키 한 개가 뜻하는 배치.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Left,
    Right,
    Up,
    Down,
}

impl Action {
    pub const ALL: [Action; 4] = [Action::Left, Action::Right, Action::Up, Action::Down];

    pub fn key(self) -> &'static str {
        match self {
            Action::Left => "left",
            Action::Right => "right",
            Action::Up => "up",
            Action::Down => "down",
        }
    }

    /// 가로 축(←→)인가 세로 축(↑↓)인가.
    pub fn is_horizontal(self) -> bool {
        matches!(self, Action::Left | Action::Right)
    }
}

/// 목표 프레임. `visible`은 대상 화면의 가용 영역, `gap`은 가장자리 여백이자 창 사이
/// 간격, `fraction`은 창이 차지할 비율(기본 1/2). 결과는 정수 pt이며 두 절반을 합쳐도
/// `visible`을 넘지 않는다(크기는 내림, 오른쪽/아래 배치는 반대편 가장자리에 붙인다).
pub fn compute_target(action: Action, visible: Rect, gap: f64, fraction: Fraction) -> Rect {
    split(action, visible, gap, gap, fraction)
}

/// 연속 입력(§6-3): 화면이 아니라 **현재 창 프레임**을 다시 나눈다. 바깥 여백은 이미
/// 들어 있으므로 창 사이 간격만 더한다.
pub fn compute_chained(action: Action, current: Rect, gap: f64, fraction: Fraction) -> Rect {
    split(action, current, 0.0, gap, fraction)
}

/// `base`를 `action` 방향으로 나눈다. `edge`는 네 변의 여백, `inner`는 나뉜 두 조각 사이 간격.
fn split(action: Action, base: Rect, edge: f64, inner: f64, fraction: Fraction) -> Rect {
    let e = edge.max(0.0).round();
    let i = inner.max(0.0).round();
    let f = fraction.ratio();
    let inner_w = (base.w - 2.0 * e).max(1.0);
    let inner_h = (base.h - 2.0 * e).max(1.0);
    let split_w = ((base.w - 2.0 * e - i).max(1.0) * f).floor();
    let split_h = ((base.h - 2.0 * e - i).max(1.0) * f).floor();
    // `+ 0.0`은 -0.0을 0.0으로 — 로그와 비교에서 부호만 다른 0을 없앤다.
    let left = (base.x + e).round() + 0.0;
    let top = (base.y + e).round() + 0.0;
    match action {
        Action::Left => Rect::new(left, top, split_w, inner_h.floor()),
        Action::Right => Rect::new(
            (base.right() - e - split_w).round() + 0.0,
            top,
            split_w,
            inner_h.floor(),
        ),
        Action::Up => Rect::new(left, top, inner_w.floor(), split_h),
        Action::Down => Rect::new(
            left,
            (base.bottom() - e - split_h).round() + 0.0,
            inner_w.floor(),
            split_h,
        ),
    }
}

/// 창이 속한 화면: 겹치는 넓이 최대 → 창 중심을 담은 화면 → 마우스 화면 → 주 화면.
pub fn pick_screen<'a>(
    frame: &Rect,
    screens: &'a [Screen],
    mouse: Option<(f64, f64)>,
) -> Option<&'a Screen> {
    if screens.is_empty() {
        return None;
    }
    let best = screens
        .iter()
        .map(|s| (s.frame.intersection_area(frame), s))
        .filter(|(area, _)| *area > 0.0)
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(_, s)| s);
    if best.is_some() {
        return best;
    }
    let (cx, cy) = frame.center();
    if let Some(s) = screens.iter().find(|s| s.frame.contains(cx, cy)) {
        return Some(s);
    }
    if let Some((mx, my)) = mouse {
        if let Some(s) = screens.iter().find(|s| s.frame.contains(mx, my)) {
            return Some(s);
        }
    }
    screens
        .iter()
        .find(|s| s.is_primary)
        .or_else(|| screens.first())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen(id: u32, frame: Rect, primary: bool) -> Screen {
        Screen {
            id,
            frame,
            visible: frame,
            is_primary: primary,
            scale: 2.0,
        }
    }

    #[test]
    fn halves_without_gap_tile_the_visible_area_exactly() {
        let v = Rect::new(0.0, 25.0, 1440.0, 875.0);
        let l = compute_target(Action::Left, v, 0.0, Fraction::Half);
        let r = compute_target(Action::Right, v, 0.0, Fraction::Half);
        assert_eq!(l, Rect::new(0.0, 25.0, 720.0, 875.0));
        assert_eq!(r, Rect::new(720.0, 25.0, 720.0, 875.0));
        let u = compute_target(Action::Up, v, 0.0, Fraction::Half);
        let d = compute_target(Action::Down, v, 0.0, Fraction::Half);
        assert_eq!(u, Rect::new(0.0, 25.0, 1440.0, 437.0));
        assert_eq!(d, Rect::new(0.0, 463.0, 1440.0, 437.0));
        assert!(d.bottom() <= v.bottom());
    }

    #[test]
    fn odd_width_never_overflows() {
        let v = Rect::new(0.0, 0.0, 1001.0, 601.0);
        let l = compute_target(Action::Left, v, 0.0, Fraction::Half);
        let r = compute_target(Action::Right, v, 0.0, Fraction::Half);
        assert_eq!(l.w, 500.0);
        assert_eq!(r.right(), 1001.0);
        assert!(l.right() <= r.x);
    }

    #[test]
    fn gap_applies_to_edges_and_between_halves() {
        let v = Rect::new(100.0, 50.0, 1000.0, 600.0);
        let l = compute_target(Action::Left, v, 8.0, Fraction::Half);
        let r = compute_target(Action::Right, v, 8.0, Fraction::Half);
        assert_eq!(l, Rect::new(108.0, 58.0, 488.0, 584.0));
        assert_eq!(r, Rect::new(604.0, 58.0, 488.0, 584.0));
        assert_eq!(r.x - l.right(), 8.0);
        assert_eq!(v.right() - r.right(), 8.0);
    }

    #[test]
    fn thirds_anchor_to_their_edge() {
        let v = Rect::new(0.0, 0.0, 900.0, 600.0);
        let r = compute_target(Action::Right, v, 0.0, Fraction::Third);
        assert_eq!(r, Rect::new(600.0, 0.0, 300.0, 600.0));
        let d = compute_target(Action::Down, v, 0.0, Fraction::TwoThirds);
        assert_eq!(d, Rect::new(0.0, 200.0, 900.0, 400.0));
    }

    #[test]
    fn chained_split_divides_the_window_and_keeps_outer_edges() {
        let left_half = Rect::new(0.0, 25.0, 720.0, 875.0);
        let q = compute_chained(Action::Down, left_half, 0.0, Fraction::Half);
        assert_eq!(q, Rect::new(0.0, 463.0, 720.0, 437.0));
        // 간격은 조각 사이에만 — 바깥 변은 그대로.
        let padded = Rect::new(8.0, 33.0, 704.0, 859.0);
        let q = compute_chained(Action::Up, padded, 8.0, Fraction::Half);
        assert_eq!(q, Rect::new(8.0, 33.0, 704.0, 425.0));
        let r = compute_chained(Action::Right, padded, 8.0, Fraction::Half);
        assert_eq!(r.right(), padded.right());
        assert_eq!(r.w, 348.0);
    }

    #[test]
    fn cocoa_flip_uses_primary_height() {
        // 주 화면 900pt 높이, 메뉴 막대 25pt: visibleFrame (0,0,1440,875) → CG y=25.
        let v = cocoa_to_cg(900.0, Rect::new(0.0, 0.0, 1440.0, 875.0));
        assert_eq!(v, Rect::new(0.0, 25.0, 1440.0, 875.0));
        // 주 화면 위에 놓인 보조 모니터(Cocoa y=900..1980) → CG y는 음수.
        let above = cocoa_to_cg(900.0, Rect::new(0.0, 900.0, 1920.0, 1080.0));
        assert_eq!(above, Rect::new(0.0, -1080.0, 1920.0, 1080.0));
    }

    #[test]
    fn picks_screen_by_overlap_then_center_then_mouse_then_primary() {
        let a = screen(1, Rect::new(0.0, 0.0, 1000.0, 800.0), true);
        let b = screen(2, Rect::new(1000.0, 0.0, 1000.0, 800.0), false);
        let screens = vec![a.clone(), b.clone()];
        let mostly_b = Rect::new(900.0, 0.0, 400.0, 400.0);
        assert_eq!(pick_screen(&mostly_b, &screens, None).unwrap().id, 2);
        let off = Rect::new(5000.0, 5000.0, 10.0, 10.0);
        assert_eq!(
            pick_screen(&off, &screens, Some((1500.0, 10.0)))
                .unwrap()
                .id,
            2
        );
        assert_eq!(pick_screen(&off, &screens, None).unwrap().id, 1);
        assert!(pick_screen(&off, &[], None).is_none());
    }
}
