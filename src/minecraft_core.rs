use crate::get_path;
use crate::write_to_file;
use futures::StreamExt;
use reqwest::Error;
use serde_json::Value;
use std::borrow::Borrow;
use std::fs::create_dir_all;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use tokio::runtime::Runtime;

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
        let version_manifest = match reqwest::blocking::get(VERSION_MANIFEST_URL) {
            Ok(result) => match result.text() {
                Ok(result) => result,
                Err(err) => return Err(err.to_string()),
            },
            Err(err) => return Err(err.to_string()),
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

        println!("完成1");

        // ----- assets.json ----- //

        println!("正在获取 assets.json... ");

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
            } else {
                println!(
                    "文件 {} 已存在, 跳过",
                    format!("https://resources.download.minecraft.net/{}/{}", two, hash)
                );
            }
        }
        println!("完成2");

        // ----- download ----- //
        println!("需要下载的文件总数: {}", urls.len());
        // 准备下载链接和线程数等参数

        let n = 16; // 分成4份

        let shared_data = Arc::new(Mutex::new(urls));

        let mut handles = vec![];

        for _ in 0..n {
            let shared_data = Arc::clone(&shared_data);
            let handle = thread::spawn(move || {
                let mut data = shared_data.lock().unwrap();

                // 例如，将data拆分的每一份传递给不同的线程进行处理
                // 注意：这里只是一个示例，具体的操作取决于你的需求
                let chunk_size = data.len() / n;
                let thread_data = data.split_off(chunk_size);

                let rt = Runtime::new().unwrap();
                rt.block_on(async {
                    let fetches = futures::stream::iter(thread_data.into_iter().map(
                        |(path, url)| async move {
                            loop {
                                match reqwest::get(&url).await {
                                    Ok(resp) => match resp.bytes().await {
                                        Ok(text) => {
                                            println!("{} 下载完成", url);
                                            write_to_file(&path, &text);
                                            break;
                                        }
                                        Err(_) => {
                                            println!("{} 在读取时发生错误", url);
                                            continue;
                                        }
                                    },
                                    Err(_) => {
                                        println!("{} 在下载时发生错误", url);
                                        continue;
                                    }
                                }
                            }
                        },
                    ))
                    .buffer_unordered(65535)
                    .collect::<Vec<()>>();
                    fetches.await;
                });
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

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
    /// `version`: 游戏版本号。
    ///
    /// `name`: 游戏自定义名称。
    ///
    /// `demo`: 是否是 demo 版。
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

                if (cfg!(windows) && available.0) || (cfg!(linux) && available.1) {
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
            Ok(_) => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }
}

/// 镜像源设置
pub struct MirrorSourceOptions {
    // 版本信息
    pub version_manifest: String,
    // 版本和版本 JSON 以及 AssetsIndex
    pub version_and_assets_index_from: String,
    pub version_and_assets_index_to: String,
    // Assets
    pub assets_from: String,
    pub assets_to: String,
    // Libraries
    pub lib_from: String,
    pub lib_to: String,
}
