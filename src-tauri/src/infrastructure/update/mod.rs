//! Signed GitHub-primary, Gitee-fallback updater boundary.

use std::time::Duration;

use reqwest::Client;
use semver::Version;
use serde::Deserialize;

use crate::domain::AppError;

const GITEE_RELEASES_API: &str =
    "https://gitee.com/api/v5/repos/milesxue/pixel-done-windows/releases";
const GITEE_MANIFEST_NAME: &str = "latest-gitee.json";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GiteeUpdateManifest {
    pub version: Version,
    pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
struct GiteeRelease {
    id: u64,
    tag_name: String,
    #[serde(default)]
    prerelease: bool,
}

#[derive(Clone, Debug, Deserialize)]
struct GiteeAttachment {
    name: String,
    browser_download_url: String,
}

pub async fn resolve_gitee_manifest(
    current_version: &Version,
    requested_version: Option<&Version>,
) -> Result<Option<GiteeUpdateManifest>, AppError> {
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(15))
        .user_agent("PixelDone-Windows-Updater")
        .build()
        .map_err(|error| AppError::Update(error.to_string()))?;
    let releases = client
        .get(GITEE_RELEASES_API)
        .query(&[("direction", "desc"), ("per_page", "100"), ("page", "1")])
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|error| AppError::Update(error.to_string()))?
        .json::<Vec<GiteeRelease>>()
        .await
        .map_err(|error| AppError::Update(error.to_string()))?;

    let Some((release, version)) = select_release(&releases, current_version, requested_version)
    else {
        return Ok(None);
    };
    let attachment_url = format!("{GITEE_RELEASES_API}/{}/attach_files", release.id);
    let attachments = client
        .get(attachment_url)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|error| AppError::Update(error.to_string()))?
        .json::<Vec<GiteeAttachment>>()
        .await
        .map_err(|error| AppError::Update(error.to_string()))?;
    Ok(attachments
        .into_iter()
        .find(|attachment| attachment.name == GITEE_MANIFEST_NAME)
        .map(|attachment| GiteeUpdateManifest {
            version,
            url: attachment.browser_download_url,
        }))
}

fn select_release<'a>(
    releases: &'a [GiteeRelease],
    current_version: &Version,
    requested_version: Option<&Version>,
) -> Option<(&'a GiteeRelease, Version)> {
    releases
        .iter()
        .filter(|release| !release.prerelease)
        .filter_map(|release| {
            let version = parse_strict_release_tag(&release.tag_name)?;
            if version <= *current_version {
                return None;
            }
            if requested_version.is_some_and(|requested| requested != &version) {
                return None;
            }
            Some((release, version))
        })
        .max_by(|left, right| left.1.cmp(&right.1))
}

fn parse_strict_release_tag(tag: &str) -> Option<Version> {
    let version_text = tag.strip_prefix('v')?;
    let version = Version::parse(version_text).ok()?;
    if !version.pre.is_empty() || !version.build.is_empty() || format!("v{version}") != tag {
        return None;
    }
    Some(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_latest_stable_or_exact_requested_release() {
        let releases = vec![
            GiteeRelease {
                id: 1,
                tag_name: "v3.2.7".into(),
                prerelease: false,
            },
            GiteeRelease {
                id: 2,
                tag_name: "v3.2.8-rc.1".into(),
                prerelease: true,
            },
            GiteeRelease {
                id: 3,
                tag_name: "v3.2.6".into(),
                prerelease: false,
            },
        ];
        let current = Version::parse("3.2.6").unwrap();
        assert_eq!(
            select_release(&releases, &current, None)
                .unwrap()
                .1
                .to_string(),
            "3.2.7"
        );
        let requested = Version::parse("3.2.7").unwrap();
        assert_eq!(
            select_release(&releases, &current, Some(&requested))
                .unwrap()
                .0
                .id,
            1
        );
    }

    #[test]
    fn rejects_current_older_and_mismatched_releases() {
        let releases = vec![GiteeRelease {
            id: 1,
            tag_name: "v3.2.7".into(),
            prerelease: false,
        }];
        let current = Version::parse("3.2.7").unwrap();
        assert!(select_release(&releases, &current, None).is_none());
        let requested = Version::parse("3.2.8").unwrap();
        let old_current = Version::parse("3.2.6").unwrap();
        assert!(select_release(&releases, &old_current, Some(&requested)).is_none());
    }

    #[test]
    fn accepts_only_canonical_stable_release_tags() {
        assert_eq!(
            parse_strict_release_tag("v3.2.7").unwrap(),
            Version::parse("3.2.7").unwrap()
        );
        for tag in [
            "3.2.7",
            "vv3.2.7",
            "v3.2",
            "v03.2.7",
            "v3.2.7-rc.1",
            "v3.2.7+build.1",
        ] {
            assert!(parse_strict_release_tag(tag).is_none(), "accepted {tag}");
        }
    }
}
