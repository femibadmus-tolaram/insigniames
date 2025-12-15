use reqwest::Client;
use std::env;

pub struct RollData {
    pub alternate_quantity: String,
    pub quantity: String,
    pub batch: String,
    pub production_order: String,
}

pub async fn post_rolls(data: RollData) -> bool {
    let url = env::var("SAP_QA_ROLL_URL").unwrap_or_default();
    let api_key = env::var("SAP_QA_ROLL_APIKEY").unwrap_or_default();
    
    if url.is_empty() || api_key.is_empty() {
        eprintln!("Missing SAP URL or API key");
        return false;
    }
    
    let json_data = serde_json::json!({
        "AlternateUOM": "KG",
        "AlternateQuantity": data.alternate_quantity,
        "UOM": "M",
        "Quantity": data.quantity,
        "Batch": data.batch,
        "ProductionOrder": data.production_order
    });

    let client = Client::new();
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("APIKey", api_key.parse().unwrap());
    headers.insert("Content-Type", "application/json".parse().unwrap());

    // println!("Sending to URL: {}", url);
    // println!("JSON Data: {:?}", json_data);

    match client.post(&url).headers(headers).json(&json_data).send().await {
        Ok(response) => {
            let status = response.status();
            // println!("Response Status: {}", status);
            status.as_u16() == 201
        }
        Err(e) => {
            eprintln!("Request error: {:?}", e);
            false
        }
    }
}

