//! The main menu's updater. Only when the player presses its button does it ask for the
//! latest release (PersistentStorage/Game/Updater.toml says where: GitHub's API, or a
//! server of anyone's own that answers the same way). A release whose tag is not this
//! program's version is offered; pressed again, the files of `download_list` are
//! downloaded, each checked against the SHA-256 the release gives for it, and put over
//! the program's own. The new version runs from the next start. PersistentStorage is
//! never written over.
//!
//! The network work runs on a thread of its own; the menu only reads its state.
//! `--update` does the same from the command line, asking before it downloads.

use anyhow::{bail, Context as _, Result};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const CONFIG_FILE: &str = "PersistentStorage/Game/Updater.toml";
/// GitHub's API answers only requests that say who they are.
const USER_AGENT: &str = "AdvancedServer-updater";
/// How a release gives a file's checksum ("digest" of GitHub's assets).
const SHA256_PREFIX: &str = "sha256:";
/// The download's progress is shown in whole percent.
const PERCENT: u64 = 100;
/// Files an update replaced keep this suffix until the next start removes them (the
/// running program and Resources.tar cannot be removed while they are in use).
const OLD_SUFFIX: &str = ".old";
const NEW_SUFFIX: &str = ".new";

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct UpdaterConfig {
    /// Where the latest release is described, in the form of GitHub's API.
    pub latest_release_url: String,
    /// The files of the release this copy of the program is made of.
    pub download_list: Vec<String>,
}

impl Default for UpdaterConfig {
    fn default() -> UpdaterConfig {
        let download_list: &[&str] = if cfg!(windows) {
            &["AdvancedServer.exe", "Resources.tar"]
        } else {
            &["AdvancedServer", "Resources.tar"]
        };
        UpdaterConfig {
            latest_release_url: "https://api.github.com/repos/thearcticfox25/sonic.exe-mp/releases/latest".to_string(),
            download_list: download_list.iter().map(|name| name.to_string()).collect(),
        }
    }
}

impl UpdaterConfig {
    /// The settings, written back so a new file shows every one of them.
    fn load() -> UpdaterConfig {
        let config = std::fs::read_to_string(CONFIG_FILE)
            .ok()
            .and_then(|text| toml::from_str(&text).inspect_err(|error| eprintln!("{CONFIG_FILE} is broken, using the defaults: {error}")).ok())
            .unwrap_or_default();
        let written = Path::new(CONFIG_FILE)
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|_| std::fs::write(CONFIG_FILE, toml::to_string(&config).expect("updater settings always serialize")));
        if let Err(error) = written {
            eprintln!("failed to save {CONFIG_FILE}: {error}");
        }
        config
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum UpdateState {
    /// Nothing asked yet.
    Idle,
    Checking,
    UpToDate,
    Available(Release),
    Downloading { percent: u64 },
    /// Installed; it runs from the next start.
    Ready,
    Failed,
}

pub struct Updater {
    config: UpdaterConfig,
    state: Arc<Mutex<UpdateState>>,
}

impl Updater {
    /// Removes what the last update left behind. Nothing is asked of the network yet.
    pub fn start() -> Updater {
        remove_replaced_files(&program_folder());
        Updater { config: UpdaterConfig::load(), state: Arc::new(Mutex::new(UpdateState::Idle)) }
    }

    pub fn state(&self) -> UpdateState {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).clone()
    }

    /// What the menu button says.
    pub fn label(&self) -> String {
        match self.state() {
            UpdateState::Idle => "check for updates".to_string(),
            UpdateState::Checking => "checking".to_string(),
            UpdateState::UpToDate => "up to date".to_string(),
            UpdateState::Available(release) => format!("update to {}", release.tag_name),
            UpdateState::Downloading { percent } => format!("downloading {percent}%"),
            UpdateState::Ready => "restart to update".to_string(),
            UpdateState::Failed => "update failed".to_string(),
        }
    }

    /// The menu button: looks for a release, or downloads the one on offer.
    pub fn press(&self) {
        match self.state() {
            UpdateState::Idle | UpdateState::UpToDate | UpdateState::Failed => self.check(),
            UpdateState::Available(release) => self.download(release),
            UpdateState::Checking | UpdateState::Downloading { .. } | UpdateState::Ready => {}
        }
    }

    fn check(&self) {
        let (url, state) = (self.config.latest_release_url.clone(), Arc::clone(&self.state));
        set(&state, UpdateState::Checking);
        std::thread::spawn(move || {
            let found = match latest_release(&url) {
                Ok(release) if is_this_version(&release.tag_name) => UpdateState::UpToDate,
                Ok(release) => UpdateState::Available(release),
                Err(error) => {
                    eprintln!("looking for updates failed: {error:#}");
                    UpdateState::Failed
                }
            };
            set(&state, found);
        });
    }

    fn download(&self, release: Release) {
        let (files, state) = (self.config.download_list.clone(), Arc::clone(&self.state));
        set(&state, UpdateState::Downloading { percent: 0 });
        std::thread::spawn(move || {
            let installed = download_and_install(&release, &files, |percent| set(&state, UpdateState::Downloading { percent }));
            if let Err(error) = &installed {
                eprintln!("updating failed: {error:#}");
            }
            set(&state, if installed.is_ok() { UpdateState::Ready } else { UpdateState::Failed });
        });
    }
}

/// --update: the latest release, if it is not this program, downloaded and put in
/// place once the person at the keyboard agrees.
pub fn update_from_command_line() {
    remove_replaced_files(&program_folder());
    let config = UpdaterConfig::load();
    println!("asking {}", config.latest_release_url);
    let release = match latest_release(&config.latest_release_url) {
        Ok(release) => release,
        Err(error) => return println!("looking for updates failed: {error:#}"),
    };
    if is_this_version(&release.tag_name) {
        return println!("this is the latest release, {}", release.tag_name);
    }
    let missing: Vec<&str> = config.download_list.iter().filter(|name| !release.assets.iter().any(|asset| &asset.name == *name)).map(String::as_str).collect();
    if !missing.is_empty() {
        return println!("{} is out, but it has no {}: nothing to update from", release.tag_name, missing.join(", "));
    }
    let size: u64 = release.assets.iter().filter(|asset| config.download_list.contains(&asset.name)).map(|asset| asset.size).sum();
    print!("{} is out (this is v{}). Download {} ({} MiB)? [y/N] ", release.tag_name, crate::packet::VERSION, config.download_list.join(", "), size / (1024 * 1024));
    let _ = std::io::Write::flush(&mut std::io::stdout());
    let mut answer = String::new();
    let _ = std::io::stdin().read_line(&mut answer);
    if !answer.trim().eq_ignore_ascii_case("y") {
        return println!("nothing downloaded");
    }
    let installed = download_and_install(&release, &config.download_list, |percent| {
        print!("\rdownloading {percent}%");
        let _ = std::io::Write::flush(&mut std::io::stdout());
    });
    println!();
    match installed {
        Ok(()) => println!("updated to {}; it runs from the next start", release.tag_name),
        Err(error) => println!("updating failed: {error:#}"),
    }
}

fn set(state: &Mutex<UpdateState>, value: UpdateState) {
    *state.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = value;
}

/// The folder the program runs from, which an update writes into.
fn program_folder() -> PathBuf {
    std::env::current_exe().ok().and_then(|program| program.parent().map(Path::to_path_buf)).unwrap_or_else(|| PathBuf::from("."))
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Asset {
    name: String,
    size: u64,
    browser_download_url: String,
    /// "sha256:<hex>"; releases made before GitHub gave checksums have none.
    digest: Option<String>,
}

fn latest_release(url: &str) -> Result<Release> {
    let text = ureq::get(url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github+json")
        .call()
        .with_context(|| format!("asking {url}"))?
        .body_mut()
        .read_to_string()
        .context("reading the latest release")?;
    serde_json::from_str(&text).context("reading the latest release")
}

/// A release is this program's when its tag is this program's version ("vα" or "α").
fn is_this_version(tag: &str) -> bool {
    tag.trim_start_matches('v') == crate::packet::VERSION.to_string()
}

/// Downloads every file of the list, and only once all of them arrived whole puts them
/// in place: a failed download leaves the program as it was. `progress` hears the
/// whole download's percent.
fn download_and_install(release: &Release, files: &[String], mut progress: impl FnMut(u64)) -> Result<()> {
    let assets = files
        .iter()
        .map(|name| release.assets.iter().find(|asset| &asset.name == name).with_context(|| format!("release {} has no {name}", release.tag_name)))
        .collect::<Result<Vec<_>>>()?;
    let total: u64 = assets.iter().map(|asset| asset.size).sum::<u64>().max(1);
    let folder = program_folder();
    let mut received = 0;
    for asset in &assets {
        let fresh = with_suffix(&folder.join(&asset.name), NEW_SUFFIX);
        download(asset, &fresh, |bytes| {
            received += bytes;
            progress((received * PERCENT / total).min(PERCENT));
        })?;
    }
    for asset in &assets {
        put_in_place(&folder.join(&asset.name))?;
    }
    Ok(())
}

/// Saves one file of the release beside its old self, and checks it against the
/// release's SHA-256. `progress` hears of every piece that arrives.
fn download(asset: &Asset, to: &Path, mut progress: impl FnMut(u64)) -> Result<()> {
    let Some(expected) = asset.digest.as_deref().and_then(|digest| digest.strip_prefix(SHA256_PREFIX)) else {
        bail!("{} has no SHA-256 to check it against", asset.name);
    };
    let url = &asset.browser_download_url;
    let mut response = ureq::get(url).header("User-Agent", USER_AGENT).call().with_context(|| format!("downloading {url}"))?;
    let mut body = response.body_mut().with_config().limit(u64::MAX).reader();
    let mut file = std::fs::File::create(to).with_context(|| format!("creating {}", to.display()))?;
    let mut hash = ring::digest::Context::new(&ring::digest::SHA256);
    let mut buffer = vec![0; 64 * 1024];
    loop {
        let read = body.read(&mut buffer).with_context(|| format!("downloading {}", asset.name))?;
        if read == 0 {
            break;
        }
        std::io::Write::write_all(&mut file, &buffer[..read]).with_context(|| format!("saving {}", to.display()))?;
        hash.update(&buffer[..read]);
        progress(read as u64);
    }
    let found = hex(hash.finish().as_ref());
    if !found.eq_ignore_ascii_case(expected) {
        let _ = std::fs::remove_file(to);
        bail!("{} is not what the release says (SHA-256 {found}, expected {expected})", asset.name);
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// The downloaded `target.new` takes the place of `target`, which is kept as .old:
/// a file in use (this program, Resources.tar) can be renamed but not always replaced.
/// The new file keeps the old one's permissions (the executable bit on Linux).
fn put_in_place(target: &Path) -> Result<()> {
    let fresh = with_suffix(target, NEW_SUFFIX);
    if let Ok(metadata) = std::fs::metadata(target) {
        std::fs::set_permissions(&fresh, metadata.permissions()).with_context(|| format!("giving {} its permissions", fresh.display()))?;
        let old = with_suffix(target, OLD_SUFFIX);
        let _ = std::fs::remove_file(&old);
        std::fs::rename(target, &old).with_context(|| format!("moving {} aside", target.display()))?;
    }
    std::fs::rename(&fresh, target).with_context(|| format!("putting {} in place", target.display()))
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

/// The .old files an update left beside the files that replaced them.
fn remove_replaced_files(folder: &Path) {
    let Ok(entries) = std::fs::read_dir(folder) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.to_string_lossy();
        let Some(replaced) = name.strip_suffix(OLD_SUFFIX) else { continue };
        if Path::new(replaced).exists() {
            let _ = std::fs::remove_file(&path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_release_tagged_with_this_version_is_this_program() {
        let own = crate::packet::VERSION;
        assert!(is_this_version(&format!("v{own}")));
        assert!(is_this_version(&own.to_string()));
        assert!(!is_this_version("v1101\u{3c5}"), "the C version's last release");
        assert!(!is_this_version("fusiontest"));
    }

    #[test]
    fn a_new_file_takes_the_place_of_the_old_which_waits_beside_it() {
        let folder = std::env::temp_dir().join(format!("advancedserver-update-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).unwrap();
        let target = folder.join("Resources.tar");
        std::fs::write(&target, "old").unwrap();
        std::fs::write(with_suffix(&target, NEW_SUFFIX), "new").unwrap();

        put_in_place(&target).unwrap();
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "new");
        assert_eq!(std::fs::read_to_string(with_suffix(&target, OLD_SUFFIX)).unwrap(), "old");
        remove_replaced_files(&folder);
        assert!(!with_suffix(&target, OLD_SUFFIX).exists(), "gone at the next start");
        std::fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn sha256_is_written_as_github_writes_it() {
        let digest = ring::digest::digest(&ring::digest::SHA256, b"abc");
        assert_eq!(hex(digest.as_ref()), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }
}
