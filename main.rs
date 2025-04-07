use actix_web::{get, web, App, HttpServer, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::Value;
use tera::{Tera, Context};

#[derive(Deserialize)]
struct Query {
    symbol: String,
}

#[get("/news")]
async fn news_page(query: web::Query<Query>, tmpl: web::Data<Tera>) -> impl Responder {
    let symbol = query.symbol.to_lowercase();

    match symbol_to_coinpaprika_id(&symbol) {
        Some(paprika_id) => {
            let coin_info = fetch_coinpaprika_info(paprika_id).await.unwrap_or(Value::Null);
            let gnews_data = fetch_gnews_articles(&symbol).await.unwrap_or(Value::Null);

            let mut ctx = Context::new();
            ctx.insert("coin", &coin_info);
            ctx.insert("articles", &gnews_data["articles"]);

            let rendered = tmpl.render("news.html.tera", &ctx)
                .unwrap_or_else(|e| format!("Ошибка шаблона: {}", e));

            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(rendered)
        }
        None => HttpResponse::NotFound()
            .content_type("text/plain; charset=utf-8")
            .body("Неизвестный символ криптовалюты"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let tera = Tera::new("templates/**/*").expect("Не удалось загрузить шаблоны");

    println!("🚀 Сервер запущен на http://localhost:8080/news?symbol=btc");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .service(news_page)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

fn symbol_to_coinpaprika_id(symbol: &str) -> Option<&'static str> {
    match symbol.to_lowercase().as_str() {
        "btc" => Some("btc-bitcoin"),
        "eth" => Some("eth-ethereum"),
        "doge" => Some("doge-dogecoin"),
        "xrp" => Some("xrp-xrp"),
        "ada" => Some("ada-cardano"),
        "ltc" => Some("ltc-litecoin"),
        "sol" => Some("sol-solana"),
        _ => None,
    }
}

async fn fetch_coinpaprika_info(coin_id: &str) -> Result<Value, reqwest::Error> {
    let url = format!("https://api.coinpaprika.com/v1/coins/{}", coin_id);
    let response = reqwest::get(&url).await?.json::<Value>().await?;
    Ok(response)
}

async fn fetch_gnews_articles(keyword: &str) -> Result<Value, reqwest::Error> {
    let api_key = "9cee94ecdc9fee6989d139de482e6435";
    let url = format!(
        "https://gnews.io/api/v4/search?q={}&lang=en&max=10&apikey={}",
        keyword, api_key
    );
    let response = reqwest::get(&url).await?.json::<Value>().await?;
    Ok(response)
}
