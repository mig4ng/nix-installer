use serde::Serialize;

use crate::HarmonicError;

use crate::actions::{ActionDescription, Actionable, ActionState, Action};

use super::{CreateFile, CreateFileError, CreateDirectory, CreateDirectoryError};

const NIX_CONF_FOLDER: &str = "/etc/nix";
const NIX_CONF: &str = "/etc/nix/nix.conf";

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct PlaceNixConfiguration {
    create_directory: CreateDirectory,
    create_file: CreateFile,
}

impl PlaceNixConfiguration {
    #[tracing::instrument(skip_all)]
    pub async fn plan(
        nix_build_group_name: String,
        extra_conf: Option<String>,
        force: bool,
    ) -> Result<Self, HarmonicError> {
        let buf = format!(
            "\
            {extra_conf}\n\
            build-users-group = {nix_build_group_name}\n\
        ",
            extra_conf = extra_conf.unwrap_or_else(|| "".into()),
        );
        let create_directory = CreateDirectory::plan(NIX_CONF_FOLDER, "root".into(), "root".into(), 0o0755, force).await?;
        let create_file =
            CreateFile::plan(NIX_CONF, "root".into(), "root".into(), 0o0664, buf, force).await?;
        Ok(Self { create_directory, create_file })
    }
}

#[async_trait::async_trait]
impl Actionable for ActionState<PlaceNixConfiguration> {
    type Error = PlaceNixConfigurationError;

    fn description(&self) -> Vec<ActionDescription> {
        vec![ActionDescription::new(
            format!("Place the nix configuration in `{NIX_CONF}`"),
            vec!["This file is read by the Nix daemon to set its configuration options at runtime.".to_string()],
        )]
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self) -> Result<(), Self::Error> {
        let Self { create_file, create_directory } = self;

        create_directory.execute().await?;
        create_file.execute().await?;

        Ok(())
    }


    #[tracing::instrument(skip_all)]
    async fn revert(&mut self) -> Result<(), Self::Error> {
        todo!();

        Ok(())
    }
}

impl From<ActionState<PlaceNixConfiguration>> for ActionState<Action> {
    fn from(v: ActionState<PlaceNixConfiguration>) -> Self {
        match v {
            ActionState::Completed(_) => ActionState::Completed(Action::PlaceNixConfiguration(v)),
            ActionState::Planned(_) => ActionState::Planned(Action::PlaceNixConfiguration(v)),
            ActionState::Reverted(_) => ActionState::Reverted(Action::PlaceNixConfiguration(v)),
        }
    }
}


#[derive(Debug, thiserror::Error, Serialize)]
pub enum PlaceNixConfigurationError {

}