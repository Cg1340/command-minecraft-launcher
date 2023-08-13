use crate::downloader::downloader;
use crate::get_path;
use crate::post::Post;
use crate::write_to_file;
use crossterm::cursor;
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType;
use crossterm::ExecutableCommand;
use indicatif::ProgressBar;
use log::Log;
use reqwest::Error;
use reqwest::header::HeaderMap;
use serde_json::Value;
use serde_json::json;
use std::borrow::Borrow;
use std::fs::create_dir_all;
use std::io::stdout;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::thread;

const VERSION_MANIFEST_URL: &str = "https://bmclapi2.bangbang93.com/mc/game/version_manifest.json";
const _DOWNLOAD_THREAD_MAX: usize = 256;

#[derive(Debug)]
pub enum GameVersionType {
    Release,
    Snapshot,
    OldBeta,
    OldAlpha,
}

#[derive(Debug)]
pub struct GameVersion {
    pub version_id: String,
    pub version_type: GameVersionType,
}

pub struct DownloadManager {}

impl GameVersion {
    pub fn build() -> Result<Vec<GameVersion>, Error> {
        let version_manifest = match reqwest::blocking::get(VERSION_MANIFEST_URL) {
            Ok(result) => match result.text() {
                Ok(result) => result,
                Err(err) => return Err(err),
            },
            Err(err) => return Err(err),
        };

        let version_manifest_json: Value = serde_json::from_str(&version_manifest).unwrap();
        // {
        //     "latest": {
        //         "release": "1.19",
        //         "snapshot": "22w24a"
        //     },
        //     "versions": [
        //         {
        //             "id": "22w24a",
        //             "type": "snapshot",
        //             "url": "https://piston-meta.mojang.com/v1/packages/b74d6df246b9b60e39855076ef171aa7071276f7/22w24a.json",
        //             "time": "2022-06-15T16:26:14+00:00",
        //             "releaseTime": "2022-06-15T16:21:49+00:00"
        //         }, ...
        //     ]
        // }
        let mut result = vec![];
        for item in version_manifest_json["versions"].as_array().unwrap() {
            result.push(GameVersion {
                version_id: item["id"].as_str().unwrap().to_string(),
                version_type: match item["type"].as_str().unwrap() {
                    "release" => GameVersionType::Release,
                    "snapshot" => GameVersionType::Snapshot,
                    "old_beta" => GameVersionType::OldBeta,
                    "old_alpha" => GameVersionType::OldAlpha,
                    _ => GameVersionType::Release,
                },
            })
        }

        Ok(result)
    }
}

impl DownloadManager {
    pub fn new() -> DownloadManager {
        DownloadManager {}
    }

    pub fn download_version(&self, version_id: &str, name: &str) -> Result<(), String> {
        let _ = crossterm::terminal::enable_raw_mode();
        let mut stdout = stdout();
        let _ = stdout.execute(Clear(ClearType::All));
        let _ = stdout.execute(cursor::MoveTo(0, 0));
        let _ = crossterm::terminal::disable_raw_mode();

        let version_manifest = match reqwest::blocking::get(VERSION_MANIFEST_URL) {
            Ok(result) => match result.text() {
                Ok(result) => result,
                Err(err) => return Err(err.to_string()),
            },
            Err(err) => return Err(err.to_string()),
        };

        let version_manifest_json: Value = match serde_json::from_str(&version_manifest) {
            Ok(result) => result,
            Err(err) => return Err(err.to_string()),
        };
        // {
        //     "latest": {
        //         "release": "1.19",
        //         "snapshot": "22w24a"
        //     },
        //     "versions": [
        //         {
        //             "id": "22w24a",
        //             "type": "snapshot",
        //             "url": "https://piston-meta.mojang.com/v1/packages/b74d6df246b9b60e39855076ef171aa7071276f7/22w24a.json",
        //             "time": "2022-06-15T16:26:14+00:00",
        //             "releaseTime": "2022-06-15T16:21:49+00:00"
        //         }, ...
        //     ]
        // }

        let res = version_manifest_json["versions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["id"] == version_id)
            .or_else(|| None);

        if res == None {
            return Err(String::from("版本未找到"));
        }
        let res = res.unwrap();

        let mut urls = vec![];
        // ----- version.json ----- //

        let version = match reqwest::blocking::get(res["url"].as_str().unwrap()) {
            Ok(result) => match result.text() {
                Ok(result) => result,
                Err(err) => return Err(err.to_string()),
            },
            Err(err) => return Err(err.to_string()),
        };

        create_dir_all(Path::new(&format!(
            "./.minecraft/versions/{}/natives/",
            &name
        )))
        .unwrap();
        write_to_file(
            &format!("./.minecraft/versions/{}/{}.json", &name, &name),
            version.as_bytes(),
        );

        let version_json: Value = serde_json::from_str(&version).unwrap();
        // {
        //     "arguments": {
        //         ...
        //     },
        //     "assetIndex": {
        //         "id": "1.19",
        //         "sha1": "d45eb5e0c20e5d753468de3d68b05c45a946f49b",
        //         "size": 385416,
        //         "totalSize": 553754183,
        //         "url": "https://piston-meta.mojang.com/v1/packages/d45eb5e0c20e5d753468de3d68b05c45a946f49b/1.19.json"
        //     },
        //     "assets": "1.19",
        //     "complianceLevel": 1,
        //     "downloads": {
        //         "client": {
        //             "sha1": "dc26b29eb345cbb60e3939af0b7c78fa52a60daa",
        //             "size": 21550637,
        //             "url": "https://piston-data.mojang.com/v1/objects/dc26b29eb345cbb60e3939af0b7c78fa52a60daa/client.jar"
        //         },...
        //     },
        //     "id": "22w24a",
        //     "javaVersion": {
        //         "component": "java-runtime-gamma",
        //         "majorVersion": 17
        //     },
        //     "libraries": [
        //         {
        //             "downloads": {
        //                 "artifact": {
        //                     "path": "com/mojang/logging/1.0.0/logging-1.0.0.jar",
        //                     "sha1": "f6ca3b2eee0b80b384e8ed93d368faecb82dfb9b",
        //                     "size": 15343,
        //                     "url": "https://libraries.minecraft.net/com/mojang/logging/1.0.0/logging-1.0.0.jar"
        //                 }
        //             },
        //             "name": "com.mojang:logging:1.0.0"
        //         },
        //         ...
        //     ],
        //     "logging": {
        //         "client": {
        //             "argument": "-Dlog4j.configurationFile=${path}",
        //             "file": {
        //                 "id": "client-1.12.xml",
        //                 "sha1": "bd65e7d2e3c237be76cfbef4c2405033d7f91521",
        //                 "size": 888,
        //                 "url": "https://launcher.mojang.com/v1/objects/bd65e7d2e3c237be76cfbef4c2405033d7f91521/client-1.12.xml"
        //             },
        //             "type": "log4j2-xml"
        //         }
        //     },
        //     "mainClass": "net.minecraft.client.main.Main",
        //     "minimumLauncherVersion": 21,
        //     "releaseTime": "2022-06-15T16:21:49+00:00",
        //     "time": "2022-06-15T16:21:49+00:00",
        //     "type": "snapshot"
        // }

        urls.push((
            /* path */ format!("./.minecraft/versions/{}/{}.jar", name, name),
            /* url  */
            version_json["downloads"]["client"]["url"]
                .as_str()
                .unwrap()
                .to_string(),
        ));

        if !version_json["logging"]["client"]["file"]["id"].is_null() {
            urls.push((
                /* path */
                format!(
                    "./.minecraft/versions/{}/{}",
                    name,
                    version_json["logging"]["client"]["file"]["id"]
                        .as_str()
                        .unwrap()
                ),
                /* url  */
                version_json["logging"]["client"]["file"]["url"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            ));
        }

        for item in version_json["libraries"].as_array().unwrap() {
            // 是否有 artifact 键
            if !item["downloads"]["artifact"].is_null() {
                urls.push((
                    /* path */
                    format!(
                        "./.minecraft/libraries/{}",
                        item["downloads"]["artifact"]["path"].as_str().unwrap()
                    ),
                    /* url  */
                    item["downloads"]["artifact"]["url"]
                        .as_str()
                        .unwrap()
                        .to_string(),
                ));
            }

            // 是否有 classifiers 键
            if !item["downloads"]["classifiers"].is_null() {
                let classifiers = item["downloads"]["classifiers"].borrow();

                // linux
                if !classifiers["natives-linux"].is_null() {
                    urls.push((
                        /* path */
                        format!(
                            "./.minecraft/libraries/{}",
                            classifiers["natives-linux"]["path"].as_str().unwrap()
                        ),
                        /* url  */
                        classifiers["natives-linux"]["url"]
                            .as_str()
                            .unwrap()
                            .to_string(),
                    ));
                }

                // windows
                if !classifiers["natives-windows"].is_null() {
                    urls.push((
                        /* path */
                        format!(
                            "./.minecraft/libraries/{}",
                            classifiers["natives-windows"]["path"].as_str().unwrap()
                        ),
                        /* url  */
                        classifiers["natives-windows"]["url"]
                            .as_str()
                            .unwrap()
                            .to_string(),
                    ));
                }

                // osx
                if !classifiers["natives-osx"].is_null() {
                    urls.push((
                        /* path */
                        format!(
                            "./.minecraft/libraries/{}",
                            classifiers["natives-osx"]["path"].as_str().unwrap()
                        ),
                        /* url  */
                        classifiers["natives-osx"]["url"]
                            .as_str()
                            .unwrap()
                            .to_string(),
                    ));
                }
            }
        }

        // ----- assets.json ----- //

        let assets =
            match reqwest::blocking::get(version_json["assetIndex"]["url"].as_str().expect("ERR!"))
            {
                Ok(result) => match result.text() {
                    Ok(result) => result,
                    Err(err) => return Err(err.to_string()),
                },
                Err(err) => return Err(err.to_string()),
            };

        write_to_file(
            &format!(
                "./.minecraft/assets/indexes/{}.json",
                version_json["assets"].as_str().unwrap()
            ),
            assets.as_bytes(),
        );

        let assets_json: Value = serde_json::from_str(&assets).unwrap();

        for (_, obj) in assets_json["objects"].as_object().unwrap() {
            let hash = obj["hash"].as_str().unwrap();
            let two = &hash[0..=1];

            if !PathBuf::from(format!("./.minecraft/assets/objects/{}/{}", two, hash)).exists() {
                urls.push((
                    /* path */ format!("./.minecraft/assets/objects/{}/{}", two, hash),
                    /* url  */
                    format!("https://resources.download.minecraft.net/{}/{}", two, hash),
                ));
            }
        }

        // ----- download ----- //
        let size = urls.len();
        let urls = Arc::new(Mutex::new(urls));
        let progress_bar = Arc::new(Mutex::new(ProgressBar::new(size.try_into().unwrap())));
        let mut handles = vec![];

        for _ in 0..size {
            let urls = urls.clone();
            let progress_bar = progress_bar.clone();
            let handle = thread::spawn(move || {
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                let mut urls = urls.lock().unwrap();
                let url = urls[0].clone();
                urls.remove(0);

                let progress_bar = progress_bar.lock().unwrap();

                runtime.block_on(downloader::download(&url.0, &url.1));

                progress_bar.inc(1);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        progress_bar.clone().lock().unwrap().finish();

        Ok(())
    }
}

pub struct LaunchInfo {
    pub player_name: String,
    pub uuid: String,
    pub version: String,
    pub name: String,
    pub demo: bool,
}

pub struct Launcher {}

impl Launcher {
    pub fn new() -> Launcher {
        Launcher {}
    }

    /// 启动一个游戏。
    ///
    /// `info`: 要启动的版本的信息
    ///
    /// 返回: `Ok()` 表示成功，`Err(str)` 表示失败，并返回一个字符串。
    ///
    pub fn start(&self, info: LaunchInfo) -> Result<(), String> {
        let mut class_path = String::new();

        // manifest.json 不存在
        if !Path::new(&format!(
            "./.minecraft/versions/{}/{}.json",
            info.name, info.name
        ))
        .exists()
        {
            return Err(format!(
                "./.minecraft/versions/{}/{}.json 不存在",
                info.name, info.name
            ));
        }

        let version_manifest = std::fs::read_to_string(&format!(
            "./.minecraft/versions/{}/{}.json",
            info.name, info.name
        ))
        .unwrap();
        let version_manifest: Value = serde_json::from_str(&version_manifest).unwrap();

        // class_path
        let mut result = version_manifest["libraries"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|x| {
                // 如果没有 downloads->artifact 键，返回 false
                if x["downloads"]["artifact"].is_null() {
                    return false;
                }

                // 如果没有 rules 键，代表所有系统都可用
                if x["rules"].is_null() {
                    return true;
                }

                // (windows, linux, osx)
                let mut available = (false, false, false);

                for item in x["rules"].as_array().unwrap() {
                    match item["action"].as_str().unwrap() {
                        "allow" => {
                            // 没有 os 键
                            if item["os"].is_null() {
                                available = (true, true, true);
                            }
                            // 有 os 键
                            else {
                                match item["os"]["name"].as_str().unwrap() {
                                    "windows" => {
                                        available.0 = true;
                                    }
                                    "linux" => {
                                        available.1 = true;
                                    }
                                    "osx" => {
                                        available.2 = true;
                                    }
                                    _ => (),
                                }
                            }
                        }
                        "disallow" => match item["os"]["name"].as_str().unwrap() {
                            "windows" => {
                                available.0 = false;
                            }
                            "linux" => {
                                available.1 = false;
                            }
                            "osx" => {
                                available.2 = false;
                            }
                            _ => (),
                        },
                        _ => (),
                    }
                }

                if (cfg!(windows) && available.0) || (cfg!(linux) && available.1) || (cfg!(osx) && available.2) {
                    return true;
                }

                false
            });

        // 分析所有得到的 library 项
        loop {
            let next = result.next();
            if next == None {
                break;
            }

            let next = next.unwrap();
            class_path.push_str(
                get_path(&format!(
                    "./.minecraft/libraries/{}",
                    next["downloads"]["artifact"]["path"].as_str().unwrap()
                ))
                .to_str()
                .unwrap(),
            );

            if cfg!(windows) {
                class_path.push(';');
            } else if cfg!(linux) {
                class_path.push(':');
            }
        }

        // 游戏主文件
        class_path.push_str(
            get_path(&format!(
                "./.minecraft/versions/{}/{}.jar",
                info.name, info.name
            ))
            .to_str()
            .unwrap(),
        );

        // 获取所有 natives 项
        let mut result = version_manifest["libraries"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|x| {
                if !x["natives"].is_null() {
                    return true;
                }
                false
            });

        // 分析所有得到的 natives 项

        let path_str = format!("./.minecraft/versions/{}/natives", info.name);
        let natives_path = Path::new(&path_str);

        loop {
            let next = result.next();
            if next == None {
                break;
            }

            let next = next.unwrap();

            // 如果这个文件夹不存在
            if !natives_path.exists() {
                std::fs::create_dir_all(natives_path).unwrap();
            }

            if cfg!(windows) && !next["downloads"]["classifiers"]["natives-windows"].is_null() {
                crate::extract(
                    Path::new(&format!(
                        "./.minecraft/libraries/{}",
                        next["downloads"]["classifiers"]["natives-windows"]["path"]
                            .as_str()
                            .unwrap()
                    )),
                    natives_path,
                )
                .unwrap();
            } else if cfg!(linux) && !next["downloads"]["classifiers"]["natives-linux"].is_null() {
                crate::extract(
                    Path::new(&format!(
                        "./.minecraft/libraries/{}",
                        next["downloads"]["classifiers"]["natives-linux"]["path"]
                            .as_str()
                            .unwrap()
                    )),
                    natives_path,
                )
                .unwrap();
            }
        }

        let mut arguments_string = String::new();

        // arguments 这个键还在
        if !version_manifest["arguments"].is_null() {
            let mut jvm_arguments = String::new();

            for item in version_manifest["arguments"]["jvm"].as_array().unwrap() {
                if item.is_string() {
                    jvm_arguments.push_str(item.as_str().unwrap());
                    jvm_arguments.push_str(" ");
                }
            }

            let mut game_arguments = String::new();

            for item in version_manifest["arguments"]["game"].as_array().unwrap() {
                if item.is_string() {
                    game_arguments.push_str(item.as_str().unwrap());
                    game_arguments.push_str(" ");
                }
            }

            arguments_string = format!(
                "{} net.minecraft.client.main.Main {}",
                jvm_arguments, game_arguments
            );

            arguments_string = arguments_string
                .replace(
                    "${natives_directory}",
                    get_path(&path_str).to_str().unwrap(),
                )
                .replace("${launcher_name}", "command-minecraft-launcher")
                .replace("${launcher_version}", "0.0.0")
                .replace("${classpath}", &class_path)
                .replace("${auth_player_name}", &info.player_name)
                .replace("${version_name}", version_manifest["id"].as_str().unwrap())
                .replace(
                    "${game_directory}",
                    get_path("./.minecraft/").to_str().unwrap(),
                )
                .replace(
                    "${assets_root}",
                    get_path("./.minecraft/assets/").to_str().unwrap(),
                )
                .replace(
                    "${assets_index_name}",
                    version_manifest["assetIndex"]["id"].as_str().unwrap(),
                )
                .replace("${auth_uuid}", &info.uuid)
                .replace("${auth_access_token}", &info.uuid)
                .replace("${user_type}", "msa")
                .replace(
                    "${version_type}",
                    version_manifest["type"].as_str().unwrap(),
                );
        }

        match Command::new("java")
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .args(arguments_string.split_whitespace().collect::<Vec<&str>>())
            .spawn()
        {
            Ok(_) => return Ok(()),
            Err(error) => return Err(error.to_string()),
        }

	Ok(())
    }
}

pub struct Login {
    logged: bool,
    name: String,
    uuid: String,
    refresh_id: String,
}

impl Login {
    pub fn new() -> Login {
        Login {
            logged: false,
            name: String::new(),
            uuid: String::new(),
            refresh_id: String::new(),
        }
    }

    pub async fn login_from_microsoft(code: String) -> Result<Login, reqwest::Error> {

        let mut result = Login::new();

        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/x-www-form-urlencoded".parse().unwrap());

        let data = json!({
            "client_id": "00000000402b5328",
            "code": &code,
            "grant_type": "authorization_code",
            "redirect_uri": "https://login.live.com/oauth20_desktop.srf",
            "scope": "service::user.auth.xboxlive.com::MBI_SSL",
        }).to_string();

        let poster = Post::new();
        let response = poster.post("https://login.live.com/oauth20_token.srf", headers, data).await?.json::<Value>().await?;
        
        println!("{:?}", json!({
            "client_id": "00000000402b5328",
            "code": &code,
            "grant_type": "authorization_code",
            "redirect_uri": "https://login.live.com/oauth20_desktop.srf",
            "scope": "service::user.auth.xboxlive.com::MBI_SSL",
        }).to_string());
        
        Ok(result)
        // result.refresh_id = String::from(response["refresh_token"].as_str().unwrap());

        // let mut headers = HeaderMap::new();
        // headers.insert("Content-Type", "application/json".parse().unwrap());
        // headers.insert("Accept", "application/json".parse().unwrap());

        // let data = json!({
        //     "Properties": {
        //         "AuthMethod": "RPS",
        //         "SiteName": "user.auth.xboxlive.com",
        //         "RpsTicket": &format!("d={}", response["access_token"].as_str().unwrap()),
        //     },
        //     "RelyingParty": "http://auth.xboxlive.com",
        //     "TokenType": "JWT",
        // }).to_string();

        // let response = poster.post("https://user.auth.xboxlive.com/user/authenticate", headers, data).await?.json::<Value>().await?;

        // let uhs = response["DisplayClaims"]["xui"].as_array().unwrap()[0].as_str().unwrap();

        // let mut headers = HeaderMap::new();
        // headers.insert("Content-Type", "application/json".parse().unwrap());
        // headers.insert("Accept", "application/json".parse().unwrap());

        // let data = json!({
        //     "Properties": {
        //         "SandboxId": "RETAIL",
        //         "UserTokens": [
        //             response["Token"].as_str().unwrap()
        //         ]
        //     },
        //     "RelyingParty": "rp://api.minecraftservices.com/",
        //     "TokenType": "JWT"
        // }).to_string();

        // let response = poster.post("https://user.auth.xboxlive.com/user/authenticate", headers, data).await?.json::<Value>().await?;

        // let mut headers = HeaderMap::new();
        // headers.insert("Content-Type", "application/json".parse().unwrap());
        // headers.insert("Accept", "application/json".parse().unwrap());

        // let data = json!({
        //     "identityToken": &format!("XBL3.0 x={};{}", uhs, response["Token"].as_str().unwrap())
        // }).to_string();

        // let response = poster.post("https://user.auth.xboxlive.com/user/authenticate", headers, data).await?.json::<Value>().await?;

        // panic!("{}", response["access_token"].as_str().unwrap());

        // Ok(result)
    }
}
