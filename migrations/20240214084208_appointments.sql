-- Add migration script here
CREATE TABLE IF NOT EXISTS appointments (
    id uuid PRIMARY KEY NOT NULL,
    patient_id uuid NOT NULL,
    doctor_id uuid NOT NULL,
    consultancy_type TEXT NOT NULL,
    description TEXT,
    timestamp TIMESTAMP NOT NULL,
    duration INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP 
);