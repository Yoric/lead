use chrono::{ DateTime, Utc };
use data::LeadName;

mod data;

enum Command {
    /// Create a new lead.
    New {
        lead: LeadName,
        updated_on: Option<DateTime<Utc>>
    },

    /// Close a lead.
    Close {
        lead: LeadName,
        reason: String,
        updated_on: Option<DateTime<Utc>>
    },

    /// Things to know about an upcoming interview.
    PreInterview {
        lead: LeadName,

        /// The name or stage of the interview.
        interview: String,
        notes: String,
        updated_on: Option<DateTime<Utc>>,
        planned: Option<DateTime<Utc>>
    },

    /// Things to know after an incoming interview.
    PostInterview {
        lead: LeadName,

        /// The name or stage of the interview.
        interview: String,
        notes: String,
        held_on: DateTime<Utc>,
        updated_on: Option<DateTime<Utc>>,
    },

    RedFlag {
        lead: LeadName,
        comment: String,
        updated_on: Option<DateTime<Utc>>,
    },

    /// Update
    Status {
        lead: LeadName,
        status: String,
        updated_on: Option<DateTime<Utc>>,
    },

    Detail {
        lead: LeadName,
        /// The kind of detail, e.g. "Open to remote", "Salary", etc.
        kind: String,
        updated_on: Option<DateTime<Utc>>,
    },

    /// Misc note.
    Note {
        name: String,
        note: String,
        updated_on: Option<DateTime<Utc>>,
    },
}

fn main() {
    println!("Hello, world!");
}
