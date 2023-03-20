use std::collections::HashSet;
use std::collections::HashMap;
use std::fs::File;
use std::io::{ BufReader, Read };
use flate2::read::GzDecoder;
use tar::Archive;

mod version;

static EXCLUDE_PATHS: &[&str] = &[
    "lightspd/runsnort.sh",
    "lightspd/manifest.json",
    "lightspd/modules/src/"
];

type Asset = (String, version::Version, String);

fn remove_matching_elements(vec_a: &mut Vec<Asset>, vec_b: &Vec<(String, version::Version)>) {
    // Create a set of tuples containing the first string and version from vec_b
    let set_b: HashSet<(&String, &version::Version)> = vec_b.iter().map(|(s, v)| (s, v)).collect();

    // Filter out all elements from vec_a whose first string and version match those in set_b
    vec_a.retain(|(s, v, _)| !set_b.contains(&(s, v)));
}


fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 4 {
        eprintln!("Usage: {} <snort-version> <snort-arch> <path/to/Talos_LightSPD.tar.gz>", &args[0]);
        std::process::exit(1);
    }

    let snort_version = version::parse_version_string(&args[1]).unwrap();
    if snort_version.any() {
        eprintln!("Error: invalid snort version: {}", &args[1]);
        std::process::exit(2);
    }


    let snort_arch = &args[2];
    let file = File::open(&args[3])?;
    let reader = BufReader::new(file);
    let gz = GzDecoder::new(reader);
    let mut archive = Archive::new(gz);
    let mut lightspd_version = String::new();

    // There are several subdirs that must always be retained
    //let keep_rules: &[&str] = &[ "3.0.0.0" ];
    let keep_modules: &[&str] = &[ "stubs" ];
    let keep_modules_arch: &[&str] = &[ snort_arch ];


    let mut assets: Vec<Asset> = Vec::new();

    // Collect all the assets in the lightspd package into a versioned map
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
            let _ = components.next();
            let name = components.next().unwrap().as_os_str().to_str().unwrap();
            let ver = components.next().unwrap().as_os_str().to_str().unwrap();
            let arch = components.next().unwrap().as_os_str().to_str().unwrap();

            let version = version::parse_version_string(&ver).unwrap();

            if version > snort_version {
                continue;
            }

            let name = name.to_string();
            let value = path.display().to_string();
            assets.push((name.to_string(), version, value));
        }

        // Special case, keep the lightspd/version.txt file.
        if path == std::path::PathBuf::from("lightspd/version.txt") {
            let name = "lightspd/version.txt".to_string();
            let value = path.display().to_string();
            assets.push((name.to_string(), version::Version::new(0, 0, 0, None, None), value));

            // Store the value of version.txt for reporting later.
            let mut file = file;
            file.read_to_string(&mut lightspd_version)?;
        }
    }

    // Identify all the versioned assets which are incompatible with our Snort version.
    let mut asset_versions: HashMap<String, version::Version> = HashMap::new();
    let mut keys_to_remove = Vec::new();
    for (name, version, _) in &assets {

        if version.any() {
            continue;
        }

        if let Some(previous_version) = asset_versions.get(name).cloned() {
            //let version = *version;
            if name == "modules" {
                if  *version > previous_version && *version <= snort_version {
                    asset_versions.insert(name.clone(), version.clone());
                    let key = (name.clone(), previous_version);
                    keys_to_remove.push(key.clone());
                } else {
                    let key = (name.clone(), version.clone());
                    keys_to_remove.push(key.clone());
                }
            }
        }
        else {
            asset_versions.insert(name.clone(), version.clone());
        }
    }


    remove_matching_elements(&mut assets, &keys_to_remove);

    // Generate the manifest from remaining asset map
    for (_, _, value) in &assets {
        println!("{}", value);
    }

    // Print summary to stderr
    eprintln!("LightSPD {}", lightspd_version);
    eprintln!("Snort {} ", snort_version);
    for (name, version, _) in &assets {
        if version.any() { continue; }
        eprintln!(" {} {}", name, version);
    }

    Ok(())
}
