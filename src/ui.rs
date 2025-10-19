use std::io::{self, Write};

/// 用户界面工具
pub struct UI;

impl UI {
    /// 清屏
    pub fn clear_screen() {
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();
    }

    /// 打印分隔线
    pub fn print_separator() {
        println!("\n{}", "=".repeat(80));
    }

    /// 打印标题
    pub fn print_title(title: &str) {
        Self::print_separator();
        println!("  {}", title);
        Self::print_separator();
    }

    /// 打印子标题
    pub fn print_subtitle(subtitle: &str) {
        println!("\n--- {} ---", subtitle);
    }

    /// 获取用户输入
    pub fn get_input(prompt: &str) -> String {
        print!("{}", prompt);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input.trim().to_string()
    }

    /// 获取用户数字输入
    pub fn get_number_input(prompt: &str, min: usize, max: usize) -> Option<usize> {
        loop {
            let input = Self::get_input(prompt);

            if input.is_empty() {
                return None;
            }

            match input.parse::<usize>() {
                Ok(num) if num >= min && num <= max => return Some(num),
                _ => println!("❌ 请输入 {} 到 {} 之间的数字", min, max),
            }
        }
    }

    /// 等待用户按回车继续
    pub fn wait_for_enter(message: &str) {
        print!("{}", message);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
    }

    /// 显示菜单并获取选择
    pub fn show_menu(title: &str, options: &[&str]) -> usize {
        Self::print_subtitle(title);
        for (i, option) in options.iter().enumerate() {
            println!("  [{}] {}", i + 1, option);
        }

        loop {
            let choice = Self::get_number_input("\n请选择 (输入序号): ", 1, options.len());
            if let Some(choice) = choice {
                return choice - 1;
            }
        }
    }

    /// 确认操作
    pub fn confirm(message: &str) -> bool {
        loop {
            let input = Self::get_input(&format!("{} (y/n): ", message));
            match input.to_lowercase().as_str() {
                "y" | "yes" | "是" => return true,
                "n" | "no" | "否" => return false,
                _ => println!("❌ 请输入 y 或 n"),
            }
        }
    }

    /// 显示成功消息
    pub fn success(message: &str) {
        println!("✅ {}", message);
    }

    /// 显示错误消息
    pub fn error(message: &str) {
        println!("❌ {}", message);
    }

    /// 显示信息消息
    pub fn info(message: &str) {
        println!("ℹ️  {}", message);
    }

    /// 显示警告消息
    pub fn warning(message: &str) {
        println!("⚠️  {}", message);
    }
}
