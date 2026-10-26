use std::{env, fs, io, path::Path, process::Command};

/// The game's resources in the source tree, and the one file a release build packs
/// them into (core/resources reads either).
const SOURCE_FOLDER: &str = "Resources";
const ARCHIVE: &str = "Resources.tar";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // A directory: cargo reruns this when any file inside it changes.
    println!("cargo:rerun-if-changed={SOURCE_FOLDER}");

    // The game looks for its resources in the folder it runs from, so the built binary gets
    // them beside it. OUT_DIR is target/<profile>/build/<crate>/out. A debug build gets a
    // copy of the Resources folder, a release build one Resources.tar (core/resources reads
    // either): one open file instead of a file system lookup for every resource.
    let out_dir = env::var("OUT_DIR").expect("cargo sets OUT_DIR");
    if let Some(profile_dir) = Path::new(&out_dir).ancestors().nth(3) {
        if env::var("PROFILE").as_deref() == Ok("release") {
            pack(Path::new(SOURCE_FOLDER), &profile_dir.join(ARCHIVE)).expect("packing Resources into Resources.tar");
            // A copy left from an older build would only confuse whoever looks there.
            let old_copy = profile_dir.join(SOURCE_FOLDER);
            if old_copy.is_dir() {
                fs::remove_dir_all(old_copy).expect("removing the old Resources copy");
            }
        } else {
            sync(Path::new(SOURCE_FOLDER), &profile_dir.join(SOURCE_FOLDER)).expect("copying Resources beside the binary");
        }
    }

    // The icon Windows shows for the program's file; the window's own is set when the
    // game starts, on every system.
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_windows_icon(Path::new(&out_dir)).expect("putting the icon into the program");
    }

    // Build metadata for the startup banner
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "unknown".into());
    println!("cargo:rustc-env=BUILD_TARGET_OS={target_os}");
    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    // "rustc 1.95.0 (hash date)" -> "rustc 1.95.0"
    let rustc_version = output_of(&rustc, "--version").split(' ').take(2).collect::<Vec<_>>().join(" ");
    println!("cargo:rustc-env=BUILD_RUSTC_VERSION={rustc_version}");
    println!("cargo:rustc-env=BUILD_DATE={}", output_of("date", "+%b %d %Y"));
    println!("cargo:rustc-env=BUILD_TIME={}", output_of("date", "+%H:%M:%S"));
}

/// What `program argument` prints, trimmed, or "unknown" when it cannot run.
fn output_of(program: &str, argument: &str) -> String {
    Command::new(program)
        .arg(argument)
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|text| text.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

/// Textures/icon.png, the window's icon, made the program's icon: an .ico may hold a
/// PNG as it is, so the picture is wrapped, not converted.
fn embed_windows_icon(out_dir: &Path) -> io::Result<()> {
    let png = fs::read(Path::new(SOURCE_FOLDER).join("Textures").join("icon.png"))?;
    // The width and height of a PNG are the first two numbers of its IHDR chunk.
    let size = |at: usize| u32::from_be_bytes([png[at], png[at + 1], png[at + 2], png[at + 3]]);
    let (width, height) = (size(16), size(20));
    // An .ico writes 256 as 0.
    let side = |pixels: u32| if pixels >= 256 { 0 } else { pixels as u8 };
    const HEADER_AND_ONE_ENTRY: u32 = 6 + 16;
    // The header: reserved, 1 for an icon, one picture.
    let mut ico = vec![0, 0, 1, 0, 1, 0];
    // The picture's entry: its size, no palette, reserved, one plane, 32 bits a pixel,
    // then how long it is and where it starts.
    ico.extend([side(width), side(height), 0, 0, 1, 0, 32, 0]);
    ico.extend((png.len() as u32).to_le_bytes());
    ico.extend(HEADER_AND_ONE_ENTRY.to_le_bytes());
    ico.extend(&png);
    let ico_path = out_dir.join("icon.ico");
    fs::write(&ico_path, ico)?;
    let rc_path = out_dir.join("icon.rc");
    let quoted = ico_path.display().to_string().replace('\\', "/");
    fs::write(&rc_path, format!("1 ICON \"{quoted}\"\n"))?;
    embed_resource::compile(&rc_path, embed_resource::NONE).manifest_optional().map_err(|error| io::Error::other(error.to_string()))
}

/// Makes `to` a copy of `from`: new and changed files are copied, files gone from
/// `from` are removed. Unchanged files (same size, not newer) are left alone, so only
/// the first build copies everything.
fn sync(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let (source, target) = (entry.path(), to.join(entry.file_name()));
        if entry.file_type()?.is_dir() {
            sync(&source, &target)?;
            continue;
        }
        let meta = entry.metadata()?;
        let unchanged = fs::metadata(&target).is_ok_and(|copied| copied.len() == meta.len() && copied.modified().ok() >= meta.modified().ok());
        if !unchanged {
            fs::copy(&source, &target)?;
        }
    }
    for entry in fs::read_dir(to)? {
        let entry = entry?;
        if !from.join(entry.file_name()).exists() {
            if entry.file_type()?.is_dir() { fs::remove_dir_all(entry.path())? } else { fs::remove_file(entry.path())? }
        }
    }
    Ok(())
}

/// Writes every file of `from` into the archive `to`, by its path inside `from`. Cargo
/// runs this only when something in `from` changed, and then every file may have, so
/// the archive is written anew; it goes to a temporary name first, so a build stopped
/// halfway never leaves a broken archive for the game to read.
fn pack(from: &Path, to: &Path) -> io::Result<()> {
    let unfinished = to.with_extension("tar.part");
    let mut archive = tar::Builder::new(io::BufWriter::new(fs::File::create(&unfinished)?));
    archive.append_dir_all("", from)?;
    archive.into_inner()?.into_inner()?;
    fs::rename(unfinished, to)
}
