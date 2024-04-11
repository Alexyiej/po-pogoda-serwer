use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::sync::Mutex;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use validator::Validate;
use actix_cors::Cors;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_data = web::Data::new(AppState {
        weather_data: Mutex::new(HashMap::new()),
    });


    HttpServer::new(move || {
        App::new()
            .wrap(Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header())        
            .app_data(app_data.clone())
            .route("/add", web::post().to(Weather::add))
            .route("/get/{timestamp}", web::get().to(Weather::get))
    })
    .bind("127.0.0.1:5000")?
    .run()
    .await
}


impl Weather{
    async fn add(data: web::Json<WeatherData>, state: web::Data<AppState>) -> impl Responder {
        let mut state_data = state.weather_data.lock().unwrap();
        println!("Timestamp: {}", data.timestamp);
        state_data.insert(data.timestamp, data.into_inner());
        HttpResponse::Ok().json("Data added successfully")
    }
    
    
    async fn get(query: web::Path<i64>, state: web::Data<AppState>) -> impl Responder {
        let state_data = state.weather_data.lock().unwrap();
    
        match get_closest_timestamp(&state_data, *query) {
            Some(closest_key) => {
                state_data.get(&closest_key).map_or_else(
                    || HttpResponse::NotFound().json("No data found"),
                    |data| HttpResponse::Ok().json(data)
                )
            },
            None => HttpResponse::NotFound().json("No data found"),
        }
    }
}


fn get_closest_timestamp(weather_data: &HashMap<i64, WeatherData>, query_timestamp: i64) -> Option<i64> {
    weather_data.keys().fold(None, |acc, &k| match acc {
        None => Some(k),
        Some(closest) => {
            if (k - query_timestamp).abs() < (closest - query_timestamp).abs() {Some(k) } else { acc }
        },
    })
}


#[derive(Debug, Serialize, Deserialize, Validate)]
struct WeatherData {
    pub timestamp: i64,
    #[validate(range(min = -100.0, max = 60.0))]
    pub temperature: f64,
    #[validate(range(min = 900.0, max = 1080.0))]
    pub pressure: f64,
    #[validate(range(min = 0.0))]
    pub wind_speed: f64,
    pub position: Position,

}


#[derive(Debug, Serialize, Deserialize, Validate)]
struct Position{
    pub city: String,
    pub country: String,
    pub state: String,
}


struct AppState {
    weather_data: Mutex<HashMap<i64, WeatherData>>,
}


struct Weather;