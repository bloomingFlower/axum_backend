use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;

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
    pub percent_change_1h: f64,
    pub percent_change_24h: f64,
    pub percent_change_7d: f64,
    pub percent_change_30d: f64,
}

impl Default for BitcoinInfo {
    fn default() -> Self {
        BitcoinInfo {
            id: "bitcoin".to_string(),
            symbol: "btc".to_string(),
            name: "Bitcoin".to_string(),
            image: "".to_string(),
            current_price: 0.0,
            market_cap: 0,
            market_cap_rank: 0,
            fully_diluted_valuation: None,
            total_volume: 0.0,
            high_24h: 0.0,
            low_24h: 0.0,
            price_change_24h: 0.0,
            price_change_percentage_24h: 0.0,
            market_cap_change_24h: 0.0,
            market_cap_change_percentage_24h: 0.0,
            circulating_supply: 0.0,
            total_supply: None,
            max_supply: None,
            ath: 0.0,
            ath_change_percentage: 0.0,
            ath_date: "".to_string(),
            atl: 0.0,
            atl_change_percentage: 0.0,
            atl_date: "".to_string(),
            roi: None,
            last_updated: "".to_string(),
            percent_change_1h: 0.0,
            percent_change_24h: 0.0,
            percent_change_7d: 0.0,
            percent_change_30d: 0.0,
        }
    }
}

pub async fn fetch_bitcoin_info() -> Result<BitcoinInfo> {
    let client = Client::new();
    let api_key = env::var("COINMARKETCAP_API_KEY").expect("COINMARKETCAP_API_KEY must be set");
    let url = "https://pro-api.coinmarketcap.com/v1/cryptocurrency/quotes/latest";

    let response = client
        .get(url)
        .query(&[("symbol", "BTC"), ("convert", "USD")])
        .header("X-CMC_PRO_API_KEY", api_key)
        .header("Accept", "application/json")
        .send()
        .await?
        .json::<Value>()
        .await?;

    let btc_data = response["data"]["BTC"].clone();
    let quote = &btc_data["quote"]["USD"];

    let current_price = quote["price"].as_f64().unwrap_or(0.0);
    let price_change_percentage_24h = quote["percent_change_24h"].as_f64().unwrap_or(0.0);

    // Calculate 24h high and low prices more accurately
    let price_change_24h = quote["percent_change_24h"].as_f64().unwrap_or(0.0);
    let high_24h = current_price * (1.0 + price_change_24h.abs() / 100.0);
    let low_24h = current_price / (1.0 + price_change_24h.abs() / 100.0);

    // Ensure high_24h is always greater than or equal to low_24h
    let (high_24h, low_24h) = if price_change_24h >= 0.0 {
        (high_24h, current_price)
    } else {
        (current_price, low_24h)
    };

    let bitcoin_info = BitcoinInfo {
        id: "bitcoin".to_string(),
        symbol: "btc".to_string(),
        name: btc_data["name"].as_str().unwrap_or("Bitcoin").to_string(),
        image: "".to_string(),
        current_price,
        market_cap: quote["market_cap"].as_u64().unwrap_or(0),
        market_cap_rank: btc_data["cmc_rank"].as_u64().unwrap_or(0) as u32,
        fully_diluted_valuation: Some(quote["fully_diluted_market_cap"].as_u64().unwrap_or(0)),
        total_volume: quote["volume_24h"].as_f64().unwrap_or(0.0),
        high_24h,
        low_24h,
        price_change_24h,
        price_change_percentage_24h,
        market_cap_change_24h: 0.0, // CoinMarketCap API doesn't provide this directly
        market_cap_change_percentage_24h: quote["market_cap_change_percent_24h"]
            .as_f64()
            .unwrap_or(0.0),
        circulating_supply: btc_data["circulating_supply"].as_f64().unwrap_or(0.0),
        total_supply: Some(btc_data["total_supply"].as_f64().unwrap_or(0.0)),
        max_supply: Some(btc_data["max_supply"].as_f64().unwrap_or(0.0)),
        ath: 0.0,                   // CoinMarketCap API doesn't provide this
        ath_change_percentage: 0.0, // CoinMarketCap API doesn't provide this
        ath_date: "".to_string(),   // CoinMarketCap API doesn't provide this
        atl: 0.0,                   // CoinMarketCap API doesn't provide this
        atl_change_percentage: 0.0, // CoinMarketCap API doesn't provide this
        atl_date: "".to_string(),   // CoinMarketCap API doesn't provide this
        roi: None,                  // CoinMarketCap API doesn't provide this
        last_updated: btc_data["last_updated"].as_str().unwrap_or("").to_string(),
        percent_change_1h: quote["percent_change_1h"].as_f64().unwrap_or(0.0),
        percent_change_24h: quote["percent_change_24h"].as_f64().unwrap_or(0.0),
        percent_change_7d: quote["percent_change_7d"].as_f64().unwrap_or(0.0),
        percent_change_30d: quote["percent_change_30d"].as_f64().unwrap_or(0.0),
    };

    Ok(bitcoin_info)
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
