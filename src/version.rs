use std::cmp::Ordering;
use std::fmt;

#[derive(Hash, Clone, Copy)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub build: u32,
    pub revision: u32,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32, build: Option<u32>, revision: Option<u32>) -> Self {
        let build_value = build.unwrap_or(0);
        let revision_value = revision.unwrap_or(0);
        Version {
            major,
            minor,
            patch,
            build: build_value,
            revision: revision_value,
        }
    }

    pub fn any(&self) -> bool {
        *self == Version::new(0, 0, 0, None, None)
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major
            && self.minor == other.minor
            && self.patch == other.patch
            && self.build == other.build
            && self.revision == other.revision
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.major != other.major {
            Some(self.major.cmp(&other.major))
        } else if self.minor != other.minor {
            Some(self.minor.cmp(&other.minor))
        } else if self.patch != other.patch {
            Some(self.patch.cmp(&other.patch))
        } else if self.build != other.build {
            Some(self.build.cmp(&other.build))
        } else {
            Some(self.revision.cmp(&other.revision))
        }
    }
}

impl Eq for Version {}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.any() {
            write!(f, "any")?;
            return Ok(());
        }
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if self.build > 0 {
            write!(f, ".{}", self.build)?;
        }
        if self.revision > 0 {
            write!(f, "-{}", self.revision)?;
        }
        Ok(())
    }
}

pub fn parse_version_string(version_string: &str) -> Result<Version,&str> {
    let mut parts: Vec<String> = vec![];
    let mut part = String::new();

    for c in version_string.chars() {
        if c == '.' {
            parts.push(part.clone());
            part.clear();
        } else if c == '-' {
            parts.push(part.clone());
            part.clear();
            for _ in parts.len()..4 {
                parts.push("0".to_string());
            }
        } else {
            part.push(c);
        }
    }
    parts.push(part.clone());

    if parts.len() > 5 {
        return Err("Invalid version string");
    }

    let major = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    let build = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
    let revision = parts.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);

    Ok(Version::new(major, minor, patch, Some(build), Some(revision)))
}
