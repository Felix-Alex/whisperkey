use crate::error::{AppError, AppResult};
use crate::license::verifier::License;

/// Activate a license code with the remote server
pub async fn activate(code: &str, fingerprint: &str) -> AppResult<License> {
    let url = "https://api.whisperkey.app/v1/activate";
    let body = serde_json::json!({
        "code": code,
        "fingerprint": fingerprint,
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .json(&body)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|_| AppError::Network)?;

    let status = resp.status();
    if status == 401 || status == 403 || status == 404 {
        return Err(AppError::LicenseInvalid);
    }
    if !status.is_success() {
        return Err(AppError::Network);
    }

    let license: License = resp.json().await.map_err(|_| AppError::Internal)?;
    Ok(license)
}
