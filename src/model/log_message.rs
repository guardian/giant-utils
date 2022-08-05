use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum FailureStage {
    UploadData,
    UploadMetadata,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status")]
pub enum LogMessage {
    Success {
        path: PathBuf,
        size: u64,
        start_millis: u128,
        end_millis: u128,
    },
    Failure {
        path: PathBuf,
        size: u64,
        start_millis: u128,
        end_millis: u128,
        failure_stage: FailureStage,
        reason: String,
    },
}

impl LogMessage {
    pub fn to_json(&self) -> String {
        let mut s = serde_json::to_string(self).unwrap();
        s.push('\n');
        s
    }

    pub fn to_tsv_row(&self) -> String {
        match self {
            Self::Failure {
                path,
                size,
                start_millis: start_epoch,
                end_millis: end_epoch,
                failure_stage,
                reason,
            } => {
                let failure_stage = match failure_stage {
                    FailureStage::UploadData => "failed_to_upload_data",
                    FailureStage::UploadMetadata => "failed_to_upload_metadata",
                };

                format!(
                    "failure\t{}\t{}\t{}\t{}\t{}\t{}\n",
                    path.display(),
                    size,
                    start_epoch,
                    end_epoch,
                    failure_stage,
                    reason
                )
            }
            Self::Success {
                path,
                size,
                start_millis: start_epoch,
                end_millis: end_epoch,
            } => {
                format!(
                    "success\t{}\t{}\t{}\t{}\t\n",
                    path.display(),
                    size,
                    start_epoch,
                    end_epoch
                )
            }
        }
    }
}
