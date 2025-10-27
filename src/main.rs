mod cultivation;
mod disciple;
mod task;
mod map;
mod sect;
mod event;
mod game;
mod ui;
mod interactive;
mod api_types;
mod web_server;
mod version;
mod pill;

use interactive::InteractiveGame;
use ui::UI;

#[tokio::main]
async fn main() {
    // 检查命令行参数
    let args: Vec<String> = std::env::args().collect();

    // 如果有 --web 参数，直接启动Web服务器
    if args.len() > 1 && (args[1] == "--web" || args[1] == "-w") {
        println!("🚀 启动 Web 服务器模式...");
        println!("📍 服务器地址: http://localhost:3000");
        println!("📚 API 文档: 见 API_DESIGN.md");
        println!("⛔ 按 Ctrl+C 停止服务器\n");

        web_server::start_server().await;
        return;
    }

    // 欢迎界面
    UI::clear_screen();
    UI::print_title("修仙宗门模拟器");

    println!("\n欢迎来到修仙世界！");
    println!("\n请选择游戏模式:");
    println!("  [1] 交互模式 - 回合制，手动管理宗门");
    println!("  [2] 自动模式 - 自动运行，观察模拟结果");
    println!("  [3] Web服务器 - 启动HTTP API服务器");
    println!("\n提示: 也可以使用 'cargo run --release -- --web' 直接启动Web服务器");

    let mode = UI::get_number_input("\n请选择 (1-3): ", 1, 3);

    match mode {
        Some(1) => {
            // 交互式游戏
            UI::print_subtitle("交互模式");
            let sect_name = UI::get_input("请输入宗门名称: ");
            let sect_name = if sect_name.is_empty() {
                "青云宗".to_string()
            } else {
                sect_name
            };

            let mut game = InteractiveGame::new(sect_name);
            game.run();
        }
        Some(2) => {
            // 自动模式
            UI::print_subtitle("自动模式");
            let sect_name = UI::get_input("请输入宗门名称 (留空默认为青云宗): ");
            let sect_name = if sect_name.is_empty() {
                "青云宗".to_string()
            } else {
                sect_name
            };

            let years = UI::get_number_input("模拟年数 (建议100): ", 1, 1000);
            let years = years.unwrap_or(100);

            UI::wait_for_enter("\n按回车开始模拟...");

            let mut game = game::Game::new(sect_name);
            game.run(years as u32);
        }
        Some(3) => {
            // Web服务器模式
            println!("\n🚀 启动 Web 服务器模式...");
            println!("📍 服务器地址: http://localhost:3000");
            println!("📚 API 文档: 见 API_DESIGN.md");
            println!("⛔ 按 Ctrl+C 停止服务器\n");

            web_server::start_server().await;
        }
        _ => {
            UI::error("无效选择");
        }
    }
}
