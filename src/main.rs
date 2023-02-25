use std::path::PathBuf;

use anyhow::Context;
use chrono::{DateTime, Utc};
use clap::Parser;
use data::CompanyName;

mod data;

#[derive(clap::Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// The name of the company.
    company: CompanyName,

    /// If there is more than one position at the company, the index of the position.
    index: Option<usize>,

    #[command(subcommand)]
    command: Command,

    #[arg(default_value_t=Utc::now())]
    updated_on: DateTime<Utc>,

    #[arg(long, default_value_t={"leads.yml".to_string()})]
    file: String,

    #[arg(long, default_value_t={"archive.yml".to_string()})]
    archive: String,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// Create a new lead.
    New {
        /// The name of the position.
        position: String,

        /// The source for this lead, typically a URL.
        source: String,
    },

    /// Close a lead.
    Close {
        reason: String,
    },

    /// Misc note.
    #[command(subcommand)]
    Note(NoteCommand),

    /// Things to prepare for a lead
    ToDo {
        /// A description of the task.
        description: String,
    },

    /// Things to know about an upcoming interview.
    PreInterview {
        /// The name or stage of the interview.
        interview: String,
        notes: String,

        /// The date the interview is planned for.
        planned: Option<DateTime<Utc>>,
    },

    /// Things to know after an incoming interview.
    PostInterview {
        /// The name or stage of the interview.
        interview: String,
        notes: String,
        held_on: DateTime<Utc>,
    },

    RedFlag {
        comment: String,
    },

    /// Update
    Status {
        status: String,
    },
}

#[derive(clap::Subcommand, Debug)]
enum NoteCommand {
    /// Create a new note.
    Add {
        name: String,
        note: String,
    },
    Replace {
        name: String,
        note: String,
    },
}

impl Args {
    pub fn execute(self, db: &mut data::Leads) -> Result<(), anyhow::Error> {
        match self.command {
            Command::New { position, source } => {
                if self.index.is_some() {
                    return Err(anyhow::anyhow!(
                        "Cannot specify index when creating a new lead"
                    ));
                }
                db.new_lead(self.company, position, source);
            }
            Command::Close { reason } => {
                let lead = db
                    .close_lead(&self.company, self.index, reason)
                    .context("Failed to remove lead")?;
                let db_archive_path = PathBuf::from(&self.archive);
                let mut db_archive = data::Leads::from_path(&db_archive_path)
                    .context("Failed to load or create archive")?;
                db_archive.push_lead(self.company, lead);
                serde_yaml::to_writer(std::fs::File::create(&db_archive_path)?, &db_archive)
                    .context("Failed to write archive")?;
            }
            Command::Note(NoteCommand::Add { name, note }) => {
                let lead = db
                    .get_mut(&self.company, self.index)
                    .context("Failed to get lead")?;
                lead.add_note(name, note);
            }

            Command::Note(NoteCommand::Replace { .. }) => {
                unimplemented!()
            }

            Command::ToDo { .. } => {
                unimplemented!()
            }
            Command::PreInterview { .. } => {
                unimplemented!()
            }
            Command::PostInterview { .. } => {
                unimplemented!()
            }
            Command::RedFlag { .. } => {
                unimplemented!()
            }
            Command::Status { .. } => {
                unimplemented!()
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    // Load db
    let db_path = PathBuf::from(&args.file);
    let mut db = data::Leads::from_path(&db_path)?;
    args.execute(&mut db)?;

    // Write back to disk.
    serde_yaml::to_writer(std::fs::File::create(&db_path)?, &db)?;
    Ok(())
}
