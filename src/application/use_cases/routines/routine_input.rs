use std::collections::HashMap;

use crate::domain::{
    entities::{Session, SessionRef},
    value_objects::{DifficultyLevel, Error},
};

pub struct RoutineInput {
    pub title: String,
    pub level: DifficultyLevel,
    pub duration_weeks: u16,
    pub description: String,
    pub session_ids: Vec<String>,
}

pub fn validate_routine(input: &RoutineInput) -> Result<(), Error> {
    if input.title.trim().is_empty() {
        return Err(Error::bad_request("title must not be empty"));
    }

    if input.duration_weeks == 0 {
        return Err(Error::bad_request("duration_weeks must be greater than 0"));
    }

    if input.session_ids.is_empty() {
        return Err(Error::bad_request("a routine needs at least one session"));
    }

    Ok(())
}

pub fn build_session_refs(
    session_ids: Vec<String>,
    catalog: &[Session],
) -> Result<Vec<SessionRef>, Error> {
    let catalog: HashMap<&str, &Session> = catalog
        .iter()
        .map(|session| (session.id.as_str(), session))
        .collect();

    session_ids
        .into_iter()
        .map(|session_id| {
            let session = catalog.get(session_id.as_str()).ok_or_else(|| {
                Error::bad_request(&format!("Session {session_id} does not exist"))
            })?;

            Ok(SessionRef {
                name: session.name.clone(),
                session_id,
            })
        })
        .collect()
}
