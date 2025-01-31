use crate::config::SETTINGS;
use base64::Engine;
use octocrab::{auth::{DeviceCodes, OAuth}, Octocrab};
use chrono;

use http::header::ACCEPT;
use secrecy::SecretBox;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::sync::OnceLock;
pub static OCTOCRAB: OnceLock<Octocrab> = OnceLock::new();
pub static SECRET_ID: OnceLock<SecretBox<str>> = OnceLock::new();
pub static CODES: OnceLock<DeviceCodes> = OnceLock::new();
use gtk::prelude::*;
use gtk::{prelude::ApplicationExtManual, ButtonsType, DialogFlags, MessageDialog, MessageType};

pub async fn test() -> Result<String,Box<dyn std::error::Error + Send + Sync>> {
    
    let client_id = "Iv23li4MQkmwBaY6lnva".to_string();

    println!("HELLOO");

    let secret_id = secrecy::SecretString::from(client_id);
    println!("HELLOO");
    let crab = octocrab::Octocrab::builder()
        .base_uri("https://github.com")?
        .add_header(ACCEPT, "application/json".to_string())
        .build()?;

        println!("HELLOO");
    let codes = crab
        .authenticate_as_device(&secret_id, ["public_repo", "read:org"])
        .await?;

    OCTOCRAB.set(crab);

    println!(
        "Go to {} and enter code {}",
        codes.verification_uri, codes.user_code
    );
    SECRET_ID.set(secret_id);
    //code = codes.user_code.clone();
    println!("before Wait spawn");

    let code = codes.user_code.clone();

    CODES.set(codes);

    return Ok(code)
}

pub async fn wait() -> octocrab::Result<OAuth> {
    println!("HEllooWW!!!");
    let crab = OCTOCRAB.get().expect("CRAB FUCKED");
    let client_id = SECRET_ID.get().expect("ID FUCKED");
    let code = CODES.get().expect("ID FUCKED");
    let auth = code.poll_until_available(&crab, &client_id).await?;

    println!("TEST!!!");
    Ok(auth)
}

pub async fn create_github_issue(content: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let settings = SETTINGS.get().unwrap();

    //Octocrab::installation(&octocrab, octocrab::models::InstallationId(1110893));
    
    let octocrab = Octocrab::builder()
        .personal_token(settings.github.token.clone())
        .build()?;

    let parts: Vec<&str> = content.split("\n\nAttachment:").collect();
    let (issue_body, attachment_info) = (parts[0], parts.get(1));

    let mut final_body = issue_body.to_string();

    if let Some(attachment_text) = attachment_info {
        if let Some(base64_start) = attachment_text.find("Base64 Data:\n") {
            let base64_data = &attachment_text[base64_start + 12..];
            let image_data = base64::engine::general_purpose::STANDARD.decode(base64_data.trim())?;
            
            let timestamp = chrono::Utc::now().timestamp();
            let filename = format!("screenshot_{}.png", timestamp);
            
            let route = format!("/repos/{}/{}/contents/{}", 
                settings.github.owner, 
                settings.github.repo, 
                filename);

            let encoded_content = base64::engine::general_purpose::STANDARD.encode(&image_data);
            
            let body = serde_json::json!({
                "message": "Add screenshot for bug report",
                "content": encoded_content
            });

            let response = octocrab._put(route, Some(&body)).await?;

            if response.status().is_success() {
                /*let bytes = hyper::body::to_bytes(response.into_body()).await?;
                let file_info: serde_json::Value = serde_json::from_slice(&bytes)?;
                if let Some(content) = file_info.get("content") {
                    if let Some(download_url) = content.get("download_url").and_then(|u| u.as_str()) {
                        final_body.push_str(&format!("\n\n![Screenshot]({})", download_url));
                    }
                }*/
            }
        }
    }

    octocrab
        .issues(&settings.github.owner, &settings.github.repo)
        .create("New Bug Report")
        .body(&final_body)
        .send()
        .await?;

    Ok(())
}
