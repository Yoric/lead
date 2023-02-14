use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct LeadName(String);

#[derive(Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct InterviewName(String);

#[derive(Debug, Deserialize, Serialize)]
pub struct Data {
    /// All our leads, opened or closed keyed by
    pub leads: HashMap<LeadName, Lead>,
}
impl Data {
    pub fn new_lead(&mut self, name: String) -> Result<Lead, anyhow::Error> {
        unimplemented!()
    }
    pub fn lead(&self, name: &str) -> Result<&Lead, Vec<&str>> {
        unimplemented!()
    }
    pub fn lead_mut(&mut self, name: &str) -> Result<&mut Lead, Vec<&str>> {
        unimplemented!()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Lead {
    interviews: Vec<(InterviewName, Interview)>,
    red_flags: Vec<String>,

    /// The status updates, from oldest to most recent.
    status_updates: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Interview {
    pre_notes:  Vec<String>,
    post_notes: Vec<String>,
}