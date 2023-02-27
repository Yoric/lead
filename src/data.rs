use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc, fmt::Display,
};

use anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A lead e.g. a company.
#[derive(clap::Args, Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct CompanyName {
    name: Arc<str>,
}
impl Display for CompanyName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
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
impl Default for Leads {
    fn default() -> Self {
        Self::new()
    }
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
        date: DateTime<Utc>,
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
        lead.add_status(date, format!("Closed: {}", reason));

        // Cleanup if it's the last position for this company.
        if positions.is_empty() {
            self.leads.remove(name);
        }
        Ok(lead)
    }
    pub fn get(
        &self,
        name: &CompanyName,
        index: Option<usize>,
    ) -> Result<&Lead, anyhow::Error> {
        let positions = self.leads.get(name).context("No such company")?;
        let lead = match index {
            None if positions.len() == 1 => &positions[0],
            None => {
                return Err(anyhow!(
                    "There are {} positions for this company, please specify which one to modify",
                    positions.len()
                ))
            }
            Some(index) if index < positions.len() => &positions[index],
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

impl<'a> std::iter::IntoIterator for &'a Leads {
    type Item = (&'a CompanyName, &'a Vec<Lead>);
    type IntoIter = std::collections::hash_map::Iter<'a, CompanyName, Vec<Lead>>;

    fn into_iter(self) -> Self::IntoIter {
        self.leads.iter()
    }
}

impl<'a> std::iter::IntoIterator for &'a mut Leads {
    type Item = (&'a CompanyName, &'a Vec<Lead>);
    type IntoIter = std::collections::hash_map::Iter<'a, CompanyName, Vec<Lead>>;

    fn into_iter(self) -> Self::IntoIter {
        self.leads.iter()
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

    /// The todo list (things that the candidate needs to do), from oldest to most recent.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    todo: Vec<Todo>,

    /// The waitlist (things that the employer needs to do), from oldest to most recent.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    wait: Vec<Wait>,
}

impl Lead {
    pub fn new(position: String, source: String) -> Self {
        Self {
            position,
            source,
            interviews: Vec::new(),
            red_flags: Vec::new(),
            status_updates: vec![(Utc::now(), "Created".to_string())]
                .into_iter()
                .collect(),
            notes: HashMap::new(),
            todo: Vec::new(),
            wait: Vec::new(),
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

    pub fn add_todo(&mut self, updated_on: DateTime<Utc>, action: String, deadline: DateTime<Utc>) {
        self.add_status(updated_on, format!("TODO: {}", action));
        self.todo.push(Todo { action, deadline });
    }

    pub fn complete_todo(&mut self, updated_on: DateTime<Utc>, index: usize) -> Result<(), anyhow::Error> {
        if index >= self.todo.len() {
            return Err(anyhow!("No such todo"));
        }
        let todo = self.todo.remove(index);
        self.add_status(updated_on, format!("DONE: {}", todo.action));
        Ok(())
    }

    pub fn add_wait(&mut self, updated_on: DateTime<Utc>, action: String, expected: Option<DateTime<Utc>>) {
        self.add_status(updated_on, format!("WAITING: {}", action));
        self.wait.push(Wait { action, expected });
    }

    pub fn complete_wait(&mut self, updated_on: DateTime<Utc>, index: usize) -> Result<(), anyhow::Error> {
        if index >= self.wait.len() {
            return Err(anyhow!("No such wait"));
        }
        let wait = self.wait.remove(index);
        self.add_status(updated_on, format!("RECEIVED: {}", wait.action));
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Interview {
    pre_notes: Vec<String>,
    post_notes: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Todo {
    action: String,
    deadline: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Wait {
    action: String,
    expected: Option<DateTime<Utc>>,
}
