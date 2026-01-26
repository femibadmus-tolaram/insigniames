use serde::{Deserialize, Serialize};
use serialport;
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct Scale {
    port_name: String,
    baud_rate: u32,
}

impl Scale {
    pub fn new(port_name: String, baud_rate: u32) -> Self {
        Self {
            port_name,
            baud_rate,
        }
    }

    pub fn test_connection(&self) -> Result<f32, String> {
        if self.port_name.is_empty() {
            return Err("Scale port not configure, call the IT".to_string());
        }

        self.reset_com_port()?;

        let mut port = serialport::new(&self.port_name, self.baud_rate)
            .timeout(std::time::Duration::from_millis(3000))
            .open()
            .map_err(|_| "Please try another port")?;

        for attempt in 0..10 {
            let mut buffer = [0u8; 100];

            match port.read(&mut buffer) {
                Ok(n) => {
                    if n > 1 {
                        let response = String::from_utf8_lossy(&buffer[..n]);

                        if let Some(weight) = self.extract_weight_from_text(&response) {
                            drop(port);
                            self.reset_com_port().ok();
                            return Ok(weight);
                        }
                    }
                }
                Err(_) => break,
            }

            if attempt < 9 {
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }

        drop(port);
        self.reset_com_port().ok();
        Err("No weight data found".to_string())
    }

    pub fn get_weight(&self) -> Result<f32, String> {
        if self.port_name.is_empty() {
            return Err("Scale port not configure, call the IT".to_string());
        }

        self.reset_com_port()?;

        let mut port = serialport::new(&self.port_name, self.baud_rate)
            .timeout(std::time::Duration::from_millis(3000))
            .open()
            .map_err(|_| "Replug the scale USB")?;

        for attempt in 0..10 {
            let mut buffer = [0u8; 100];

            match port.read(&mut buffer) {
                Ok(n) => {
                    if n > 1 {
                        let response = String::from_utf8_lossy(&buffer[..n]);

                        if let Some(weight) = self.extract_weight_from_text(&response) {
                            drop(port);
                            self.reset_com_port().ok();
                            return Ok(weight);
                        }
                    }
                }
                Err(_) => break,
            }

            if attempt < 9 {
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }

        drop(port);
        self.reset_com_port().ok();
        Err("No weight data found".to_string())
    }

    fn reset_com_port(&self) -> Result<(), String> {
        std::thread::sleep(std::time::Duration::from_millis(100));

        let mut cmd = std::process::Command::new("cmd");
        cmd.args(&["/C", &format!("mode {}:115200,N,8,1,P", self.port_name)]);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        let output = cmd
            .output()
            .map_err(|e| format!("Failed to reset COM port: {}", e))?;

        if !output.status.success() {
            println!(
                "[NOTE] Reset command completed with status: {}",
                output.status
            );
        }

        Ok(())
    }

    fn extract_weight_from_text(&self, text: &str) -> Option<f32> {
        use regex::Regex;

        let patterns = [r"-?\d+\.\d+", r"-?\d+", r"[SW]\s*:?\s*(\d+\.\d+)"];

        for pattern in patterns.iter() {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(text) {
                    let matched = caps.get(1).map_or(caps.get(0), Some);

                    if let Some(m) = matched {
                        if let Ok(weight) = m.as_str().parse::<f32>() {
                            return Some(weight);
                        }
                    }
                }
            }
        }

        None
    }
}

pub fn check_printer() -> bool {
    let data_dir = Path::new("data");
    let pdf_path = data_dir.join("test.pdf");
    let pdf_exe_path = data_dir.join("pdf.exe");

    if !pdf_path.exists() || !pdf_exe_path.exists() {
        return false;
    }

    match Command::new(pdf_exe_path)
        .args(&[
            "-print-to-default",
            pdf_path.to_str().unwrap(),
            "-print-settings",
            "landscape",
            "-exit-when-done",
            "-silent",
        ])
        .output()
    {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

pub fn print_pdf(pdf_data: &[u8]) -> Result<(), String> {
    let data_dir = Path::new("data");
    let pdf_path = data_dir.join("temp_print.pdf");
    let pdf_exe_path = data_dir.join("pdf.exe");

    if !pdf_exe_path.exists() {
        return Err("SumatraPDF not found".to_string());
    }

    fs::write(&pdf_path, pdf_data).map_err(|e| format!("Failed to save PDF: {}", e))?;

    let output = Command::new(pdf_exe_path)
        .args(&[
            "-print-to-default",
            pdf_path.to_str().unwrap(),
            "-print-settings",
            "portrait",
            "-exit-when-done",
            "-silent",
        ])
        .output()
        .map_err(|e| format!("Print failed: {}", e))?;

    let _ = fs::remove_file(pdf_path);

    if output.status.success() {
        Ok(())
    } else {
        Err("Print job failed".to_string())
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct DeviceConfig {
    pub port_name: String,
    pub baud_rate: u32,
}

impl DeviceConfig {
    pub fn from_env(device_type: &str) -> Self {
        let env_path = "data/.env";
        let mut port_name = String::new();
        let mut baud_rate = 9600;

        if let Ok(content) = std::fs::read_to_string(env_path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    match key.trim() {
                        k if k == format!("{}_PORT_NAME", device_type) => {
                            port_name = value.trim().to_string()
                        }
                        k if k == format!("{}_BAUD_RATE", device_type) => {
                            baud_rate = value.trim().parse().unwrap_or(9600)
                        }
                        _ => {}
                    }
                }
            }
        }

        Self {
            port_name,
            baud_rate,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AppConfig;

impl AppConfig {
    pub fn scanner() -> DeviceConfig {
        DeviceConfig::from_env("SCANNER")
    }

    pub fn scale() -> DeviceConfig {
        DeviceConfig::from_env("SCALE")
    }

    pub fn get_scale() -> Scale {
        let scale = Self::scale();
        Scale::new(scale.port_name, scale.baud_rate)
    }

    pub fn has_scanner() -> bool {
        !Self::scanner().port_name.is_empty()
    }

    pub fn has_scale() -> bool {
        !Self::scale().port_name.is_empty()
    }

    pub fn test_scanner_connection() -> Result<(), String> {
        let scanner = Self::scanner();
        if scanner.port_name.is_empty() {
            return Err("Scanner port not configured".to_string());
        }
        serialport::new(&scanner.port_name, scanner.baud_rate)
            .timeout(std::time::Duration::from_secs(3))
            .open()
            .map_err(|e| format!("Scanner connection failed: {}", e))?;
        Ok(())
    }

    pub fn save_to_env(scanner: DeviceConfig, scale: DeviceConfig) {
        let content = format!(
            "SCANNER_PORT_NAME={}\n\
             SCANNER_BAUD_RATE={}\n\
             SCALE_PORT_NAME={}\n\
             SCALE_BAUD_RATE={}\n",
            scanner.port_name, scanner.baud_rate, scale.port_name, scale.baud_rate
        );

        if let Err(e) = std::fs::write("data/.env", content) {
            eprintln!("Failed to update .env: {}", e);
        }
    }
}
