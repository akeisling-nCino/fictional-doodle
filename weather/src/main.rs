use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
struct GeoResult {
    lat: String,
    lon: String,
    display_name: String,
}

#[derive(Deserialize)]
struct CurrentWeather {
    temperature: f64,
    windspeed: f64,
    weathercode: u32,
}

#[derive(Deserialize)]
struct WeatherResponse {
    current_weather: CurrentWeather,
}

fn weather_description(code: u32) -> &'static str {
    match code {
        0 => "Clear sky",
        1 => "Mainly clear",
        2 => "Partly cloudy",
        3 => "Overcast",
        45 | 48 => "Foggy",
        51 | 53 | 55 => "Drizzle",
        61 | 63 | 65 => "Rain",
        71 | 73 | 75 => "Snow",
        80 | 81 | 82 => "Rain showers",
        95 => "Thunderstorm",
        96 | 99 => "Thunderstorm with hail",
        _ => "Unknown",
    }
}

fn celsius_to_fahrenheit(c: f64) -> f64 {
    c * 9.0 / 5.0 + 32.0
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: weather <city or zipcode>");
        std::process::exit(1);
    }

    let raw = args[1..].join(" ");
    // Disambiguate bare US zip codes (5 digits) by appending country
    let query = if raw.chars().all(|c| c.is_ascii_digit()) && raw.len() == 5 {
        format!("{raw}, USA")
    } else {
        raw
    };

    let client = reqwest::blocking::Client::builder()
        .user_agent("weather-cli/1.0")
        .build()
        .expect("Failed to build HTTP client");

    // Geocode the query using Nominatim (OpenStreetMap)
    let geo_url = format!(
        "https://nominatim.openstreetmap.org/search?q={}&format=json&limit=1",
        urlencoded(&query)
    );

    let geo_results: Vec<GeoResult> = client
        .get(&geo_url)
        .send()
        .expect("Failed to contact geocoding API")
        .json()
        .expect("Failed to parse geocoding response");

    let location = geo_results
        .into_iter()
        .next()
        .unwrap_or_else(|| {
            eprintln!("Could not find location: {query}");
            std::process::exit(1);
        });

    let lat: f64 = location.lat.parse::<f64>().expect("Invalid latitude");
    let lon: f64 = location.lon.parse::<f64>().expect("Invalid longitude");

    // Fetch weather from Open-Meteo (no API key required)
    let weather_url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}&current_weather=true&temperature_unit=celsius"
    );

    let weather: WeatherResponse = client
        .get(&weather_url)
        .send()
        .expect("Failed to contact weather API")
        .json()
        .expect("Failed to parse weather response");

    let cw = weather.current_weather;
    let temp_c = cw.temperature;
    let temp_f = celsius_to_fahrenheit(temp_c);

    println!("Location:    {}", location.display_name);
    println!("Condition:   {}", weather_description(cw.weathercode));
    println!("Temperature: {temp_f:.1}°F / {temp_c:.1}°C");
    println!("Wind speed:  {:.1} km/h", cw.windspeed);
}

fn urlencoded(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "+".to_string(),
            c if c.is_alphanumeric() || "-_.~".contains(c) => c.to_string(),
            c => format!("%{:02X}", c as u32),
        })
        .collect()
}
