use reqwest::Url;
use serde::Serialize;

use crate::actions::{Action, ActionDescription, ActionState, Actionable};

use crate::actions::base::{CreateFile, CreateFileError};

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct PlaceChannelConfiguration {
    channels: Vec<(String, Url)>,
    create_file: CreateFile,
    action_state: ActionState,
}

impl PlaceChannelConfiguration {
    #[tracing::instrument(skip_all)]
    pub async fn plan(
        channels: Vec<(String, Url)>,
        force: bool,
    ) -> Result<Self, PlaceChannelConfigurationError> {
        let buf = channels
            .iter()
            .map(|(name, url)| format!("{} {}", url, name))
            .collect::<Vec<_>>()
            .join("\n");
        let create_file = CreateFile::plan(
            dirs::home_dir()
                .ok_or(PlaceChannelConfigurationError::NoRootHome)?
                .join(".nix-channels"),
            None,
            None,
            0o0664,
            buf,
            force,
        )
        .await?;
        Ok(Self {
            create_file,
            channels,
            action_state: ActionState::Uncompleted,
        })
    }
}

#[async_trait::async_trait]
impl Actionable for PlaceChannelConfiguration {
    type Error = PlaceChannelConfigurationError;

    fn describe_execute(&self) -> Vec<ActionDescription> {
        let Self {
            channels: _,
            create_file,
            action_state: _,
        } = self;
        if self.action_state == ActionState::Completed {
            vec![]
        } else {
            vec![ActionDescription::new(
                format!(
                    "Place channel configuration at `{}`",
                    create_file.path.display()
                ),
                vec![],
            )]
        }
    }

    #[tracing::instrument(skip_all, fields(
        channels = self.channels.iter().map(|(c, u)| format!("{c}={u}")).collect::<Vec<_>>().join(", "),
    ))]
    async fn execute(&mut self) -> Result<(), Self::Error> {
        let Self {
            create_file,
            channels: _,
            action_state,
        } = self;
        if *action_state == ActionState::Completed {
            tracing::trace!("Already completed: Placing channel configuration");
            return Ok(());
        }
        *action_state = ActionState::Progress;
        tracing::debug!("Placing channel configuration");

        create_file.execute().await?;

        tracing::trace!("Placed channel configuration");
        *action_state = ActionState::Completed;
        Ok(())
    }

    fn describe_revert(&self) -> Vec<ActionDescription> {
        let Self {
            channels: _,
            create_file,
            action_state: _,
        } = self;
        if self.action_state == ActionState::Uncompleted {
            vec![]
        } else {
            vec![ActionDescription::new(
                format!(
                    "Remove channel configuration at `{}`",
                    create_file.path.display()
                ),
                vec![],
            )]
        }
    }

    #[tracing::instrument(skip_all, fields(
        channels = self.channels.iter().map(|(c, u)| format!("{c}={u}")).collect::<Vec<_>>().join(", "),
    ))]
    async fn revert(&mut self) -> Result<(), Self::Error> {
        let Self {
            create_file,
            channels: _,
            action_state,
        } = self;
        if *action_state == ActionState::Uncompleted {
            tracing::trace!("Already reverted: Removing channel configuration");
            return Ok(());
        }
        *action_state = ActionState::Progress;
        tracing::debug!("Removing channel configuration");

        create_file.revert().await?;

        tracing::debug!("Removed channel configuration");
        *action_state = ActionState::Uncompleted;
        Ok(())
    }
}

impl From<PlaceChannelConfiguration> for Action {
    fn from(v: PlaceChannelConfiguration) -> Self {
        Action::PlaceChannelConfiguration(v)
    }
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum PlaceChannelConfigurationError {
    #[error("Creating file")]
    CreateFile(
        #[source]
        #[from]
        CreateFileError,
    ),
    #[error("No root home found to place channel configuration in")]
    NoRootHome,
}
