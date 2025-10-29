use std::io::{stdout, Write};
use tokio::time::{sleep, Duration};
use crossterm::{style::{Color, SetForegroundColor, ResetColor}, ExecutableCommand};
use figlet_rs::FIGfont;
use tokio::sync::watch::Receiver;

pub async fn startup_message(app_name: &str, ready_signal: Receiver<i32>) {
    let spinner = ['|', '/', '-', '\\'];
    let mut msg_index = 0;

    let standard_font = FIGfont::standard().unwrap();
    let figure = standard_font.convert(app_name).unwrap();

    stdout().execute(SetForegroundColor(Color::Rgb { r: 234, g: 53, b: 147 })).unwrap();
    println!("{}", figure);
    stdout().execute(ResetColor).unwrap();

    let info_lines = [
        "Date: 17-09-2025",
        "Version: 1.0.0+4",
        "Author: Dev Femi Badmus",
        "Support: femi.badmus@tolaram.com",
        " ",
    ];

    for line in &info_lines {
        for (_i, c) in line.chars().enumerate() {
            print!("{}", c);
            stdout().flush().unwrap();
            sleep(Duration::from_millis(50)).await;
        }
        println!();
        sleep(Duration::from_millis(200)).await;
    }

    let loading_text = "Loading...";
    let mut port ;

    loop {
        port = *ready_signal.borrow();
        for &s in &spinner {
            if port != 0{
                break;
            }

            if msg_index < loading_text.len() {
                print!("\r{} {}", &loading_text[..=msg_index], s);
                msg_index += 1;
            } else {
                print!("\r{} {}", loading_text, s);
            }
            stdout().flush().unwrap();
            sleep(Duration::from_millis(100)).await;
        }
        if port != 0{
            break;
        }
    }
    stdout().execute(SetForegroundColor(Color::Green)).unwrap();
    print!("Application Started ✅ : ");
    stdout().execute(SetForegroundColor(Color::White)).unwrap();
    println!("http://localhost:{}", port);
    // webbrowser::open(&format!("http://localhost:{}", port)).ok();
    stdout().execute(ResetColor).unwrap();
}
