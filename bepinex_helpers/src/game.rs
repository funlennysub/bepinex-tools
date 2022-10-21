use semver::Version;
use std::{
    error,
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};
use steamlocate::SteamDir;

#[macro_export]
macro_rules! game_type {
    ($ty:expr) => {
        $ty.as_ref()
            .unwrap()
            .to_string()
            .split('.')
            .collect::<Vec<_>>()
            .join("")
    };
}

#[derive(Debug)]
pub enum GameArch {
    X64,
    X86,
}

impl Display for GameArch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameArch::X64 => write!(f, "x64"),
            GameArch::X86 => write!(f, "x86"),
        }
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum GameType {
    UnityMono,
    UnityIL2CPP,
}

impl Display for GameType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameType::UnityMono => write!(f, "Unity.Mono"),
            GameType::UnityIL2CPP => write!(f, "Unity.IL2CPP"),
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
    pub fn set_bie(&mut self, bie: Option<Version>) {
        self.bepinex_version = bie;
    }

    pub fn set_arch(&mut self, arch: GameArch) {
        self.arch = arch.to_string();
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

    pub fn get_game_arch(&self) -> GameArch {
        let path = &self.path.join(format!("{}.exe", &self.name));
        fs::read(path)
            .map(|bytes| {
                let start =
                    i32::from_le_bytes(bytes[60..64].try_into().unwrap_or_default()) as usize;
                let machine_type =
                    u16::from_le_bytes(bytes[start + 4..start + 6].try_into().unwrap_or_default());
                match machine_type {
                    34404 => GameArch::X64,
                    _ => GameArch::X86,
                }
            })
            .unwrap_or_else(|_| GameArch::X64)
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
                        arch: "x64".to_owned(),
                        path: app.path.to_owned(),
                        bepinex_version: None,
                        ty: None,
                    };

                    let bie_ver = game.get_installed_bepinex_version();
                    let game_type = game.get_game_type();
                    let game_arch = game.get_game_arch();
                    game.set_bie(bie_ver);
                    game.set_ty(game_type);
                    game.set_arch(game_arch);

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

    // "Converts" 5.*.*.* into 5.*.* becuase BepInEx devs decided to add build num 💀
    if version.starts_with('5') && version.split('.').count() > 3 {
        let ver = version.split('.').into_iter().collect::<Vec<&str>>()[0..3].join(".");
        return Ok(Version::parse(&ver).unwrap());
    }

    // TODO: Do some proper handling of invalid semver that bie has in older versions 💀
    Ok(Version::parse(version).unwrap())
}
