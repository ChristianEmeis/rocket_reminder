use std::{fs::OpenOptions, num::ParseIntError, io::Read, collections::HashMap};

use chrono::{NaiveDate, NaiveDateTime};
use nanoid::nanoid;
use serde::{Serialize, Deserialize};

use crate::{SentNotifications, LaunchNotifications};
pub fn get_random_id() -> String {
    nanoid!()
}

#[derive(Serialize, Deserialize, Debug)]
struct DownloadedImages {
    map: HashMap<String, String>,
}


pub fn add_downloaded_image_to_json(path: String, id: String) {
    let mut readable_file = match OpenOptions::new().read(true).open("images.json") {
        Ok(c) => c,
        Err(_) => OpenOptions::new().create(true).write(true).read(true).open("images.json").unwrap()
    };
    let mut content = String::new();
    readable_file.read_to_string(&mut content)
        .expect("Could not read file 'notifications.json'");
    let mut parsed_json: DownloadedImages = match serde_json::from_str(&content){
        Ok(c) => c,
        Err(_) => DownloadedImages { map: HashMap::new() }
    };
    let writeable_file = OpenOptions::new().write(true).truncate(true).open("images.json").expect("Could not open writeable images.json");
    parsed_json.map.insert(id, path);
    serde_json::ser::to_writer(writeable_file, &parsed_json).expect("Could not serialize images.json");
}

pub fn remove_downloaded_image_from_json(id: String) {
    let mut readable_file = match OpenOptions::new().read(true).open("images.json")  {
        Ok(c) => c,
        Err(_) => OpenOptions::new().create(true).write(true).read(true).open("images.json").unwrap()
    };
    let mut content = String::new();
    readable_file.read_to_string(&mut content)
        .expect("Could not read file 'notifications.json'");
        let mut parsed_json: DownloadedImages = match serde_json::from_str(&content){
            Ok(c) => c,
            Err(_) => DownloadedImages { map: HashMap::new() }
        };
        let writeable_file = OpenOptions::new().write(true).truncate(true).open("images.json").expect("Could not open writeable images.json");
        parsed_json.map.remove(&id);
        serde_json::ser::to_writer(writeable_file, &parsed_json).expect("Could not serialize images.json");
}

pub fn get_send_notifications() -> SentNotifications{
    let mut readable_file = match OpenOptions::new().read(true).open("notifications.json")  {
        Ok(c) => c,
        Err(_) => OpenOptions::new().create(true).write(true).read(true).open("notifications.json").unwrap()
    };
        let mut content = String::new();
        readable_file.read_to_string(&mut content)
            .expect("Could not read file 'notifications.json'");
        match serde_json::from_str(&content) {
            Ok(c) => c,
            Err(_) => SentNotifications { map: HashMap::new() }
        }
}

pub fn write_send_notifications(input: LaunchNotifications, id: String) {
    let mut readable_file = match OpenOptions::new().read(true).open("notifications.json")  {
        Ok(c) => c,
        Err(_) => OpenOptions::new().create(true).write(true).read(true).open("notifications.json").unwrap()
    };
    let mut content = String::new();
    readable_file.read_to_string(&mut content)
        .expect("Could not read file 'notifications.json'");
    let mut parsed_json: SentNotifications = match serde_json::from_str(&content){
        Ok(c) => c,
        Err(_) => SentNotifications { map: HashMap::new() }
    };

    let writeable_empty_file = OpenOptions::new().write(true).truncate(true).open("notifications.json").expect("error opening writeable file notifications.json");
    parsed_json.map.insert(id, input);
    serde_json::ser::to_writer(writeable_empty_file, &parsed_json).expect("Could not serialize images.json");
} 

pub async fn remove_unwanted_images(){
    let mut readable_file = match OpenOptions::new().read(true).open("images.json") {
        Ok(c) => c,
        Err(_) => OpenOptions::new().create(true).write(true).read(true).open("images.json").unwrap()
    };
    let mut content = String::new();
    readable_file.read_to_string(&mut content)
        .expect("Could not read file 'notifications.json'");
    let mut parsed_json: DownloadedImages = match serde_json::from_str(&content){
        Ok(c) => c,
        Err(_) => DownloadedImages { map: HashMap::new() }
    };
    let iter = parsed_json.map.drain();
    for i in iter {
        match tokio::fs::remove_file(i.1).await{
            Ok(_) => println!("Deleted image file"),
            Err(e) => println!("Cannot delete image file because of error: {}", e)
        };
    }
    OpenOptions::new().write(true).truncate(true).open("images.json").expect("Could not open writeable images.json");
}

#[derive(Debug)]
pub enum ParseTimeError {
    ParseIntError(ParseIntError),
    InvalidSplit(&'static str, u32),
}
impl From<ParseIntError> for ParseTimeError {
    fn from(e: ParseIntError) -> Self {
        Self::ParseIntError(e)
    }
}
pub fn parse_time(time: &str) -> Result<NaiveDateTime, ParseTimeError> {
    let parts = time.split('T').collect::<Vec<_>>();
    if parts.len() != 2 {
        return Err(ParseTimeError::InvalidSplit("Parts", parts.len() as u32));
    }
    let date_split = parts[0]
        .split('-')
        .map(|s| s.parse::<u32>())
        .collect::<Result<Vec<_>, _>>()?;
    if date_split.len() != 3 {
        return Err(ParseTimeError::InvalidSplit(
            "Date",
            date_split.len() as u32,
        ));
    }
    let time_split = parts[1][..(parts[1].len() - 1)]
        .split(':')
        .map(|s| s.parse::<u32>())
        .collect::<Result<Vec<_>, _>>()?;
    if time_split.len() != 3 {
        return Err(ParseTimeError::InvalidSplit(
            "Time",
            time_split.len() as u32,
        ));
    }
    Ok(
        NaiveDate::from_ymd(date_split[0] as i32, date_split[1], date_split[2]).and_hms(
            time_split[0],
            time_split[1],
            time_split[2],
        ),
    )
}