use chrono::NaiveDateTime;
use rstest::rstest;
use uuid::Uuid;

use crate::utils::spawn_app;

#[rstest]
#[tokio::test]
#[case("CheckUp", 1737327600000, 60, "TEST")]
async fn requests_missing_authorization_are_rejected(
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
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(401, response.status().as_u16());
    assert_eq!(
        r#"Basic realm="Restricted""#,
        response.headers()["WWW-Authenticate"]
    );
}

#[rstest]
#[tokio::test]
#[case("CheckUp", 1737327600000, 60, "TEST")]
async fn non_existing_user_is_rejected(
    #[case] consultancy_type: &str,
    #[case] timestamp: i64,
    #[case] duration: i32,
    #[case] description: &str,
) {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let username = uuid::Uuid::new_v4().to_string();
    let password = uuid::Uuid::new_v4().to_string();

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
        .basic_auth(username, Some(password))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    //Assert
    assert_eq!(401, response.status().as_u16());
    assert_eq!(
        r#"Basic realm="Restricted""#,
        response.headers()["WWW-Authenticate"]
    );
}
