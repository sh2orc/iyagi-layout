//! 개발용 CLI — Tauri 셸 없이 macOS 경로를 검증한다.
//!   cargo run -p layout-probe -- permission | prompt | screens | focused
//!   cargo run -p layout-probe -- left|right|up|down [--gap 8] [--fraction half|two-thirds|third]

use clap::{Parser, Subcommand, ValueEnum};
use layout_core::settings::EnhancedUiPolicy;
use layout_core::{compute_target, pick_screen, Action, Fraction};

#[derive(Parser)]
#[command(name = "layout-probe", about = "IYAGI Layout developer probe")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Copy, ValueEnum)]
enum FractionArg {
    Half,
    TwoThirds,
    Third,
}

#[derive(Subcommand)]
enum Command {
    /// 손쉬운 사용 권한 상태
    Permission,
    /// 권한 안내창 띄우기
    Prompt,
    /// 화면 목록(CG 좌표)
    Screens,
    /// 포커스 창 정보
    Focused,
    /// 로그인 항목 상태
    LoginItem,
    /// 포커스 창을 절반으로 배치
    #[command(name = "left")]
    Left(Place),
    #[command(name = "right")]
    Right(Place),
    #[command(name = "up")]
    Up(Place),
    #[command(name = "down")]
    Down(Place),
}

#[derive(clap::Args)]
struct Place {
    #[arg(long, default_value_t = 0)]
    gap: u32,
    #[arg(long, value_enum, default_value_t = FractionArg::Half)]
    fraction: FractionArg,
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Permission => {
            println!("trusted: {}", layout_macos::permission::is_trusted());
            Ok(())
        }
        Command::Prompt => {
            println!(
                "trusted after prompt: {}",
                layout_macos::permission::prompt()
            );
            Ok(())
        }
        Command::Screens => layout_macos::screens::list().map(|screens| {
            for s in screens {
                println!(
                    "{}{} frame={:?} visible={:?} scale={}",
                    s.id,
                    if s.is_primary { " (primary)" } else { "" },
                    s.frame,
                    s.visible,
                    s.scale
                );
            }
        }),
        Command::Focused => describe_focused().map(|_| ()),
        Command::LoginItem => {
            println!("login item: {:?}", layout_macos::login_item::status());
            Ok(())
        }
        Command::Left(p) => place(Action::Left, p),
        Command::Right(p) => place(Action::Right, p),
        Command::Up(p) => place(Action::Up, p),
        Command::Down(p) => place(Action::Down, p),
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn describe_focused() -> Result<Option<layout_macos::window::WindowHandle>, layout_macos::AxError> {
    let Some(w) = layout_macos::window::focused()? else {
        println!("no focused window");
        return Ok(None);
    };
    println!(
        "pid={} bundle={:?} frame={:?} resizable={} fullscreen={} enhancedUI={:?}",
        w.pid,
        w.bundle_id,
        w.frame()?,
        w.is_resizable(),
        w.is_fullscreen(),
        w.enhanced_ui()
    );
    Ok(Some(w))
}

fn place(action: Action, p: Place) -> Result<(), layout_macos::AxError> {
    layout_macos::ax_timeout();
    let Some(w) = describe_focused()? else {
        return Ok(());
    };
    if w.is_fullscreen() {
        println!("full-screen window — skipped");
        return Ok(());
    }
    let frame = w.frame()?;
    let screens = layout_macos::screens::list()?;
    let screen = pick_screen(&frame, &screens, None)
        .ok_or(layout_macos::AxError::Other("no screens".into()))?;
    let fraction = match p.fraction {
        FractionArg::Half => Fraction::Half,
        FractionArg::TwoThirds => Fraction::TwoThirds,
        FractionArg::Third => Fraction::Third,
    };
    let target = compute_target(action, screen.visible, p.gap as f64, fraction);
    println!("screen={} target={:?}", screen.id, target);
    let actual = w.set_frame(target, EnhancedUiPolicy::Automatic)?;
    println!("actual={:?}", actual);
    Ok(())
}
