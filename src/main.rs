#![feature(async_closure)]
mod upcoming_launch;
use chrono::DateTime;
use chrono::Local;
use chrono::TimeZone;
use serde::{Serialize, Deserialize};
use serde_json::Map;
use chrono::Utc;
use util::add_downloaded_image_to_json;
use util::get_random_id;
use util::parse_time;
use util::remove_downloaded_image_from_json;
use util::remove_unwanted_images;
use util::write_send_notifications;
use util::get_send_notifications;
use std::{
    borrow::Cow, collections::HashMap, path::PathBuf,
    time::Duration,
};

mod util;

#[tokio::main]
async fn main() {
    println!("TEST");
    remove_unwanted_images().await;
    tokio::spawn(check());
    println!("AFTER RUNNING THREAD");
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}

async fn check(){
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        println!("RUN CHECK");
        check_for_notification().await;
    }
}


use winrt_notification::{
    Action,
    Toast,
    ToastWithHandlers,
};




async fn download_image(url: &str, id: String) -> Result<PathBuf, Box<dyn std::error::Error>> {
    use tokio::io::AsyncWriteExt;
    let format_from_url = || -> Option<&'static str> {
        let format = url.split('.').last()?;
        match format {
            "png" => Some("png"),
            "jpeg" | "jpg" | "JPEG" | "JPG" => Some("jpg"),
            _ => None
        }
    };
    let resp = reqwest::get(url).await?;
    let extension = match resp.headers().get("content-type").map(|val| val.to_str()).transpose()? {
        Some("image/jpeg" | "image/jpg") => "jpg",
        Some("image/png") => "png",
        _ => format_from_url().ok_or("failed to find out format")?
    };
    let test = resp.bytes().await?;
    let path = PathBuf::from(format!("{id}.{extension}"));
    let mut f = tokio::fs::File::create(&path).await?;
    f.write_all(test.to_vec().as_slice()).await?;
    Ok(path)
}


async fn send_notification(notification: NotificationData) {
    let image_id = get_random_id();
    let path_to_image = download_image(&notification.image_path, image_id.clone()).await.expect("Failed to download image");
    let path_to_image = dunce::canonicalize(path_to_image).unwrap();
    add_downloaded_image_to_json(path_to_image.to_str().unwrap().to_string(), image_id.clone());

    let url = notification.url;

    let barrier = std::sync::Arc::new(tokio::sync::Semaphore::new(0));
    let barrier_ref1 = barrier.clone();
    let barrier_ref2 = barrier.clone();
    let barrier_ref3 = barrier.clone();
    let toast = Toast::new(Toast::POWERSHELL_APP_ID)
    .title(notification.name.as_str())
    .text1(notification.notification_text.as_str())
    .text2(notification.time_string.as_str())
    .image(&path_to_image, "Rocket Launching")
    .action(Action::from_content("More"));
    ToastWithHandlers::new(toast)
        .on_activate(move |_args| {
            open_browser(&url);
            // exit the waiting loop
            barrier_ref1.add_permits(1);
            Ok(())
        })
        .on_dismiss(move |_, _| {
            barrier_ref2.add_permits(1);
            Ok(())
        })
        .on_fail(move |_, _| {
            barrier_ref3.add_permits(1);
            Ok(())
        })
        .show()
        .expect("unable to send notification");

    
    tokio::spawn(async move { 
        let _ = barrier.acquire().await.unwrap();
        if let Err(e) = tokio::fs::remove_file(path_to_image).await {
            println!("{}", e);
        };
        remove_downloaded_image_from_json(image_id);
        
    });
}

fn open_browser(url: &str) {
    webbrowser::open(url).unwrap();
}

struct NotificationData {
    name: String,
    image_path: String,
    url: String,
    notification_text: String,
    time_string: String
}

#[derive(Clone, Copy)]
pub enum UpcomingLaunchStatusId {
    ToBeDetermined,
    ToBeConfirmed,
    GoForLaunch,
    LaunchSuccessful,
    LaunchFailure,
    OnHold,
    LaunchInFlight,
    LaunchWasAPartialFailure,
}
impl UpcomingLaunchStatusId {
    pub fn from_id(id: i32) -> Option<Self> {
        use UpcomingLaunchStatusId::*;
        Some(match id {
            2 => ToBeDetermined,
            8 => ToBeConfirmed,
            1 => GoForLaunch,
            3 => LaunchSuccessful,
            4 => LaunchFailure,
            5 => OnHold,
            6 => LaunchInFlight,
            7 => LaunchWasAPartialFailure,
            _ => return None,
        })
    }
    pub fn is_launched(&self) -> bool {
        use UpcomingLaunchStatusId::*;
        matches!(*self, LaunchSuccessful | LaunchFailure | LaunchInFlight | LaunchWasAPartialFailure)
        //match *self {
        //    LaunchSuccessful | LaunchFailure | LaunchInFlight | LaunchWasAPartialFailure => true,
        //    _ => false,
        //}
    }
}

async fn check_for_notification(){
    let mut sent = get_send_notifications();
    let now = Utc::now().naive_utc();
    let res = match get_upcoming_launches().await {
        Some(c) => c,
        None => return
    };
    for launch in res.results.unwrap() {
        let date_time = match parse_time(&launch.net.clone().unwrap()) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("Error while parsing date time: {:?}", e);
                continue;
            }
        };
        let since = date_time.signed_duration_since(now).num_seconds();
        if since > -(60 * 60) {
            let mut data = Map::with_capacity(1);
                data.insert(
                    "launch_id".to_owned(),
                    serde_json::Value::String(launch.clone().id.unwrap().clone()),
                );
        }

        let is_launched = match UpcomingLaunchStatusId::from_id(launch.status.clone().unwrap().id.unwrap() as i32) {
            Some(id) => id.is_launched(),
            None => {
                eprintln!("Received unknown launch status id: {}", launch.status.unwrap().id.unwrap());
                since < 0
            }
        };
        if is_launched {
            let sent_launch = sent.get_sent(launch.id.as_ref().unwrap(), &launch.net.unwrap());
            if !sent_launch.now {
                sent_launch.now = true;
                let more_url = format!("https://spacelaunchnow.me/launch/{}", launch.slug.unwrap());
                let date_time_local: DateTime<Local> = Local.from_local_datetime(&date_time).unwrap();
                    let formated_date_string = date_time_local.format("%d/%m/%Y %H:%M").to_string();
                let noti = NotificationData {
                    name: launch.name.unwrap(),
                    image_path: launch.image.unwrap(),
                    url: more_url,
                    notification_text: "launched".to_string(),
                    time_string: formated_date_string
                };
                write_send_notifications(sent_launch.clone(), launch.id.unwrap());
                send_notification(noti).await;
            }

        }
        else if since < 24 * 60 * 60 {
            // less than one day ago, check if notifications should be sent
            let sent_launch = sent.get_sent(&launch.id.clone().unwrap(), &launch.net.unwrap());
            let send = if since < 60 * 60 {
                if sent_launch.one_hour {
                    false
                } else {
                    println!("In < one hour: {}", &launch.name.clone().unwrap());
                    sent_launch.one_hour = true;
                    true
                }
            } else if sent_launch.one_day {
                    false
                } else {
                    println!("In < 24 hours: {}", launch.name.clone().unwrap());
                    sent_launch.one_day = true;
                    true
                };
            if send {
                let total_minutes = (since / 60) as u32;
                let hours = (total_minutes / 60) as u32;
                let minutes = (total_minutes % 60) as u32;
                let time_text = match (hours, minutes) {
                    (23, m) if m > 30 => Cow::Borrowed("1 day"),
                    (0, m) if m > 55 => Cow::Borrowed("1 hour"),
                    (0, m) => Cow::Owned(format!("{} minutes", m)),
                    (1, _) => Cow::Borrowed("1 hour"),
                    (h, _) => Cow::Owned(format!("{} hours", h)),
                };
                println!("preparing push notifications, since: {}, hours: {}, minutes: {}, text: {}", since, hours, minutes, time_text);
                let more_url = format!("https://spacelaunchnow.me/launch/{}", launch.slug.unwrap());
                let date_time_local: DateTime<Local> = Local.from_local_datetime(&date_time).unwrap();
                let formated_date_string = date_time_local.format("%d/%m/%Y %H:%M").to_string();
        
                let notification = NotificationData{
                    name: launch.name.clone().unwrap(),
                    image_path: launch.image.unwrap(),
                    url: more_url,
                    notification_text: format!("launches in {}", time_text),
                    time_string: formated_date_string
                };
                write_send_notifications(sent_launch.clone(), launch.id.unwrap());
                send_notification(notification).await;
            }
        }
    }

}


async fn get_upcoming_launches() -> Option<upcoming_launch::Events> {
    let res;
    let url = "https://api.emeis.dev/launch/upcoming";
    let client = reqwest::Client::new();
    let response = client.get(url).send().await;
    match response {
        Ok(c) => res = c,
        Err(_) => return None
    };  

    match res.json::<upcoming_launch::Events>().await {
        Ok(parsed) => {
            Some(parsed)
        }
        Err(_) => {
            println!("Hm, the response didn't match the shape we expected.");
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SentNotifications {
    map: HashMap<String, LaunchNotifications>,
}

impl SentNotifications {
    fn get_sent<'a>(&'a mut self, id: &str, timestamp: &str) -> &'a mut LaunchNotifications {
        if let Some(val) = self.map.get_mut(id) {
            // if the timestamp changed and a launch notification wasn't already sent
            if !val.now && val.timestamp != timestamp {
                let reset_notifications = match (parse_time(&val.timestamp), parse_time(timestamp))
                {
                    (Err(e), _) | (_, Err(e)) => {
                        eprintln!("Failed to parse timestamp when parsing: {:?}", e);
                        false
                    }
                    (Ok(previous), Ok(new)) => {
                        let reset = previous.signed_duration_since(new).num_minutes() >= 5;
                        println!("Launch date was updated, id: {}, raw: {} => {}, parsed: {} => {}, reset: {}", id, val.timestamp, timestamp, previous, new, reset);
                        if !reset {
                            val.timestamp = timestamp.to_owned();
                        }
                        reset
                    }
                };

                if reset_notifications {
                    *val = LaunchNotifications::new(timestamp.to_owned());
                }
            }
        } else {
            println!(
                "New launch notification id found: id: {}, timestamp: {}",
                id, timestamp
            );
            self.map.insert(
                id.to_owned(),
                LaunchNotifications::new(timestamp.to_owned()),
            );
        }
        self.map.get_mut(id).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LaunchNotifications {
    timestamp: String,
    one_day: bool,
    one_hour: bool,
    now: bool,
}
impl LaunchNotifications {
    fn new(timestamp: String) -> Self {
        Self {
            timestamp,
            one_day: false,
            one_hour: false,
            now: false,
        }
    }
}

