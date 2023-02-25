use std::{collections::{HashMap, BTreeMap}, sync::Arc};

use anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A lead e.g. a company.
#[derive(clap::Args, Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct CompanyName {
    name: Arc<str>,
}
impl From<String> for CompanyName {
    fn from(name: String) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct InterviewName {
    name: Arc<str>,
}
impl From<String> for InterviewName {
    fn from(name: String) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Leads {
    /// All our leads, indexed by the company name.
    #[serde(flatten)]
    pub leads: HashMap<CompanyName, Vec<Lead>>,
}
impl Leads {
    pub fn new() -> Self {
        Self {
            leads: HashMap::new(),
        }
    }
    pub fn from_path(path: &std::path::Path) -> Result<Self, anyhow::Error> {
        let db = match std::fs::File::open(path) {
            // Reuse file if possible.
            Ok(db_file) => serde_yaml::from_reader(&db_file)
                .with_context(|| format!("Invalid yaml file {}", path.display()))?,

            // Create the file if it doesn't exist.
            Err(ref err) if matches!(err.kind(), std::io::ErrorKind::NotFound) => Leads::new(),

            // Otherwise, propagate error.
            Err(err) => {
                return Err(err).with_context(|| {
                    format!("Error while reading file {path}", path = path.display())
                })
            }
        };
        Ok(db)
    }
    pub fn new_lead(&mut self, name: CompanyName, position: String, source: String) -> usize {
        let lead = Lead::new(position, source);
        self.push_lead(name, lead)
    }
    pub fn push_lead(&mut self, name: CompanyName, lead: Lead) -> usize {
        let positions = self.leads.entry(name).or_default();
        positions.push(lead);
        positions.len() - 1
    }
    pub fn close_lead(
        &mut self,
        name: &CompanyName,
        index: Option<usize>,
        reason: String,
    ) -> Result<Lead, anyhow::Error> {
        let positions = self.leads.get_mut(name).context("No such company")?;
        let mut lead = match index {
            None if positions.len() == 1 => positions.pop().unwrap(),
            None => {
                return Err(anyhow!(
                    "There are {} positions for this company, please specify which one to close",
                    positions.len()
                ))
            }
            Some(index) if index < positions.len() => positions.remove(index),
            Some(index) => {
                return Err(anyhow!(
                    "There are only {} positions for this company, cannot close position {}",
                    positions.len(),
                    index
                ))
            }
        };
        lead.closed = Some(reason);
        Ok(lead)
    }
    pub fn get_mut(
        &mut self,
        name: &CompanyName,
        index: Option<usize>,
    ) -> Result<&mut Lead, anyhow::Error> {
        let positions = self.leads.get_mut(name).context("No such company")?;
        let lead = match index {
            None if positions.len() == 1 => &mut positions[0],
            None => {
                return Err(anyhow!(
                    "There are {} positions for this company, please specify which one to modify",
                    positions.len()
                ))
            }
            Some(index) if index < positions.len() => &mut positions[index],
            Some(index) => {
                return Err(anyhow!(
                    "There are only {} positions for this company, cannot modify position {}",
                    positions.len(),
                    index
                ))
            }
        };
        Ok(lead)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Lead {
    /// The name of the position.
    position: String,

    /// The source of the lead, typically a URL.
    source: String,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    notes: HashMap<String, Vec<String>>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    interviews: Vec<(InterviewName, Interview)>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    red_flags: Vec<String>,

    /// The status updates, from oldest to most recent.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    status_updates: BTreeMap<DateTime<Utc>, String>,

    /// The reason for which this lead was closed, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    closed: Option<String>,
}

impl Lead {
    pub fn new(position: String, source: String) -> Self {
        Self {
            position,
            source,
            interviews: Vec::new(),
            red_flags: Vec::new(),
            status_updates: vec![(Utc::now(), "Created".to_string())].into_iter().collect(),
            closed: None,
            notes: HashMap::new(),
        }
    }

    /// Add a note.
    pub fn add_note(&mut self, name: String, note: String) {
        self.notes.entry(name).or_default().push(note);
    }

    /// Add a status update.
    pub fn add_status(&mut self, date: DateTime<Utc>, status: String) {
        self.status_updates.insert(date, status);
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Interview {
    pre_notes: Vec<String>,
    post_notes: Vec<String>,
}
