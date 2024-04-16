use actix_web::{
    http::header::{self, HeaderValue},
    web, HttpRequest, HttpResponse, ResponseError,
};
use chrono::{NaiveDateTime, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::{query, FromRow, PgPool};
use uuid::Uuid;

use crate::auth::{basic_auth, validate_creds, AuthError};
use crate::models::appointments::{Appointment, ConsultationType};
use crate::models::user::UserRequest;

#[derive(Debug, thiserror::Error)]
pub enum BookingError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for BookingError {
    fn error_response(&self) -> HttpResponse {
        match self {
            BookingError::AuthError(_) => {
                let mut response = HttpResponse::new(StatusCode::UNAUTHORIZED);
                let header_value = HeaderValue::from_str("Basic realm=\"Restricted\"").unwrap();
                response
                    .headers_mut()
                    .insert(header::WWW_AUTHENTICATE, header_value);
                response
            }
            BookingError::UnexpectedError(_) => {
                HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow, PartialEq, Eq)]
pub struct AppointmentForm {
    pub patient_id: Uuid,
    pub doctor_id: Uuid,
    pub consultancy_type: ConsultationType,
    pub timestamp: NaiveDateTime,
    pub duration: i32,
    pub description: Option<String>,
}

impl TryFrom<AppointmentForm> for Appointment {
    type Error = String;

    fn try_from(value: AppointmentForm) -> Result<Self, Self::Error> {
        if !is_valid_timestamp(value.timestamp) {
            return Err("Invalid timestamp".to_string());
        }
        Ok(Appointment {
            id: Uuid::new_v4().into(),
            patient_id: value.patient_id,
            doctor_id: value.doctor_id,
            consultancy_type: value.consultancy_type,
            timestamp: value.timestamp,
            duration: value.duration,
            description: value.description,
            created_at: NaiveDateTime::from_timestamp_opt(Utc::now().timestamp(), 0),
            updated_at: NaiveDateTime::from_timestamp_opt(Utc::now().timestamp(), 0),
        })
    }
}

#[tracing::instrument(
    name = "Booking a new appointment",
    skip(appointment_data, connection),
    fields(
        consultancy_type = %appointment_data.consultancy_type,
    )
)]
pub async fn booking_appointment(
    appointment_data: web::Json<AppointmentForm>,
    connection: web::Data<PgPool>,
    request: HttpRequest,
) -> Result<HttpResponse, BookingError> {
    let credentials =
        basic_auth(request.headers()).map_err(|e| BookingError::AuthError(e.into()))?;
    tracing::Span::current().record("username", credentials.username.as_str());

    let user_id = validate_creds(credentials, &connection)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => BookingError::AuthError(e.into()),
            AuthError::UnexpectedError(_) => BookingError::UnexpectedError(e.into()),
        })?;
    tracing::Span::current().record("user_id", &user_id.to_string());

    let appointment = match appointment_data.0.try_into() {
        Ok(val) => val,
        Err(_) => return Ok(HttpResponse::BadRequest().finish()),
    };

    match insert_appointment(appointment, &connection).await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

#[tracing::instrument(
    name = "Saving new appointment details in the database",
    skip(appointment, pool)
)]
pub async fn insert_appointment(
    appointment: Appointment,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    query!(
        r#"
        INSERT INTO appointments (id, patient_id, doctor_id, consultancy_type, description, timestamp, duration, created_at, updated_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
        appointment.id,
        appointment.patient_id, //This has to be changed
        appointment.doctor_id,
        appointment.consultancy_type.to_string(),
        appointment.description,
        appointment.timestamp,
        appointment.duration,
        appointment.created_at,
        appointment.updated_at
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(name = "Fetching user appointment details", skip(user, pool))]
pub async fn get_appointments_query(
    user: web::Query<UserRequest>,
    pool: &PgPool,
    is_doctor: bool,
) -> Result<Vec<Appointment>, sqlx::Error> {
    let result = if is_doctor {
        sqlx::query_as!(
            Appointment,
            r#"SELECT * FROM appointments WHERE doctor_id = $1"#,
            user.id,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {}", e);
            e
        })?
    } else {
        sqlx::query_as!(
            Appointment,
            r#"SELECT * FROM appointments WHERE patient_id = $1"#,
            user.id,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {}", e);
            e
        })?
    };
    Ok(result)
}

#[tracing::instrument(name = "Patient views their appointments", skip(user, connection))]
pub async fn get_patient_appointments(
    user: web::Query<UserRequest>,
    connection: web::Data<PgPool>,
) -> HttpResponse {
    match get_appointments_query(user, &connection, false).await {
        Ok(val) => {
            let json_response = serde_json::json!({
                "status": "success",
                "length": val.len(),
                "data": val
            });
            HttpResponse::Ok().json(json_response)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(name = "Doctor views their appointments", skip(user, connection))]
pub async fn get_doctor_appointments(
    user: web::Query<UserRequest>,
    connection: web::Data<PgPool>,
) -> HttpResponse {
    match get_appointments_query(user, &connection, true).await {
        Ok(val) => {
            let json_response = serde_json::json!({
                "status": "success",
                "length": val.len(),
                "data": val
            });
            HttpResponse::Ok().json(json_response)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

fn is_valid_timestamp(dt: NaiveDateTime) -> bool {
    let current_time = chrono::Local::now().naive_local();
    dt > current_time
}

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
