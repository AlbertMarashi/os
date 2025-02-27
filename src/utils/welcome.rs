use crate::utils::print::{TextColor, TextStyle};

pub fn print_welcome_message() {
    let message = r#"
╔══════════════════════════════════════════════════════╗
║                                                      ║
║   _    _   _ __  __ ___ _   _    _       ___  ____   ║
║  | |  | | | |  \/  |_ _| \ | |  / \     / _ \/ ___|  ║
║  | |  | | | | |\/| || ||  \| | / _ \   | | | \___ \  ║
║  | |__| |_| | |  | || || |\  |/ ___ \  | |_| |___) | ║
║  |_____\___/|_|  |_|___|_| \_/_/   \_\  \___/|____/  ║
║                                                      ║
╚══════════════════════════════════════════════════════╝
"#;

    for line in message.lines() {
        for c in line.chars() {
            match c {
                '╔' | '╗' | '╚' | '╝' | '═' | '║' => {
                    print!("{}", TextColor::Blue);
                    print!("{}", c);
                }
                _ => {
                    print!("{}", TextColor::Red);
                    print!("{}", c);
                }
            }
        }
        println!("{}", TextStyle::Reset);
    }
}
