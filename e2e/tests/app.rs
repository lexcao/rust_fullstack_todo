//! 浏览器端到端测试。需要 Chrome + chromedriver，所以默认 `#[ignore]`：
//!
//! ```sh
//! (cd frontend && trunk build --release)
//! cargo test -p e2e -- --ignored            # 测本地 frontend/dist
//! E2E_BASE_URL='https://todos.lexcao.io/?v=1' cargo test -p e2e -- --ignored   # 测生产
//! ```

use std::time::{Duration, Instant};

use e2e::*;
use fantoccini::actions::{InputSource, KeyAction, KeyActions};
use fantoccini::key::Key;
use fantoccini::{Client, Locator};

async fn click(client: &Client, css: &str) {
    client.find(Locator::Css(css)).await.unwrap_or_else(|e| panic!("找不到 {css}: {e}")).click().await.unwrap();
}

async fn click_tab(client: &Client, label: &str) {
    let xpath = format!("//button[contains(@class,'tab')][span[1][normalize-space()='{label}']]");
    client.find(Locator::XPath(&xpath)).await.unwrap().click().await.unwrap();
}

async fn first_text(client: &Client) -> String {
    client.find(Locator::Css(".todo-text")).await.unwrap().text().await.unwrap()
}

async fn type_keys(client: &Client, text: &str) {
    client.active_element().await.unwrap().send_keys(text).await.unwrap();
}

/// 原 Playwright e2e.js 的 16 项，顺序执行（后一步依赖前一步的状态）。
#[tokio::test]
#[ignore = "需要 Chrome + chromedriver，用 --ignored 运行"]
async fn todo_flow_16_checks() {
    let c = session(Viewport { name: "flow", width: 1280, height: 900, mobile: false, dark: false }).await;
    open_app(&c).await;

    // 1 用回车添加 3 条
    for t in ["a1", "a2", "a3"] {
        add_todo(&c, t).await;
    }
    assert_eq!(count(&c, ".todo").await, 3, "1 回车添加 3 条");

    // 2 空白输入被忽略
    c.find(Locator::Css("#new-todo")).await.unwrap().send_keys("   ").await.unwrap();
    click(&c, ".btn-primary").await;
    assert_eq!(count(&c, ".todo").await, 3, "2 空白输入被忽略");

    // 3 编辑时输入框自动聚焦
    click(&c, ".todo [data-tip='Edit']").await;
    let active_class = c.active_element().await.unwrap().attr("class").await.unwrap().unwrap_or_default();
    assert!(active_class.contains("todo-input"), "3 编辑输入框自动聚焦，实际: {active_class}");

    // 4 回车保存编辑（输入框内容已全选，直接输入即替换）
    type_keys(&c, &format!("a3-edited{}", char::from(Key::Enter))).await;
    assert_eq!(first_text(&c).await, "a3-edited", "4 回车保存编辑");

    // 5 Esc 取消编辑
    click(&c, ".todo [data-tip='Edit']").await;
    type_keys(&c, &format!("zzz{}", char::from(Key::Escape))).await;
    assert_eq!(first_text(&c).await, "a3-edited", "5 Esc 取消编辑");

    // 6 勾选完成（aria-checked）
    click(&c, ".check").await;
    let checked = c.find(Locator::Css(".check")).await.unwrap().attr("aria-checked").await.unwrap();
    assert_eq!(checked.as_deref(), Some("true"), "6 勾选后 aria-checked=true");

    // 7 进度条用真实计数
    let progress = c.find(Locator::Css(".progress-label")).await.unwrap().text().await.unwrap();
    assert_eq!(progress, "1 of 3 done", "7 进度条文字");

    // 8 归档第二条，Archived 分组显示 1 条
    c.find_all(Locator::Css(".todo")).await.unwrap()[1]
        .find(Locator::Css("[data-tip='Archive']")).await.unwrap()
        .click().await.unwrap();
    click_tab(&c, "Archived").await;
    assert_eq!(count(&c, ".todo").await, 1, "8 Archived 分组 1 条");

    // 9 移到回收站，Bin 分组显示 1 条
    click(&c, "[data-tip='Move to bin']").await;
    click_tab(&c, "Bin").await;
    assert_eq!(count(&c, ".todo").await, 1, "9 Bin 分组 1 条");

    // 10 清空回收站后显示空状态
    c.find(Locator::XPath("//button[normalize-space()='Empty the bin']")).await.unwrap().click().await.unwrap();
    c.find(Locator::XPath("//button[normalize-space()='Yes, empty the bin']")).await.unwrap().click().await.unwrap();
    assert_eq!(count(&c, ".empty").await, 1, "10 清空后显示空状态");

    // 11 清空后再添加，本地 id 不重复
    click_tab(&c, "All").await;
    add_todo(&c, "a4").await;
    add_todo(&c, "a5").await;
    let stored = c
        .execute(&format!("return localStorage.getItem('{TODOS_KEY}')"), vec![])
        .await
        .unwrap();
    let todos: Vec<serde_json::Value> = serde_json::from_str(stored.as_str().unwrap()).unwrap();
    let mut ids: Vec<i64> = todos.iter().map(|t| t["id"].as_i64().unwrap()).collect();
    let n = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), n, "11 清空回收站后 id 不重复: {ids:?}");

    // 12 刷新后任务还在
    c.refresh().await.unwrap();
    wait_for(&c, ".todo").await;
    assert_eq!(count(&c, ".todo").await, 4, "12 刷新后保留 4 条");

    // 13 第一次按 Tab 落在跳转链接
    let tab = char::from(Key::Tab);
    c.perform_actions(KeyActions::new("kb".into()).then(KeyAction::Down { value: tab }).then(KeyAction::Up { value: tab }))
        .await
        .unwrap();
    let active = c.active_element().await.unwrap().attr("class").await.unwrap().unwrap_or_default();
    assert_eq!(active, "skip-link", "13 首个 Tab 到 skip link");

    // 14 / 15 后端离线：出现演示模式提示，Cloud 选项禁用
    let deadline = Instant::now() + OFFLINE_WAIT;
    while count(&c, ".notice[role=status]").await == 0 {
        assert!(Instant::now() < deadline, "15 演示模式提示没有出现");
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    let cloud = &c.find_all(Locator::Css(".switch-option")).await.unwrap()[1];
    assert!(cloud.attr("disabled").await.unwrap().is_some(), "14 后端离线时 Cloud 禁用");

    // 16 没有页面异常（离线 ping 的失败是预期内的）
    let errors = unexpected_errors(&severe_logs(&c).await);
    assert!(errors.is_empty(), "16 页面报错: {errors:?}");

    c.close().await.unwrap();
}

/// 原 tabs-check.js：360 / 390 / 1440 三个宽度，每个 5 项。
#[tokio::test]
#[ignore = "需要 Chrome + chromedriver，用 --ignored 运行"]
async fn filter_tabs_fit_without_scrolling() {
    for vp in [Viewport::MOBILE_SMALL, Viewport::MOBILE, Viewport::DESKTOP] {
        let w = vp.width;
        let c = session(vp).await;
        open_app(&c).await;
        for t in ["Read the Yew chapter", "Wire actix-web to Postgres", "Ship the wasm bundle"] {
            add_todo(&c, t).await;
        }
        c.find_all(Locator::Css(".check")).await.unwrap()[2].click().await.unwrap();
        c.find_all(Locator::Css(".todo")).await.unwrap()[1]
            .find(Locator::Css("[data-tip='Archive']")).await.unwrap()
            .click().await.unwrap();

        let nav = c.find(Locator::Css(".tabs")).await.unwrap();
        let (nx, _, nw, _) = nav.rectangle().await.unwrap();
        let scroll_w: f64 = num_prop(&c, &nav, "scrollWidth").await;
        let client_w: f64 = num_prop(&c, &nav, "clientWidth").await;
        assert!(scroll_w <= client_w + 0.5 && !has_horizontal_overflow(&c).await, "{w}px 标签需要横滑或页面横向溢出");

        let tabs = c.find_all(Locator::Css(".tab")).await.unwrap();
        assert_eq!(tabs.len(), 5);
        for tab in &tabs {
            let (x, _, tw, th) = tab.rectangle().await.unwrap();
            let label = tab.find(Locator::Css("span")).await.unwrap();
            let name = label.text().await.unwrap();
            assert!(x >= nx - 0.5 && x + tw <= nx + nw + 0.5, "{w}px 标签 {name} 超出可视区");
            let lsw: f64 = num_prop(&c, &label, "scrollWidth").await;
            let lcw: f64 = num_prop(&c, &label, "clientWidth").await;
            assert!(lsw <= lcw + 0.5, "{w}px 标签 {name} 文字被截断");
            assert!(th >= 44.0 && (!vp.mobile || tw >= 44.0), "{w}px 标签 {name} 触控区 {tw}x{th} < 44");
        }

        for tag in c.find_all(Locator::Css(".todo-tag")).await.unwrap() {
            let (tx, _, _, _) = tag.rectangle().await.unwrap();
            let sw: f64 = num_prop(&c, &tag, "scrollWidth").await;
            let row = tag.find(Locator::XPath("./ancestor::li[1]")).await.unwrap();
            let (rx, _, _, _) = row.rectangle().await.unwrap();
            let parent = tag.find(Locator::XPath("..")).await.unwrap();
            let parent_w = num_prop(&c, &parent, "clientWidth").await;
            assert!(tx >= rx && sw <= parent_w, "{w}px 状态标签溢出行");
        }
        c.close().await.unwrap();
    }
}

/// SEO 头和静态内容（不依赖 JS 也能被爬虫看到）。
#[tokio::test]
#[ignore = "需要 Chrome + chromedriver，用 --ignored 运行"]
async fn brand_and_seo_head() {
    let c = session(Viewport::DESKTOP).await;
    open_app(&c).await;
    let title = c.title().await.unwrap();
    assert_eq!(title, "蟹记 Crabnote｜Rust 全栈待办示例");
    // 入场动画期间 opacity 为 0，WebDriver 的可见文字会是空的，所以读 textContent
    let h1 = c.find(Locator::Css("h1")).await.unwrap().prop("textContent").await.unwrap().unwrap_or_default();
    assert_eq!(h1.split_whitespace().collect::<Vec<_>>(), ["Crabnote", "蟹记"]);
    let og = c.find(Locator::Css("meta[property='og:image']")).await.unwrap().attr("content").await.unwrap();
    assert_eq!(og.as_deref(), Some("https://todos.lexcao.io/og.png"));
    let icon = c.find(Locator::Css("link[rel=icon]")).await.unwrap().attr("href").await.unwrap().unwrap();
    assert!(icon.contains("favicon.svg?v="), "favicon 带版本号: {icon}");
    assert!(!has_horizontal_overflow(&c).await);
    c.close().await.unwrap();
}
