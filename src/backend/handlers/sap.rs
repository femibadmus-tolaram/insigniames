use actix_web::{HttpResponse, Responder, web};
use reqwest::Client;
use std::env;

pub async fn process_order(search: web::Query<std::collections::HashMap<String, String>>) -> impl Responder {
    let user = env::var("SAP_USERNAME").unwrap_or_default();
    let pass = env::var("SAP_PASSWORD").unwrap_or_default();
    let base_url = env::var("SAP_URL").unwrap_or_default();
    let mut url = format!("{}?$format=json&$top=10", base_url);

    let mut filters = Vec::new();
    filters.push("Plant eq 'A710'".to_string());
    if let Some(order) = search.get("order") { filters.push(format!("ManufacturingOrder eq '{}'", order)); }
    if let Some(machine) = search.get("machine") { filters.push(format!("WorkCenter eq '{}'", machine)); }
    if !filters.is_empty() { url.push_str(&format!("&$filter={}", filters.join(" and "))); }

    // url.push_str("&$expand=to_ProductionOrderComponent,to_ProductionOrderOperation");

    let client = Client::new();
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("sap-client", reqwest::header::HeaderValue::from_static("200"));

    let res = client
        .get(&url)
        .headers(headers)
        .basic_auth(user, Some(pass))
        .send()
        .await;

    match res {
        Ok(r) => {
            let text = r.text().await.unwrap_or_default();
            if text.is_empty() {
                HttpResponse::NotFound().body("No data found")
            } else {
                match serde_json::from_str::<serde_json::Value>(&text) {
                    // Ok(json) => HttpResponse::Ok().json(format_order_data(&json)),
                    Ok(json) => HttpResponse::Ok().json(&json),
                    Err(_) => HttpResponse::BadGateway().body(text),
                }
            }
        }
        Err(e) => HttpResponse::BadGateway().body(e.to_string()),
    }
}

fn format_order_data(json: &serde_json::Value) -> serde_json::Value {
    let orders = json["d"]["results"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|order| {
            let components = order["to_ProcessOrderComponent"]["results"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|c| serde_json::json!({
                    "Material": c["Material"],
                    "RequiredQuantity": c["RequiredQuantity"],
                    "StorageLocation": c["StorageLocation"]
                }))
                .collect::<Vec<_>>();

            let operations = order["to_ProcessOrderOperation"]["results"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|o| serde_json::json!({
                    "Operation": o["ManufacturingOrderOperation"],
                    "Description": o["MfgOrderOperationText"],
                    "WorkCenter": o["WorkCenter"]
                }))
                .collect::<Vec<_>>();

            serde_json::json!({
                "ManufacturingOrder": order["ManufacturingOrder"],
                "ManufacturingOrderType": order["ManufacturingOrderType"],
                "ManufacturingOrderCategory": order["ManufacturingOrderCategory"],
                "Plant": order["Plant"],
                "TotalQuantity": order["TotalQuantity"],
                "ProductionUnit": order["ProductionUnit"],
                "ProductionVersion": order["ProductionVersion"],
                "OrderIsReleased": order["OrderIsReleased"],
                "OrderIsPreCosted": order["OrderIsPreCosted"],
                "SettlementRuleIsCreated": order["SettlementRuleIsCreated"],
                "OrderInternalBillOfOperations": order["OrderInternalBillOfOperations"],
                "Dates/Times": {
                    "MfgOrderCreationDate": order["MfgOrderCreationDate"],
                    "MfgOrderPlannedStartDate": order["MfgOrderPlannedStartDate"],
                    "MfgOrderPlannedEndDate": order["MfgOrderPlannedEndDate"],
                    "MfgOrderScheduledStartDate": order["MfgOrderScheduledStartDate"],
                    "MfgOrderScheduledEndDate": order["MfgOrderScheduledEndDate"]
                },
                "Operations": operations,
                "Unit": order["ProductionUnit"],
                "OperationControlProfile": operations.iter()
                    .map(|o| o["Operation"].as_str().unwrap_or(""))
                    .collect::<Vec<_>>()
                    .join(" / "),
                "Components": components,
                "ComponentCostRelevant": "X",
                "BOMItemCategory": "L",
                "BillOfMaterialCategory": "M",
                "Misc": {
                    "CostingVariants": {
                        "Actual": order["ActualCostsCostingVariant"],
                        "Planned": order["PlannedCostsCostingVariant"]
                    },
                    "ProfitCenter": order["ProfitCenter"],
                    "MRPArea": order["MRPArea"]
                }
            })
        })
        .collect::<Vec<_>>();

    serde_json::json!(orders)
}

pub async fn static_data() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!([
        { "po": "PO001", "sku": "SKU-BF-280G", "materials": "20mic 1840mm Bopp", "planned_films": "Yellow, Black, Magenta, MEK, EA" },
        { "po": "PO002", "sku": "SKU-CH-500G", "materials": "25mic 1620mm Bopp", "planned_films": "Cyan, Black, Yellow" },
        { "po": "PO003", "sku": "SKU-TM-120G", "materials": "18mic 1500mm Bopp", "planned_films": "Red, Yellow, Black" },
        { "po": "PO004", "sku": "SKU-PR-900G", "materials": "30mic 2000mm Bopp", "planned_films": "Magenta, Cyan" },
        { "po": "PO005", "sku": "SKU-KD-70G", "materials": "22mic 1400mm Bopp", "planned_films": "Black, Yellow" },
        { "po": "PO006", "sku": "SKU-LX-300G", "materials": "20mic 1300mm Bopp", "planned_films": "Magenta, Yellow" },
        { "po": "PO007", "sku": "SKU-WF-1KG", "materials": "28mic 2100mm Bopp", "planned_films": "Black, Cyan" },
        { "po": "PO008", "sku": "SKU-SN-45G", "materials": "24mic 1550mm Bopp", "planned_films": "Yellow, Cyan" },
        { "po": "PO009", "sku": "SKU-CR-800G", "materials": "26mic 1700mm Bopp", "planned_films": "Black, Magenta" },
        { "po": "PO010", "sku": "SKU-MF-250G", "materials": "19mic 1480mm Bopp", "planned_films": "Yellow, Red, Black" }
    ]))
}

