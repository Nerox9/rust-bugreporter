use crate::config::SETTINGS;
use base64::Engine;
use octocrab::Octocrab;
use chrono;

pub async fn create_github_issue(content: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let settings = SETTINGS.get().unwrap();
    
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
                let bytes = hyper::body::to_bytes(response.into_body()).await?;
                let file_info: serde_json::Value = serde_json::from_slice(&bytes)?;
                if let Some(content) = file_info.get("content") {
                    if let Some(download_url) = content.get("download_url").and_then(|u| u.as_str()) {
                        final_body.push_str(&format!("\n\n![Screenshot]({})", download_url));
                    }
                }
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
