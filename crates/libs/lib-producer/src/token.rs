use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::info;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BitcoinInfo {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub image: String,
    pub current_price: f64,
    pub market_cap: u64,
    pub market_cap_rank: u32,
    pub fully_diluted_valuation: Option<u64>,
    pub total_volume: f64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub price_change_24h: f64,
    pub price_change_percentage_24h: f64,
    pub market_cap_change_24h: f64,
    pub market_cap_change_percentage_24h: f64,
    pub circulating_supply: f64,
    pub total_supply: Option<f64>,
    pub max_supply: Option<f64>,
    pub ath: f64,
    pub ath_change_percentage: f64,
    pub ath_date: String,
    pub atl: f64,
    pub atl_change_percentage: f64,
    pub atl_date: String,
    pub roi: Option<serde_json::Value>,
    pub last_updated: String,
}

pub async fn fetch_bitcoin_info() -> Result<BitcoinInfo> {
    let client = Client::new();
    let api_key = env::var("COINGECKO_API_KEY").expect("COINGECKO_API_KEY must be set");
    let url = format!("https://api.coingecko.com/api/v3/coins/markets?vs_currency=usd&ids=bitcoin&x_cg_demo_api_key={}", api_key);

    let response = client
        .get(url)
        .send()
        .await?
        .json::<Vec<BitcoinInfo>>()
        .await?;

    let response_size = serde_json::to_string(&response)?.len();
    // For debugging
    info!("Response size: {} bytes", response_size);

    response
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No Bitcoin info returned"))
}

pub async fn get_bitcoin_price() -> Result<String> {
    let bitcoin_info = fetch_bitcoin_info().await?;
    Ok(format!("${:.2}", bitcoin_info.current_price))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_bitcoin_info() {
        let result = fetch_bitcoin_info().await;
        assert!(result.is_ok());
        let bitcoin_info = result.unwrap();
        assert_eq!(bitcoin_info.id, "bitcoin");
        assert_eq!(bitcoin_info.symbol, "btc");
        assert_eq!(bitcoin_info.name, "Bitcoin");
    }

    #[tokio::test]
    async fn test_get_bitcoin_price() {
        let result = get_bitcoin_price().await;
        assert!(result.is_ok());
        let price = result.unwrap();
        assert!(price.starts_with('$'));
        assert!(price.parse::<f64>().is_ok());
    }
}
