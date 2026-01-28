use actix_web::{HttpResponse, Responder, web};
use reqwest::Client;
use std::{env, error};

pub async fn get_batch_availability(
    material: &str,
    storage_location: &str,
) -> Result<Vec<(String, String)>, Box<dyn error::Error>> {
    let base_url = env::var("SAP_QA_MAV_BASE_URL")?;
    let api_key = env::var("SAP_QA_MAV_APIKey")?;

    let url = format!(
        "{}?$format=json&$filter=Material+eq+'{}'and+StorageLocation+eq+'{}'+and+Plant+eq+'A710'",
        base_url, material, storage_location
    );

    let client = Client::new();
    let res = client
        .get(&url)
        .header("APIKey", api_key)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let mut batches = Vec::new();

    if let Some(results) = res["d"]["results"].as_array() {
        for item in results {
            // let valuation_type = item["ValuationType"].as_str().unwrap_or("");

            // if valuation_type == "" {
            if let (Some(batch), Some(total_stock), Some(unit)) = (
                item["Batch"].as_str(),
                item["TotalStock"].as_str(),
                item["UnitOfMeasure"].as_str(),
            ) {
                let stock_value: f64 = total_stock.trim().parse().unwrap_or(0.0);
                if stock_value > 0.0 {
                    let start_weight = format!("{}{}", total_stock.trim(), unit);
                    batches.push((batch.to_string(), start_weight));
                }
            }
            // }
        }
    }

    Ok(batches)
}

pub async fn get_batch_availability_handler(
    web::Query(params): web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let material_number = match params.get("material_number") {
        Some(num) => num,
        None => return HttpResponse::BadRequest().body("Missing material_number parameter"),
    };
    let storage_location = match params.get("storage_location") {
        Some(num) => num,
        None => return HttpResponse::BadRequest().body("Missing storage_location parameter"),
    };

    match get_batch_availability(material_number, storage_location).await {
        Ok(batches) => HttpResponse::Ok().json(batches),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
