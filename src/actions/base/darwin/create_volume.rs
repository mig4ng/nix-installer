use std::path::{Path, PathBuf};

use serde::Serialize;
use tokio::process::Command;

use crate::execute_command;

use crate::actions::{Action, ActionDescription, ActionState, Actionable};

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct CreateVolume {
    disk: PathBuf,
    name: String,
    case_sensitive: bool,
    action_state: ActionState,
}

impl CreateVolume {
    #[tracing::instrument(skip_all)]
    pub async fn plan(
        disk: impl AsRef<Path>,
        name: String,
        case_sensitive: bool,
    ) -> Result<Self, CreateVolumeError> {
        Ok(Self {
            disk: disk.as_ref().to_path_buf(),
            name,
            case_sensitive,
            action_state: ActionState::Uncompleted,
        })
    }
}

#[async_trait::async_trait]
impl Actionable for CreateVolume {
    type Error = CreateVolumeError;

    fn describe_execute(&self) -> Vec<ActionDescription> {
        if self.action_state == ActionState::Completed {
            vec![]
        } else {
            vec![ActionDescription::new(
                format!(
                    "Create a volumne on `{}` named `{}`",
                    self.disk.display(),
                    self.name
                ),
                vec![],
            )]
        }
    }

    #[tracing::instrument(skip_all, fields(
        disk = %self.disk.display(),
        name = %self.name,
        case_sensitive = %self.case_sensitive,
    ))]
    async fn execute(&mut self) -> Result<(), Self::Error> {
        let Self {
            disk,
            name,
            case_sensitive,
            action_state,
        } = self;
        if *action_state == ActionState::Completed {
            tracing::trace!("Already completed: Creating volume");
            return Ok(());
        }
        tracing::debug!("Creating volume");

        execute_command(Command::new("/usr/sbin/diskutil").args([
            "apfs",
            "addVolume",
            &format!("{}", disk.display()),
            if !*case_sensitive {
                "APFS"
            } else {
                "Case-sensitive APFS"
            },
            name,
            "-nomount",
        ]))
        .await
        .map_err(Self::Error::Command)?;

        tracing::trace!("Created volume");
        *action_state = ActionState::Completed;
        Ok(())
    }

    fn describe_revert(&self) -> Vec<ActionDescription> {
        if self.action_state == ActionState::Uncompleted {
            vec![]
        } else {
            vec![ActionDescription::new(
                format!(
                    "Remove the volume on `{}` named `{}`",
                    self.disk.display(),
                    self.name
                ),
                vec![],
            )]
        }
    }

    #[tracing::instrument(skip_all, fields(
        disk = %self.disk.display(),
        name = %self.name,
        case_sensitive = %self.case_sensitive,
    ))]
    async fn revert(&mut self) -> Result<(), Self::Error> {
        let Self {
            disk,
            name,
            case_sensitive: _,
            action_state,
        } = self;
        if *action_state == ActionState::Uncompleted {
            tracing::trace!("Already reverted: Deleting volume");
            return Ok(());
        }
        tracing::debug!("Deleting volume");

        execute_command(Command::new("/usr/sbin/diskutil").args(["apfs", "deleteVolume", name]))
            .await
            .map_err(Self::Error::Command)?;

        tracing::trace!("Deleted volume");
        *action_state = ActionState::Completed;
        Ok(())
    }
}

impl From<CreateVolume> for Action {
    fn from(v: CreateVolume) -> Self {
        Action::DarwinCreateVolume(v)
    }
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum CreateVolumeError {
    #[error("Failed to execute command")]
    Command(
        #[source]
        #[serde(serialize_with = "crate::serialize_error_to_display")]
        std::io::Error,
    ),
}