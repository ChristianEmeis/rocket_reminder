// Example code that deserializes and serializes the model.
// extern crate serde;
// #[macro_use]
// extern crate serde_derive;
// extern crate serde_json;
//
// use generated_module::[object Object];
//
// fn main() {
//     let json = r#"{"answer": 42}"#;
//     let model: [object Object] = serde_json::from_str(&json).unwrap();
// }

use serde::{Serialize, Deserialize};

extern crate serde_derive;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Events {
    pub count: Option<i64>,
    pub next: Option<String>,
    pub previous: Option<serde_json::Value>,
    pub results: Option<Vec<Result>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Result {
    pub id: Option<String>,
    pub url: Option<String>,
    pub slug: Option<String>,
    pub name: Option<String>,
    pub status: Option<Status>,
    pub last_updated: Option<String>,
    pub net: Option<String>,
    pub window_end: Option<String>,
    pub window_start: Option<String>,
    pub probability: Option<i64>,
    pub holdreason: Option<String>,
    pub failreason: Option<String>,
    pub hashtag: Option<serde_json::Value>,
    pub launch_service_provider: Option<LaunchServiceProvider>,
    pub rocket: Option<Rocket>,
    pub mission: Option<Mission>,
    pub pad: Option<Pad>,
    pub webcast_live: Option<bool>,
    pub image: Option<String>,
    pub infographic: Option<serde_json::Value>,
    pub program: Option<Vec<Program>>,
    pub orbital_launch_attempt_count: Option<i64>,
    pub location_launch_attempt_count: Option<i64>,
    pub pad_launch_attempt_count: Option<i64>,
    pub agency_launch_attempt_count: Option<i64>,
    pub orbital_launch_attempt_count_year: Option<i64>,
    pub location_launch_attempt_count_year: Option<i64>,
    pub pad_launch_attempt_count_year: Option<i64>,
    pub agency_launch_attempt_count_year: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LaunchServiceProvider {
    pub id: Option<i64>,
    pub url: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub launch_service_provider_type: Option<Type>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mission {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub launch_designator: Option<serde_json::Value>,
    #[serde(rename = "type")]
    pub mission_type: Option<String>,
    pub orbit: Option<Status>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Status {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub abbrev: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pad {
    pub id: Option<i64>,
    pub url: Option<String>,
    pub agency_id: Option<i64>,
    pub name: Option<String>,
    pub info_url: Option<String>,
    pub wiki_url: Option<String>,
    pub map_url: Option<String>,
    pub latitude: Option<String>,
    pub longitude: Option<String>,
    pub location: Option<Location>,
    pub map_image: Option<String>,
    pub total_launch_count: Option<i64>,
    pub orbital_launch_attempt_count: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Location {
    pub id: Option<i64>,
    pub url: Option<String>,
    pub name: Option<String>,
    pub country_code: Option<String>,
    pub map_image: Option<String>,
    pub total_launch_count: Option<i64>,
    pub total_landing_count: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Program {
    pub id: Option<i64>,
    pub url: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub agencies: Option<Vec<LaunchServiceProvider>>,
    pub image_url: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<serde_json::Value>,
    pub info_url: Option<String>,
    pub wiki_url: Option<String>,
    pub mission_patches: Option<Vec<Option<serde_json::Value>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rocket {
    pub id: Option<i64>,
    pub configuration: Option<Configuration>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Configuration {
    pub id: Option<i64>,
    pub url: Option<String>,
    pub name: Option<String>,
    pub family: Option<String>,
    pub full_name: Option<String>,
    pub variant: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Type {
    Commercial,
    Government,
    Multinational,
}
