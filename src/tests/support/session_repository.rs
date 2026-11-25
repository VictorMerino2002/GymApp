use std::{collections::HashMap, sync::Mutex};

use async_trait::async_trait;

use crate::{
    application::ports::repositories::SessionRepository,
    domain::{
        entities::Session,
        value_objects::{Error, Muscle, Page},
    },
};

#[derive(Default)]
pub struct InMemorySessionRepository {
    sessions: Mutex<HashMap<String, Session>>,
}

impl InMemorySessionRepository {
    pub fn with(sessions: Vec<Session>) -> Self {
        let repository = Self::default();
        for session in sessions {
            repository.insert(session);
        }
        repository
    }

    pub fn insert(&self, session: Session) {
        self.sessions
            .lock()
            .unwrap()
            .insert(session.id.clone(), session);
    }

    pub fn get(&self, id: &str) -> Option<Session> {
        self.sessions.lock().unwrap().get(id).cloned()
    }
}

#[async_trait]
impl SessionRepository for InMemorySessionRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Session>, Error> {
        Ok(self.get(id))
    }

    async fn find_by_ids(&self, ids: &[String]) -> Result<Vec<Session>, Error> {
        let sessions = self.sessions.lock().unwrap();
        Ok(ids
            .iter()
            .filter_map(|id| sessions.get(id).cloned())
            .collect())
    }

    async fn find_paginated(
        &self,
        _limit: u16,
        _cursor: Option<&str>,
        _muscle: Option<Muscle>,
    ) -> Result<Page<Session>, Error> {
        unimplemented!()
    }

    async fn find_by_author(
        &self,
        _author_id: &str,
        _limit: u16,
        _cursor: Option<&str>,
        _muscle: Option<Muscle>,
    ) -> Result<Page<Session>, Error> {
        unimplemented!()
    }

    async fn save(&self, session: &Session) -> Result<(), Error> {
        self.insert(session.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), Error> {
        self.sessions.lock().unwrap().remove(id);
        Ok(())
    }
}
