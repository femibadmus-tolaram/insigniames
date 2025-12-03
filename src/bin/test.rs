use tiberius::{Client, Config, AuthMethod};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let server = env::var("SCRAP_SERVER")?;
    let database = env::var("SCRAP_DB")?;
    let username = env::var("SCRAP_UID")?;
    let password = env::var("SCRAP_PASSWD")?;

    let mut config = Config::new();
    config.host(&server);
    config.port(1433);
    config.database(&database);
    config.authentication(AuthMethod::sql_server(&username, &password));
    config.trust_cert();
    
    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;
    
    let mut client = Client::connect(config, tcp.compat_write()).await?;

    let query = "
        SELECT 
            CAST(REPLACE(w.shift, 'SHIFT ', '') AS int) AS [Shift],
            w.actualdate + ' ' + w.log_time AS [DateTime],
            w.materials AS [Scrap Type],
            w.weight AS [Weight (kg)],
            CONCAT(u.firstname, ' ', u.surname) AS [Created By]
        FROM weight_log w
        LEFT JOIN Users u ON w.staffid = u.staffid
        ORDER BY w.id
    ";

    let stream = client.query(query, &[]).await?;
    let rows = stream.into_results().await?;
    
    println!("Shift | DateTime | Scrap Type | Weight (kg) | Created By");
    println!("--------------------------------------------------------");
    
    for row in rows {
        for col in row {
            let shift: i32 = col.get("Shift").unwrap_or(0);
            let datetime: Option<&str> = col.get("DateTime");
            let scrap_type: Option<&str> = col.get("Scrap Type");
            let weight: Option<f64> = col.get("Weight (kg)");
            let created_by: Option<&str> = col.get("Created By");
            
            println!("{:5} | {:20} | {:10} | {:10.2} | {}",
                shift,
                datetime.unwrap_or(""),
                scrap_type.unwrap_or(""),
                weight.unwrap_or(0.0),
                created_by.unwrap_or("")
            );
        }
    }

    Ok(())
}