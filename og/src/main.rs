//! 生成静态社交分享图 `frontend/og.png`（蟹记 Crabnote，Logo A 蟹钳）。
//!
//! 版式与原 `generate_og.py` 一致：1200×630，坐标、字号、颜色都沿用。
//! 字体：英文用仓库里的 Bricolage Grotesque（`frontend/assets/bricolage-latin.woff2`，OFL，运行时解成 TTF），
//! 中文用系统的 Noto Sans CJK SC（可用环境变量 `OG_CJK_FONT` 指定路径）。
//!
//! 运行：`cargo run -p og-gen --release`

use std::path::{Path, PathBuf};

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 630;

const BG: &str = "#f4efe7";
const INK: &str = "#1a1612";
const INK_2: &str = "#5a5148";
const RUST: &str = "#b7472a";
const LINE: &str = "#e4dacd";
const DOT: &str = "#ddd3c6";

const DEFAULT_CJK_FONTS: &[&str] = &[
    "/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc",
    "/usr/share/fonts/noto-cjk/NotoSansCJK-Bold.ttc",
];

fn frontend_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../frontend")
}

/// Logo A：缺口圆（右上方缺 60°）+ 钳口圆点，几何与 logo/a-crabnote-claw.svg 一致。
fn claw(cx: f64, cy: f64, r: f64) -> String {
    let rad = 30f64.to_radians();
    let (ex, ey) = (cx + r * rad.cos(), cy - r * rad.sin());
    let k = r / 28.0;
    let (dx, dy, dr) = (cx + 14.0 * k, cy - 22.0 * k, 6.0 * k);
    format!(
        r#"<path fill="{RUST}" d="M{cx} {top} A{r} {r} 0 1 0 {ex:.2} {ey:.2} L{cx} {cy} Z"/><circle fill="{RUST}" cx="{dx:.2}" cy="{dy:.2}" r="{dr:.2}"/>"#,
        top = cy - r
    )
}

/// 点阵底纹：只放在右上留白区，不压字。
fn dots() -> String {
    let mut out = String::new();
    for x in (36..WIDTH).step_by(24) {
        for y in (36..HEIGHT).step_by(24) {
            if x > 760 && y < 300 {
                out.push_str(&format!(r#"<circle cx="{x}" cy="{y}" r="1" fill="{DOT}"/>"#));
            }
        }
    }
    out
}

fn text(x: u32, y: u32, size: u32, weight: u32, fill: &str, family: &str, body: &str) -> String {
    format!(
        r#"<text x="{x}" y="{y}" font-family="'{family}'" font-size="{size}" font-weight="{weight}" font-variation-settings="'opsz' 96" fill="{fill}">{body}</text>"#
    )
}

pub fn svg(latin: &str) -> String {
    let cjk = "Noto Sans CJK SC";
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{HEIGHT}" viewBox="0 0 {WIDTH} {HEIGHT}">
<rect width="{WIDTH}" height="{HEIGHT}" fill="{BG}"/>
{dots}
{claw}
{title}
{zh}
{sub}
<rect x="80" y="519" width="1040" height="2" fill="{LINE}"/>
{url}
</svg>"#,
        dots = dots(),
        claw = claw(158.0, 250.0, 78.0),
        title = text(270, 312, 132, 700, INK, latin, "Crabnote"),
        zh = text(276, 392, 44, 700, RUST, cjk, "蟹记 · Rust 全栈待办示例"),
        sub = text(80, 470, 32, 500, INK_2, latin, "A full-stack Rust to-do demo — Yew (WebAssembly) + actix-web"),
        url = text(80, 568, 28, 600, RUST, latin, "todos.lexcao.io"),
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut opt = resvg::usvg::Options::default();
    let db = opt.fontdb_mut();

    let woff2 = std::fs::read(frontend_dir().join("assets/bricolage-latin.woff2"))?;
    let ttf = woff2_patched::decode::convert_woff2_to_ttf(&mut woff2.as_slice())
        .map_err(|e| format!("woff2 解码失败: {e:?}"))?;
    db.load_font_data(ttf);
    // woff2 里的家族名带着默认实例名（如 "Bricolage Grotesque 96pt ExtraBold"），按实际名字引用。
    let latin = db
        .faces()
        .flat_map(|face| face.families.iter().map(|(name, _)| name.clone()))
        .find(|name| name.starts_with("Bricolage Grotesque"))
        .ok_or("woff2 里没找到 Bricolage Grotesque")?;

    let cjk = std::env::var("OG_CJK_FONT")
        .ok()
        .map(PathBuf::from)
        .or_else(|| DEFAULT_CJK_FONTS.iter().map(PathBuf::from).find(|p| p.exists()))
        .ok_or("找不到 Noto Sans CJK 字体，请用 OG_CJK_FONT 指定")?;
    db.load_font_file(&cjk)?;

    let tree = resvg::usvg::Tree::from_str(&svg(&latin), &opt)?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(WIDTH, HEIGHT).ok_or("pixmap")?;
    resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());

    let out = std::env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| frontend_dir().join("og.png"));
    pixmap.save_png(&out)?;
    println!("{} ({WIDTH}x{HEIGHT})", out.display());
    Ok(())
}
