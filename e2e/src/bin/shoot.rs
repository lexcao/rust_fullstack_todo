//! 截图：桌面 1440 / 手机 390 / 手机 360，浅色和深色；各截空状态、有数据（视口）和整页。
//!
//! 用法：`cargo run -p e2e --bin shoot -- <前缀> <输出目录>`，站点地址同测试（`E2E_BASE_URL` 或本地 dist）。

use std::path::PathBuf;
use std::time::Duration;

use e2e::*;
use fantoccini::Locator;

const TASKS: [&str; 4] = [
    "Read the Yew chapter",
    "Wire actix-web to Postgres",
    "Ship the wasm bundle",
    "Write the retro",
];

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let prefix = args.next().unwrap_or_else(|| "local".into());
    let out = PathBuf::from(args.next().unwrap_or_else(|| "shots".into()));
    std::fs::create_dir_all(&out).unwrap();

    let runs = [
        Viewport::DESKTOP,
        Viewport::MOBILE,
        Viewport::MOBILE_SMALL,
        Viewport::DESKTOP.dark("desktop-dark"),
        Viewport::MOBILE.dark("mobile-dark"),
    ];
    for vp in runs {
        let c = session(vp).await;
        open_app(&c).await;
        // 等后端探测超时，显示离线提示
        tokio::time::sleep(OFFLINE_WAIT).await;
        let save = |name: String, png: Vec<u8>| {
            let path = out.join(format!("{prefix}-{name}.png"));
            std::fs::write(&path, png).unwrap();
            println!("{}", path.display());
        };
        save(format!("{}-empty", vp.name), c.screenshot().await.unwrap());

        for t in TASKS {
            add_todo(&c, t).await;
        }
        let checks = c.find_all(Locator::Css(".check")).await.unwrap();
        checks[3].click().await.unwrap();
        checks[2].click().await.unwrap();
        c.find(Locator::Css("#new-todo"))
            .await
            .unwrap()
            .click()
            .await
            .ok();
        c.execute(
            "document.activeElement.blur(); window.scrollTo(0, 0)",
            vec![],
        )
        .await
        .unwrap();
        tokio::time::sleep(Duration::from_millis(700)).await;

        let summary = format!(
            "{}: title={:?} todos={} overflow_x={} errors={:?}",
            vp.name,
            c.title().await.unwrap(),
            count(&c, ".todo").await,
            has_horizontal_overflow(&c).await,
            unexpected_errors(&severe_logs(&c).await),
        );
        eprintln!("{summary}");
        save(vp.name.to_string(), c.screenshot().await.unwrap());
        // 先拉高视口再截面板，否则手机上面板会被视口裁掉
        expand_to_page(&c, vp).await;
        save(format!("{}-full", vp.name), c.screenshot().await.unwrap());
        save(
            format!("{}-board", vp.name),
            c.find(Locator::Css(".board"))
                .await
                .unwrap()
                .screenshot()
                .await
                .unwrap(),
        );
        set_metrics(&c, vp, vp.height).await;
        c.close().await.unwrap();
    }
}
