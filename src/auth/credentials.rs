use actix_web::http::header::HeaderMap;
use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use base64::Engine;
use secrecy::{ExposeSecret, Secret};
use sqlx::{query, PgPool};

use crate::telemetry::spawn_blocking_with_tracing;

pub struct Credentials {
    pub username: String,
    pub password: Secret<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Authentication failed")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

pub fn basic_auth(headers: &HeaderMap) -> Result<Credentials, AuthError> {
    let auth_header = headers
        .get("Authorization")
        .context("Missing Authorization header")?
        .to_str()
        .context("Failed to parse Authorization header")?;
    let base64 = auth_header
        .strip_prefix("Basic ")
        .context("Invalid Authorization header")?;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(base64)
        .context("Failed to decode base64")?;
    let decoded_credentials = String::from_utf8(decoded).context("Invalid UTF-8")?;

    // Split it into 2 sections separated by :
    let mut creds = decoded_credentials.splitn(2, ':');
    let username = creds
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing username in Basic Auth"))?
        .to_string();
    let password = creds
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing password in Basic Auth"))?
        .to_string();
    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
}

pub async fn validate_creds(
    credentials: Credentials,
    pool: &PgPool,
) -> Result<uuid::Uuid, AuthError> {
    let mut user_id = None;
    // To mitigate a timing attack, we introduced a new fallback pwd
    let mut expected_password_hash = Secret::new(
        "$argon2id$v=19$m=15000,t=2,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
            .to_string(),
    );

    if let Some((stored_user_id, stored_password_hash)) =
        get_stored_credentials(&credentials.username, pool).await?
    {
        user_id = Some(stored_user_id);
        expected_password_hash = stored_password_hash;
    }

    // This tasks heavy resources since it takes around 1ms for it to be finish executing
    // Hence we'll spawn a new blocking thread so it wouldnt interfere with the scheduling of async tasks
    spawn_blocking_with_tracing(move || {
        verify_password_hash(expected_password_hash, credentials.password)
    })
    .await
    // Handle blocking thread errors
    .context("Failed to spawn a blocking task")??;

    user_id
        .ok_or_else(|| anyhow::anyhow!("User not found"))
        .map_err(AuthError::InvalidCredentials)
}

#[tracing::instrument(
    name = "Validate credentials",
    skip(expected_password_hash, password_candidate)
)]
fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>,
) -> Result<(), AuthError> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse hash in PHC string format.")?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .context("Invalid password.")
        .map_err(AuthError::InvalidCredentials)
}

#[tracing::instrument(name = "Get store credentials", skip(username, pool))]
async fn get_stored_credentials(
    username: &str,
    pool: &PgPool,
) -> Result<Option<(uuid::Uuid, Secret<String>)>, anyhow::Error> {
    let row = query!(
        r#"
            SELECT id, password FROM users WHERE name = $1
            "#,
        username
    )
    .fetch_optional(pool)
    .await
    .context("Failed to query user from db")?
    .map(|row| (row.id, Secret::new(row.password)));
    Ok(row)
}
