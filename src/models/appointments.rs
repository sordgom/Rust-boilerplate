use chrono::NaiveDateTime;
use enum_display::EnumDisplay;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, EnumDisplay, Serialize, Deserialize, PartialEq, Eq, Copy, Clone)]
pub enum ConsultationType {
    CheckUp,
    FollowUp,
    Whitening,
    Filling,
    Extraction,
    Braces,
    Implants,
}

impl From<String> for ConsultationType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "CheckUp" => ConsultationType::CheckUp,
            "Follow Up" => ConsultationType::FollowUp,
            "Whitening" => ConsultationType::Whitening,
            "Filling" => ConsultationType::Filling,
            "Extraction" => ConsultationType::Extraction,
            "Braces" => ConsultationType::Braces,
            "Implants" => ConsultationType::Implants,
            _ => panic!("Invalid ConsultationType"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow, PartialEq, Eq)]
pub struct Appointment {
    pub id: Option<Uuid>,
    pub patient_id: Uuid,
    pub doctor_id: Uuid,
    pub consultancy_type: ConsultationType,
    pub timestamp: NaiveDateTime,
    pub duration: i32,
    pub description: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
