//! Current weather from Open-Meteo (https://open-meteo.com), no API key.
//! Opt-in: the frontend only calls this when Weather is enabled, since it
//! sends the configured city name over the network.

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const CACHE_FOR: Duration = Duration::from_secs(15 * 60);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Weather {
    city: String,
    /// In the requested unit.
    temperature: f64,
    high: Option<f64>,
    low: Option<f64>,
    /// WMO weather interpretation code.
    code: u32,
    is_day: bool,
}

#[derive(Deserialize)]
struct GeoResponse {
    results: Option<Vec<GeoResult>>,
}

#[derive(Deserialize)]
struct GeoResult {
    name: String,
    latitude: f64,
    longitude: f64,
}

#[derive(Deserialize)]
struct Forecast {
    current: Current,
    daily: Option<Daily>,
}

#[derive(Deserialize)]
struct Current {
    temperature_2m: f64,
    weather_code: u32,
    is_day: u8,
}

#[derive(Deserialize)]
struct Daily {
    temperature_2m_max: Vec<f64>,
    temperature_2m_min: Vec<f64>,
}

static CACHE: Mutex<Option<(String, Instant, Weather)>> = Mutex::new(None);

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(8)))
        .user_agent("FLOAT (https://github.com/Sahaj1207/FLOAT)")
        .build()
        .into()
}

#[derive(Deserialize)]
struct IpApiLocation {
    city: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
}

#[derive(Deserialize)]
struct FreeIpLocation {
    #[serde(rename = "cityName")]
    city_name: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

fn auto_locate(agent: &ureq::Agent) -> Result<(String, f64, f64), String> {
    // 1. Try ip-api.com
    if let Ok(mut resp) = agent.get("http://ip-api.com/json").call() {
        if let Ok(loc) = resp.body_mut().read_json::<IpApiLocation>() {
            if let (Some(c), Some(lat), Some(lon)) = (loc.city, loc.lat, loc.lon) {
                if !c.is_empty() {
                    return Ok((c, lat, lon));
                }
            }
        }
    }

    // 2. Fallback to freeipapi.com
    if let Ok(mut resp) = agent.get("https://freeipapi.com/api/json").call() {
        if let Ok(loc) = resp.body_mut().read_json::<FreeIpLocation>() {
            if let (Some(c), Some(lat), Some(lon)) = (loc.city_name, loc.latitude, loc.longitude) {
                if !c.is_empty() {
                    return Ok((c, lat, lon));
                }
            }
        }
    }

    Err("Could not auto-detect location".into())
}

fn fetch(city: &str, fahrenheit: bool) -> Result<Weather, String> {
    let agent = agent();
    let is_auto = city.trim().is_empty() || city.trim().eq_ignore_ascii_case("auto");

    let (place_name, lat, lon) = if is_auto {
        auto_locate(&agent)?
    } else {
        let geo: GeoResponse = agent
            .get("https://geocoding-api.open-meteo.com/v1/search")
            .query("name", city)
            .query("count", "1")
            .call()
            .map_err(|e| e.to_string())?
            .body_mut()
            .read_json()
            .map_err(|e| e.to_string())?;
        let place = geo
            .results
            .and_then(|r| r.into_iter().next())
            .ok_or_else(|| format!("Couldn't find \"{city}\""))?;
        (place.name, place.latitude, place.longitude)
    };

    let forecast: Forecast = agent
        .get("https://api.open-meteo.com/v1/forecast")
        .query("latitude", lat.to_string())
        .query("longitude", lon.to_string())
        .query("current", "temperature_2m,weather_code,is_day")
        .query("daily", "temperature_2m_max,temperature_2m_min")
        .query("forecast_days", "1")
        .query("timezone", "auto")
        .query("temperature_unit", if fahrenheit { "fahrenheit" } else { "celsius" })
        .call()
        .map_err(|e| e.to_string())?
        .body_mut()
        .read_json()
        .map_err(|e| e.to_string())?;

    Ok(Weather {
        city: place_name,
        temperature: forecast.current.temperature_2m,
        high: forecast.daily.as_ref().and_then(|d| d.temperature_2m_max.first().copied()),
        low: forecast.daily.as_ref().and_then(|d| d.temperature_2m_min.first().copied()),
        code: forecast.current.weather_code,
        is_day: forecast.current.is_day == 1,
    })
}

/// Current conditions for a city, cached for 15 minutes.
#[tauri::command]
pub async fn get_weather(city: String, fahrenheit: bool) -> Result<Weather, String> {
    let city = city.trim().to_string();
    let query_city = if city.is_empty() { "auto".to_string() } else { city };
    let key = format!("{}|{}", query_city.to_lowercase(), fahrenheit);
    if let Some((k, at, weather)) = CACHE.lock().unwrap().as_ref() {
        if *k == key && at.elapsed() < CACHE_FOR {
            return Ok(weather.clone());
        }
    }
    let weather = tauri::async_runtime::spawn_blocking(move || fetch(&query_city, fahrenheit))
        .await
        .map_err(|e| e.to_string())??;
    *CACHE.lock().unwrap() = Some((key, Instant::now(), weather.clone()));
    Ok(weather)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "hits open-meteo.com"]
    fn fetches_weather() {
        let w = fetch("London", false).expect("weather");
        println!("{}: {:.1} (hi {:?} lo {:?}) code {} day {}", w.city, w.temperature, w.high, w.low, w.code, w.is_day);
    }
}
