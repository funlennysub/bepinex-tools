use semver::Version;
use std::{
    error,
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use steamlocate::SteamDir;

use crate::error::Error;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum GameType {
    UnityMono,
    UnityIL2CPP,
}

impl ToString for GameType {
    fn to_string(&self) -> String {
        match self {
            Self::UnityMono => "UnityMono".into(),
            Self::UnityIL2CPP => "UnityIL2CPP".into(),
        }
    }
}

impl FromStr for GameType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "UnityMono" => Ok(Self::UnityMono),
            "Mono" => Ok(Self::UnityMono),
            "UnityIL2CPP" => Ok(Self::UnityIL2CPP),
            "IL2CPP" => Ok(Self::UnityIL2CPP),
            _ => Err(Error::invalid_game_type()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct Game {
    pub name: String,
    pub arch: String,
    pub path: PathBuf,
    pub ty: Option<GameType>,
    pub bepinex_version: Option<Version>,
}

impl Game {
    pub fn set_bix(&mut self, bix: Option<Version>) {
        self.bepinex_version = bix;
    }

    pub fn set_arch(&mut self, arch: String) {
        self.arch = arch;
    }

    pub fn set_ty(&mut self, ty: Option<GameType>) {
        self.ty = ty;
    }

    pub fn get_installed_bepinex_version(&self) -> Option<Version> {
        let core_path = self.path.join("BepInEx").join("core");
        match core_path.exists() {
            true => {
                if core_path.join("BepInEx.Core.dll").exists() {
                    return get_dll_version(core_path.join("BepInEx.Core.dll")).ok();
                } else if core_path.join("BepInEx.dll").exists() {
                    return get_dll_version(core_path.join("BepInEx.dll")).ok();
                }
                None
            }
            false => None,
        }
    }

    pub fn get_game_type(&self) -> Option<GameType> {
        let mono = "Managed";
        let il2cpp = "il2cpp_data";

        match fs::read_dir(&self.path) {
            Ok(dir) => {
                let data_dir = dir.filter_map(Result::ok).find(|el| {
                    el.file_name().to_str().unwrap().ends_with("_Data")
                        && el.file_type().unwrap().is_dir()
                });

                let data_dir = data_dir.as_ref()?.path();
                if data_dir.join(mono).exists() {
                    Some(GameType::UnityMono)
                } else if data_dir.join(il2cpp).exists() {
                    Some(GameType::UnityIL2CPP)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self {
            name: "Not selected".to_owned(),
            arch: "x64".to_owned(),
            path: Default::default(),
            ty: None,
            bepinex_version: None,
        }
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub fn get_unity_games() -> Result<Vec<Game>, Box<dyn error::Error>> {
    let mut steamapps = SteamDir::locate().ok_or("Steam not found")?;
    let apps = steamapps.apps();

    let unity_games = apps
        .iter()
        .filter_map(|(_id, app)| app.as_ref())
        .filter_map(|app| {
            let path = Path::new(&app.path);
            match path.join("UnityPlayer.dll").exists() {
                true => {
                    let mut game = Game {
                        name: app.name.clone().unwrap_or_default(),
                        arch: "a".to_owned(),
                        path: app.path.to_owned(),
                        bepinex_version: None,
                        ty: None,
                    };

                    let bix_ver = game.get_installed_bepinex_version();
                    let game_type = game.get_game_type();
                    game.set_bix(bix_ver);
                    game.set_ty(game_type);

                    Some(game)
                }
                false => None,
            }
        })
        .collect::<Vec<_>>();
    Ok(unity_games)
}

pub fn get_dll_version(path: PathBuf) -> Result<Version, Box<dyn error::Error>> {
    let file = pelite::FileMap::open(path.as_path())?;
    let img = pelite::PeFile::from_bytes(file.as_ref())?;
    let resources = img.resources()?;
    let version_info = resources.version_info()?;

    let lang = version_info
        .translation()
        .get(0)
        .ok_or("Failed to get lang")?;

    let strings = version_info.file_info().strings;
    let string = strings
        .get(lang)
        .ok_or("Failed to get strings for that lang")?;

    let version = string
        .get("ProductVersion")
        .ok_or("Failed to get prod. version")?;

    // TODO: Do some proper handling of invalid semver that bix has in older versions 💀
    Ok(Version::parse(version).unwrap())
}
