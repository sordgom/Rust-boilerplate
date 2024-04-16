use crate::utils::spawn_app;
use chrono::NaiveDateTime;
use rstest::rstest;
use serde::{Deserialize, Serialize};
use sqlx::query;
use uuid::Uuid;
use zero2prod::models::appointments::{Appointment, ConsultationType};

#[rstest]
#[tokio::test]
#[case("CheckUp", 1737327600000, 60, "TEST")]
async fn booking_appointment_returns_200(
    #[case] consultancy_type: &str,
    #[case] timestamp: i64,
    #[case] duration: i32,
    #[case] description: &str,
) {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = &serde_json::json!({
        "patient_id": Uuid::new_v4(), // this has to change
        "doctor_id": Uuid::new_v4(),
        "consultancy_type": consultancy_type,
        "timestamp":  NaiveDateTime::from_timestamp_millis(timestamp),
        "duration": duration,
        "description": description
    });
    let response = client
        .post(&format!("{}/patients/appointments", &app.address))
        .basic_auth(app.test_user.name, Some(app.test_user.password))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = query!("SELECT consultancy_type FROM appointments")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    println!("{:?}", saved.consultancy_type);
    assert_eq!(saved.consultancy_type, "CheckUp");
}

#[rstest]
#[tokio::test]
#[case("bad input")]
async fn booking_appointment_returns_error(#[case] input: &str) {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/patients/appointments", &app.address))
        .header("Content-Type", "application/json")
        .body(input.to_string())
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(400, response.status().as_u16());
}

#[derive(Serialize, Deserialize, Debug)]
struct APIResponse {
    status: String,
    data: Vec<Appointment>,
    length: usize,
}

#[rstest]
#[tokio::test]
#[case(
    "036bd774-9e6b-4907-b74c-e76c24ac5784",
    "036bd774-9e6b-4907-b74c-e76c24ac5784"
)]
async fn viewing_appointments_returns_200(
    #[case] patient_id: uuid::Uuid,
    #[case] doctor_id: uuid::Uuid,
) {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let _saved = query!(r#"
        INSERT INTO appointments (id, patient_id, doctor_id, consultancy_type, description, timestamp, duration, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
    "#, 
    Uuid::parse_str("036bd774-9e6b-4907-b74c-e76c24ac5784").unwrap(), 
    patient_id,
    doctor_id,
    "CheckUp", 
    "TEST", 
    NaiveDateTime::from_timestamp_millis(1737327600000),
    60,
    NaiveDateTime::from_timestamp_millis(1737327600000),
    NaiveDateTime::from_timestamp_millis(1737327600000))
        .execute(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    // Patient API test
    let patient_query = format!("?id={}", patient_id.to_string());
    let patient_response = client
        .get(&format!(
            "{}/patients/appointments{}",
            &app.address, patient_query
        ))
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(200, patient_response.status().as_u16());
    let api_response = patient_response
        .json::<APIResponse>()
        .await
        .expect("Failed to parse response.");
    let patient_appointments = &api_response.data;
    assert_eq!(1, patient_appointments.len());
    assert_eq!(
        patient_appointments[0].consultancy_type,
        ConsultationType::CheckUp
    );

    // Doctor API test
    let doctor_query = format!("?id={}", doctor_id.to_string());
    let doctor_response = client
        .get(&format!(
            "{}/doctors/appointments{}",
            &app.address, doctor_query
        ))
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(200, doctor_response.status().as_u16());
    let api_response = doctor_response
        .json::<APIResponse>()
        .await
        .expect("Failed to parse response.");
    let doctor_appointments = &api_response.data;
    assert_eq!(1, doctor_appointments.len());
    assert_eq!(
        doctor_appointments[0].consultancy_type,
        ConsultationType::CheckUp
    );
}
