use semver::Version;
use std::{
    error,
    fmt::Display,
    path::{Path, PathBuf},
};
use steamlocate::SteamDir;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum GameType {
    UnityMono,
    UnityIL2CPP,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct Game {
    pub name: String,
    pub arch: String,
    pub path: PathBuf,
    pub ty: Option<GameType>,
    pub bepinex_version: Option<String>,
}

impl Game {
    pub fn set_bix(&mut self, bix: Option<String>) -> Game {
        self.bepinex_version = bix;
        self.to_owned()
    }

    pub fn set_arch(&mut self, arch: String) {
        self.arch = arch;
    }

    pub fn set_ty(&mut self, ty: Option<GameType>) {
        self.ty = ty;
    }

    pub fn get_installed_bepinex_version(&self) -> Option<String> {
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
    let mut unity_games: Vec<Game> = Vec::new();

    let mut steamapps = SteamDir::locate().unwrap_or_default();
    let apps = steamapps.apps();

    apps.iter().for_each(|(_id, app)| match app {
        Some(app) => {
            let path = Path::new(&app.path);
            if path.join("UnityPlayer.dll").exists() {
                let mut game = Game {
                    name: app.name.clone().unwrap_or_default(),
                    arch: "a".to_owned(),
                    path: app.path.to_owned(),
                    bepinex_version: None,
                    ty: None,
                };
                let bix_ver = game.get_installed_bepinex_version();
                game.set_bix(bix_ver);

                unity_games.push(game);
            }
        }
        None => {}
    });
    Ok(unity_games)
}

pub fn get_dll_version(path: PathBuf) -> Result<String, Box<dyn error::Error>> {
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

    println!("{:?}", Version::parse(version));
    Ok(version.to_owned())
}
