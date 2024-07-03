use reqwest::{self, ClientBuilder, Version};
use reqwest::Client;
use serde_json;
use serde_json::json;
use serde::Serialize;
use anyhow::Result;
use std::fs::File;
use std::io::{self, Read, Write};
use std::error::Error;
use chrono::Utc;
use tokio::time::{sleep, Duration};

async fn send_voucher_code(code: &str, cookie_content: &str, log_file: &mut std::fs::File) -> Result<()> {
	let cookie_content_owned = cookie_content.to_string();

	// Pass the cloned String to extract_csrftoken
	let csrftoken = extract_csrftoken(&cookie_content_owned);
	//println!("csrftoken: {}", csrftoken);
	let csrftoken_string = csrftoken.to_string();

    let body_json = json!({
        "voucher_code": code,
        "need_user_voucher_status": true,
    });
	
	let body_str = serde_json::to_string(&body_json)?;

	println!("{}", body_str);
	writeln!(log_file, "{}", body_str);
	
	let mut headers = reqwest::header::HeaderMap::new();
	headers.insert("User-Agent", reqwest::header::HeaderValue::from_static("Android app Shopee appver=29330 app_type=1"));
	headers.insert("accept", reqwest::header::HeaderValue::from_static("application/json"));
	headers.insert("Content-Type", reqwest::header::HeaderValue::from_static("application/json"));
	headers.insert("x-api-source", reqwest::header::HeaderValue::from_static("rn"));
	headers.insert("if-none-match-", reqwest::header::HeaderValue::from_static("55b03-97d86fe6888b54a9c5bfa268cf3d922f"));
	headers.insert("shopee_http_dns_mode", reqwest::header::HeaderValue::from_static("1"));
	headers.insert("x-shopee-client-timezone", reqwest::header::HeaderValue::from_static("Asia/Jakarta"));
	headers.insert("af-ac-enc-dat", reqwest::header::HeaderValue::from_static(""));
	headers.insert("af-ac-enc-id", reqwest::header::HeaderValue::from_static(""));
	headers.insert("x-sap-access-t", reqwest::header::HeaderValue::from_static(""));
	headers.insert("x-sap-access-f", reqwest::header::HeaderValue::from_static(""));
	headers.insert("referer", reqwest::header::HeaderValue::from_static("https://mall.shopee.co.id/"));
	headers.insert("x-csrftoken", reqwest::header::HeaderValue::from_str(&csrftoken_string)?);
	headers.insert("af-ac-enc-sz-token", reqwest::header::HeaderValue::from_static(""));
	headers.insert(reqwest::header::COOKIE, reqwest::header::HeaderValue::from_str(&cookie_content)?);

	//println!("");
	//println!("header:{:#?}", headers);
	let mut attempt_count = 0;
	let max_attempts = 3; // Ubah angka sesuai kebutuhan Anda
	loop {
		let client = ClientBuilder::new()
			.gzip(true)
			.use_rustls_tls() // Use Rustls for HTTPS
			.build()?;

		// Buat permintaan HTTP POST
		let response = client
			.post("https://mall.shopee.co.id/api/v2/voucher_wallet/save_voucher")
			.header("Content-Type", "application/json")
			.headers(headers.clone())
			.body(body_str.clone())
			.version(Version::HTTP_2) 
			.send()
			.await?;
		// Check for HTTP status code indicating an error
		//let http_version = response.version(); 		// disable output features
		//println!("HTTP Version: {:?}", http_version); // disable output features
		let status = response.status();
		println!("{}", status);
		let text = response.text().await?;	
		if status == reqwest::StatusCode::OK && !text.contains("\"error\":76100003")  {
			writeln!(log_file, "{}", text);
			println!("{}", text);
			println!("Claim Berhasil!");
			sleep(Duration::from_secs(5)).await;
			break;
		} else if status == reqwest::StatusCode::OK && text.contains("\"error\":76100003") {
			println!("Cooldown detected. Waiting for 1 minute...");
			sleep(Duration::from_secs(30)).await;
			continue;
		} else if status == reqwest::StatusCode::IM_A_TEAPOT {
			println!("Gagal, status code: 418 - I'm a teapot. Mencoba kembali...");
			println!("{}", text);
			attempt_count += 1;
			if attempt_count >= max_attempts {
				println!("Batas percobaan maksimum tercapai.");
				break;
			}
			continue;
		}else {
			println!("Status: {}", status);
			break;
		}
	}
	Ok(())
}


fn generate_combinations(prefix: &str, start_point: &str) -> Vec<String> {
    let digits = "0123456789".chars().collect::<Vec<_>>();
    let letters = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect::<Vec<_>>();
    let mut combinations = Vec::new();
    let mut start_collecting = false;

    for &digit in &digits {
        for &letter1 in &letters {
            for &letter2 in &letters {
                let combo = format!("{}{}{}{}", prefix, digit, letter1, letter2);

                if combo.as_str() >= start_point {
                    start_collecting = true;
                }

                if start_collecting {
                    combinations.push(combo);
                }
            }
        }
    }

    combinations
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	println!("-------------------------------------------");
	println!("save_vouchers [Version 1.2.1]");
	println!("");
	println!("Dapatkan Info terbaru di https://google.com");
	println!("");
	println!("-------------------------------------------");

	// Display the list of available cookie files
	println!("Daftar file cookie yang tersedia:");
	let files = std::fs::read_dir("./akun")?;
	let mut file_options = Vec::new();
	for (index, file) in files.enumerate() {
		if let Ok(file) = file {
			let file_name = file.file_name();
			println!("{}. {}", index + 1, file_name.to_string_lossy());
			file_options.push(file_name.to_string_lossy().to_string());
		}
	}
	// Select the file number for the cookie
	let selected_file = loop {
		let input = get_user_input("Pilih nomor file cookie yang ingin digunakan: ");

		// Convert input to index number
		if let Ok(index) = input.trim().parse::<usize>() {
			if index > 0 && index <= file_options.len() {
				break file_options[index - 1].clone();
			}
		}
	};

	// Read the content of the selected cookie file
	let file_path = format!("./akun/{}", selected_file);
	let mut cookie_content = String::new();
	File::open(&file_path)?.read_to_string(&mut cookie_content)?;

    let prefix = "SGV500MP05RVW";
    let start_point = get_user_input("Enter the start point (e.g., SGV500MP05RVW0DX): ");
    let start_time = Utc::now();
    let log_file_name = format!("{}_voucher_log.txt", start_time.format("%Y-%m-%d_%H-%M-%S"));
    let mut log_file = std::fs::File::create(&log_file_name)?;

    println!("Starting voucher generation. Log file: {}", log_file_name);
    let combinations = generate_combinations(prefix, &start_point);

    for combo in combinations {
        send_voucher_code(&combo, &cookie_content, &mut log_file).await?;
    }

    Ok(())
}
fn extract_csrftoken(cookie_string: &str) -> String {
	let mut csrftoken = String::new();
	if let Some(token_index) = cookie_string.find("csrftoken=") {
		let token_start = token_index + "csrftoken=".len();
		if let Some(token_end) = cookie_string[token_start..].find(';') {
			csrftoken = cookie_string[token_start..token_start + token_end].to_string();
		}
	}
	csrftoken
}
fn get_user_input(prompt: &str) -> String {
	print!("{}", prompt);
	io::stdout().flush().unwrap();
	let mut input = String::new();
	io::stdin().read_line(&mut input).unwrap();
	input.trim().to_string()
}