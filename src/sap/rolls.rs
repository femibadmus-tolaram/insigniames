use reqwest::Client;
use std::env;

pub struct RollData {
    pub weight: String,
    pub meter: String,
    pub batch: String,
    pub production_order: String,
}

pub async fn post_rolls(data: RollData) -> Result<(), String> {
    let url = env::var("SAP_QA_ROLL_URL").unwrap_or_default();
    let api_key = env::var("SAP_QA_ROLL_APIKEY").unwrap_or_default();

    if url.is_empty() || api_key.is_empty() {
        return Err("Missing SAP URL or API key".to_string());
    }

    let json_data = serde_json::json!({
        "AlternateUOM": "KG",
        "AlternateQuantity": data.weight,
        "UOM": "M",
        "Quantity": data.meter,
        "Batch": data.batch,
        "ProductionOrder": data.production_order
    });

    let client = Client::new();
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("APIKey", api_key.parse().unwrap());
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Accept", "application/json".parse().unwrap());

    match client
        .post(&url)
        .headers(headers)
        .json(&json_data)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            let statustxt = response.text().await.unwrap_or_default();

            if status.as_u16() == 201 {
                Ok(())
            } else {
                let error_msg =
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&statustxt) {
                        json["error"]["message"]["value"]
                            .as_str()
                            .unwrap_or(&statustxt)
                            .to_string()
                    } else {
                        format!("HTTP {}: {}", status, statustxt)
                    };
                Err(error_msg)
            }
        }
        Err(e) => Err(format!("Request failed: {}", e)),
    }
}
