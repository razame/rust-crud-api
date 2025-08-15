#[macro_use]
extern crate rocket;
extern crate diesel;

mod schema;
mod models;
use rocket_sync_db_pools::database;
use rocket::serde::json::Json;
use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use models::{User, NewUser};



use aws_sdk_s3::primitives::ByteStream;
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct UploadRequest {
    url: String,
}



#[get("/users")]
async fn get_users(conn: DbConn) -> Json<Vec<User>> {
    use schema::users::dsl::*;
    let results = conn.run(move |c| {
        users.load::<User>(c)
    }).await.expect("Error loading users");

    Json(results)
}

#[get("/users/<user_id>")]
async fn get_user(conn: DbConn, user_id: i32) -> Option<Json<User>> {
    use crate::schema::users::dsl::*;

    conn.run(move |c| {
        users
            .filter(id.eq(user_id))
            .first::<User>(c)
            .ok()
    }).await.map(Json)
}


#[post("/users", data = "<new_user>")]
async fn create_user(conn: DbConn, new_user: Json<NewUser>) -> Json<User> {
    use schema::users;

    let new_user_data = new_user.into_inner();

    // 👇 clone what you need before the move
    let name = new_user_data.name.clone();
    let email = new_user_data.email.clone();

    conn.run(move |c| {
        diesel::insert_into(users::table)
            .values(&new_user_data)
            .execute(c)
    }).await.expect("Error inserting user");

    Json(User {
        id: 0, // Consider querying the DB for the inserted ID
        name,
        email,
    })
}

#[put("/users/<user_id>", data = "<updated_user>")]
async fn update_user(conn: DbConn, user_id: i32, updated_user: Json<NewUser>) -> &'static str {
    use schema::users::dsl::*;

    let new_data = updated_user.into_inner();

    conn.run(move |c| {
        diesel::update(users.filter(id.eq(user_id)))
            .set((name.eq(new_data.name), email.eq(new_data.email)))
            .execute(c)
    }).await.expect("Error updating user");

    "User updated"
}



#[delete("/users/<user_id>")]
async fn delete_user(conn: DbConn, user_id: i32) -> &'static str {
    use schema::users::dsl::*;
    conn.run(move |c| {
        diesel::delete(users.filter(id.eq(user_id)))
            .execute(c)
    }).await.expect("Error deleting user");

    "User deleted"
}

#[get("/billion-iterations")]
async fn billion_iterations() -> &'static str {
    
    let mut i = 0;

    while i < 2100000000 {
        i = i + 1;
    }

    "Completed Billion Iterations"
}



// =============================================================================================
// =============================================================================================


#[post("/upload", data = "<payload>")]
async fn upload_file_from_url_to_s3(payload: Json<UploadRequest>) -> Json<serde_json::Value> {
    if !payload.url.starts_with("http") {
        return Json(serde_json::json!({
            "success": false,
            "message": "Invalid URL"
        }));
    }

    match download_and_upload(&payload.url).await {
        Ok(url) => Json(serde_json::json!({
            "success": true,
            "file_url": url
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": e
        })),
    }
}

async fn download_and_upload(file_url: &str) -> Result<String, String> {
    // Download file
    let resp = reqwest::get(file_url).await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err("Failed to download file".to_string());
    }

    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;

    // Init AWS S3 Client
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let client = aws_sdk_s3::Client::new(&config);

    // Guess file extension
    let ext = mime_guess::from_path(file_url)
        .first_or_text_plain()
        .subtype()
        .as_str()
        .to_string();

    let file_name = format!("uploads/{}.{}", Uuid::new_v4(), ext);

    // Upload to S3
    client.put_object()
        .bucket(std::env::var("AWS_BUCKET").unwrap())
        .key(&file_name)
        .body(ByteStream::from(bytes.to_vec()))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let bucket_url = std::env::var("AWS_URL")
        .unwrap_or_else(|_| format!("https://{}.s3.amazonaws.com", std::env::var("AWS_BUCKET").unwrap()));

    Ok(format!("{}/{}", bucket_url, file_name))
}




// use fake::faker::{
//     company::en::Bs,
//     lorem::en::{Sentence, Word},
// };
// use fake::Fake;
// use rand::Rng;
// use serde::{Deserialize, Serialize};
// use std::fs::{self, File};
// use std::io::Write;

// // Configure these constants
// const PHASES: usize = 20;
// const CONTENTS_PER_PHASE: usize = 50;
// const DISTRIBUTIONS_PER_CONTENT: usize = 200;
// // const PHASES: usize = 2; // For testing
// // const CONTENTS_PER_PHASE: usize = 3; // For testing
// // const DISTRIBUTIONS_PER_CONTENT: usize = 4; // For testing

// #[derive(Debug, Serialize, Deserialize)]
// struct Campaign {
//     name: String,
//     status: String,
//     brand_promoting: String,
//     brand_industry: String,
//     brand_sub_industry: String,
//     phases: Vec<Phase>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct Phase {
//     name: String,
//     phase_title: String,
//     no_of_content_pieces: usize,
//     earliest_phase_launch_date: String,
//     earliest_phase_end_date: String,
//     latest_phase_launch_date: String,
//     latest_phase_end_date: String,
//     solution_id: usize,
//     contents: Vec<Content>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct Content {
//     name: String,
//     #[serde(rename = "type")]
//     content_type: String,
//     content_distributions: Vec<ContentDistribution>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct ContentDistribution {
//     name: String,
//     distribution_type: u8,
//     distribution_category: u8,
// }

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // Create output directory
//     let output_dir = "campaign_data";
//     fs::create_dir_all(output_dir)?;

//     // Generate campaign data
//     let mut campaign = Campaign {
//         name: "Spring Sale Campaign".to_string(),
//         status: "draft".to_string(),
//         brand_promoting: "Acme™".to_string(),
//         brand_industry: "Automotive".to_string(),
//         brand_sub_industry: "Automotive Services".to_string(),
//         phases: Vec::with_capacity(PHASES),
//     };

//     // Generate platforms list
//     let platforms = vec![
//         "YouTube", "Instagram", "Facebook", "TikTok", "LinkedIn",
//         "Twitter", "Pinterest", "Google Display", "Reddit", "Snapchat",
//         "AutomotiveNews", "CarEnthusiastBlog", "VehicleReviewSites",
//         "EmailNewsletter", "InfluencerPartnerships", "PodcastAds",
//         "RadioSpots", "TVCommercials", "Billboards", "DirectMail",
//     ];

//     for phase_num in 1..=PHASES {
//         let phase = generate_phase(phase_num, &platforms);
//         campaign.phases.push(phase);
        
//         // Write individual phase file
//         let phase_file = format!("{}/phase_{}.json", output_dir, phase_num);
//         let json = serde_json::to_string_pretty(&campaign.phases[phase_num - 1])?;
//         let mut file = File::create(phase_file)?;
//         file.write_all(json.as_bytes())?;
//     }

//     // Write complete campaign file
//     let complete_file = format!("{}/complete_campaign.json", output_dir);
//     let json = serde_json::to_string_pretty(&campaign)?;
//     let mut file = File::create(complete_file)?;
//     file.write_all(json.as_bytes())?;

//     println!("✅ Generated {} phases with {} contents each", PHASES, CONTENTS_PER_PHASE);
//     Ok(())
// }

// fn generate_phase(phase_num: usize, platforms: &[&str]) -> Phase {
//     let mut _rng = rand::thread_rng();
    
//     Phase {
//         name: format!("Phase {} - {}", phase_num, Bs().fake::<String>()),
//         phase_title: Sentence(1..3).fake(),
//         no_of_content_pieces: CONTENTS_PER_PHASE,
//         earliest_phase_launch_date: "2025-06-01".to_string(),
//         earliest_phase_end_date: "2025-06-05".to_string(),
//         latest_phase_launch_date: "2025-06-03".to_string(),
//         latest_phase_end_date: "2025-06-08".to_string(),
//         solution_id: 100 + phase_num,
//         contents: (1..=CONTENTS_PER_PHASE)
//             .map(|content_num| generate_content(phase_num, content_num, platforms))
//             .collect(),
//     }
// }

// fn generate_content(phase_num: usize, content_num: usize, platforms: &[&str]) -> Content {
//     let content_types = vec![
//         "video", "image", "carousel", "blog", 
//         "story", "infographic", "email", "whitepaper"
//     ];
    
//     Content {
//         name: format!(
//             "Content {}-{}: {}",
//             phase_num,
//             content_num,
//             Sentence(3..6).fake::<String>()
//         ),
//         content_type: content_types[rand::thread_rng().gen_range(0..content_types.len())]
//             .to_string(),
//         content_distributions: (0..DISTRIBUTIONS_PER_CONTENT)
//             .map(|_| generate_distribution(platforms))
//             .collect(),
//     }
// }

// fn generate_distribution(platforms: &[&str]) -> ContentDistribution {
//     let mut rng = rand::thread_rng();
    
//     ContentDistribution {
//         name: format!(
//             "{} {}",
//             platforms[rng.gen_range(0..platforms.len())],
//             Word().fake::<String>().to_uppercase()
//         ),
//         distribution_type: rng.gen_range(1..=5),
//         distribution_category: rng.gen_range(1..=5),
//     }
// }



// --- DB connection wrapper ---
#[database("mysql_db")]
struct DbConn(MysqlConnection);

#[launch]
fn rocket() -> _ {
    dotenvy::dotenv().ok();

    rocket::build()
        .attach(DbConn::fairing())
        .mount(
            "/",
            routes![
                get_users, create_user,
                delete_user, update_user,
                get_user, billion_iterations,
                upload_file_from_url_to_s3
            ]
        )
}