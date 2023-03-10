use std::fs::File;
use std::io::{ BufReader, Read };
use tar::Archive;
use flate2::read::GzDecoder;
use std::collections::HashMap;
use indexmap::IndexMap;

mod version;

static EXCLUDE_PATHS: &[&str] = &[
    "lightspd/runsnort.sh",
    "lightspd/manifest.json",
    "lightspd/modules/src/"
];

type AssetsMap = IndexMap<(String, version::Version), Vec<String>>;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 4 {
        eprintln!("Usage: {} <snort-version> <snort-arch> <path/to/Talos_LightSPD.tar.gz>", &args[0]);
        std::process::exit(1);
    }

    let snort_version = version::parse_version_string(&args[1]).unwrap();
    let snort_arch = &args[2];
    let file = File::open(&args[3])?;
    let reader = BufReader::new(file);
    let gz = GzDecoder::new(reader);
    let mut archive = Archive::new(gz);

    if snort_version.any() {
        eprintln!("Error: invalid snort version: {}", &args[1]);
        std::process::exit(2);
    }

    let mut lightspd_version = String::new();

    // There are several subdirs that must always be retained
    let keep_rules: &[&str] = &[ "3.0.0.0" ];
    let keep_modules: &[&str] = &[ "stubs" ];
    let keep_modules_arch: &[&str] = &[ snort_arch ];

    // Collect all the assets in the lightspd package into a versioned map
    let mut map: AssetsMap = IndexMap::new();
    for file in archive.entries().unwrap() {
        let file = file.unwrap();
        let header = file.header();

        if header.entry_type().is_dir() {
            continue;
        }

        let path = file.path()?;
        if EXCLUDE_PATHS.iter().any(|&p| path.starts_with(p)) {
            continue;
        }

        if path.components().count() >= 4 {
            let mut components = path.components();
            let _ = components.next().unwrap().as_os_str().to_str().unwrap();
            let name = components.next().unwrap().as_os_str().to_str().unwrap();
            let ver = components.next().unwrap().as_os_str().to_str().unwrap();
            let arch = components.next().unwrap().as_os_str().to_str().unwrap();

            let version = version::parse_version_string(&ver).unwrap();
            let version: version::Version = match name {
                "rules" => {
                    if keep_rules.contains(&ver) {
                        version::Version::new(0, 0, 0, None, None)
                    } else {
                        version
                    }
                },
                "modules" => {
                    if !keep_modules.contains(&ver)
                        && !keep_modules_arch.contains(&arch) {
                        version::Version::new(999, 999, 999, None, None)
                    } else {
                        version
                    }
                },
                _ => version,
            };

            if version > snort_version {
                continue;
            }

            let key = (name.to_string(), version);
            let value = path.display().to_string();

            map.entry(key)
                .or_insert_with(Vec::new)
                .push(value);
        }

        // Special case, keep the lightspd/version.txt file.
        if path == std::path::PathBuf::from("lightspd/version.txt") {
            let key = ("lightspd/version.txt".to_string(), version::Version::new(0,0,0,None,None));
            let value = path.display().to_string();
            map.entry(key)
                .or_insert_with(Vec::new)
                .push(value);
            let mut file = file;
            file.read_to_string(&mut lightspd_version)?;
        }
    }

    // Identify all the versioned assets which are incompatible with our Snort version.
    let mut asset_versions: HashMap<String, version::Version> = HashMap::new();
    let mut keys_to_remove = Vec::new();
    for (key, _) in &map {
        let (name, version) = key;
        if version.any() {
            continue;
        }
        if let Some(previous_version) = asset_versions.get(name).cloned() {
            if *version > previous_version && *version <= snort_version {
                asset_versions.insert(name.clone(), version.clone());
                let key = (name.clone(), previous_version);
                keys_to_remove.push(key.clone());
            } else {
                let key = (name.clone(), version.clone());
                keys_to_remove.push(key.clone());
            }
        }
        else {
            asset_versions.insert(name.clone(), version.clone());
        }
    }

    // Remove the incompatible assets
    for key in keys_to_remove {
        map.remove(&key);
    }

    // Generate the manifest from remaining asset map
    for (_, value) in &map {
        for path in value {
            println!("{}", path);
        }
    }

    // Print summary to stderr
    eprintln!("LightSPD {}", lightspd_version);
    eprintln!("Snort {} ", snort_version);
    for ((name, version), _) in &map {
        if version.any() { continue; }
        eprintln!(" {} {}", name, version);
    }

    Ok(())
}
