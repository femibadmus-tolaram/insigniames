use chrono::Local;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[allow(non_snake_case)]
#[derive(Serialize)]
struct MaterialDocItem {
    Material: String,
    GoodsMovementType: String,
    Plant: String,
    StorageLocation: String,
    QuantityInEntryUnit: String,
    EntryUnit: String,
    ManufacturingOrder: String,
    Batch: String,
}

#[allow(non_snake_case)]
#[derive(Serialize)]
struct MaterialDocRequest {
    PostingDate: String,
    DocumentDate: String,
    GoodsMovementCode: String,
    to_MaterialDocumentItem: Vec<MaterialDocItem>,
}

#[derive(Deserialize)]
struct MaterialDocResponse {
    d: MaterialDocResponseData,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct MaterialDocResponseData {
    MaterialDocument: String,
}

pub async fn post_material_document(
    material: &str,
    batch: &str,
    order: &str,
    quantity: &str,
    unit: &str,
    posting_date: &str,
    storage_location: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let url = env::var("SAP_QA_MATERIAL_INSUANCE_URL")?;
    let api_key = env::var("SAP_QA_MATERIAL_INSUANCE_APIKEY")?;

    let now = Local::now().format("%Y-%m-%d").to_string();
    let date_str = format!("{}T00:00:00", now);

    let request = MaterialDocRequest {
        PostingDate: posting_date.to_string(),
        DocumentDate: date_str,
        GoodsMovementCode: "05".to_string(),
        to_MaterialDocumentItem: vec![MaterialDocItem {
            Material: material.to_string(),
            GoodsMovementType: "261".to_string(),
            Plant: "A710".to_string(),
            StorageLocation: storage_location.to_string(),
            QuantityInEntryUnit: quantity.to_string(),
            EntryUnit: unit.to_string(),
            ManufacturingOrder: order.to_string(),
            Batch: batch.to_string(),
        }],
    };

    let client = Client::new();
    let res = client
        .post(&url)
        .header("APIKey", api_key)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&request)
        .send()
        .await?;

    let status = res.status();
    let response_text = res.text().await?;
    // println!("POSTING DATE: {}", posting_date.to_string());

    if status.is_success() {
        let response: MaterialDocResponse = serde_json::from_str(&response_text)?;
        Ok(response.d.MaterialDocument)
    } else {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response_text) {
            let msg = json["error"]["message"]["value"]
                .as_str()
                .unwrap_or(&response_text);
            Err(msg.into())
        } else {
            Err(format!("SAP Error {}: {}", status, response_text).into())
        }
    }
}

#[tokio::test]
async fn test_post_material_document() {
    use dotenv::dotenv;

    dotenv().ok();

    let result = post_material_document(
        "30000950",
        "J23-612",
        "220012061",
        "9613.7",
        "KG",
        "2026-01-09T00:00:00",
        "DW01",
    )
    .await;

    match result {
        Ok(doc) => {
            println!("Document Number: {}", doc);
            assert!(!doc.is_empty(), "Document number should not be empty");
        }
        Err(e) => {
            println!("Error: {}", e);
            panic!("Test failed: {}", e);
        }
    }
}
