use colored::Colorize;
use command_minecraft_launcher::{
    generate_uuid_without_hyphens,
    minecraft_core::{DownloadManager, GameVersion, LaunchInfo, Launcher, Login},
    post::Post,
};
use cursive::{
    view::{Nameable, Resizable, Scrollable},
    views::{Button, Checkbox, Dialog, EditView, LinearLayout, SelectView, TextView},
    Cursive, CursiveExt,
};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::header::HeaderMap;
use serde_json::{json, Value};
use std::{collections::HashMap, sync::Mutex};

lazy_static! {
    static ref NIGANMA: Mutex<u32> = Mutex::new(42);
    static ref PLAYER_NAME: Mutex<String> = Mutex::new(String::new());
}

const LOGIN_SUPER_LINK: &str = "https://login.live.com/oauth20_authorize.srf?client_id=00000000402b5328&response_type=code&scope=service%3A%3Auser.auth.xboxlive.com%3A%3AMBI_SSL&redirect_uri=https%3A%2F%2Flogin.live.com%2Foauth20_desktop.srf";

fn dialog_error(content: &str) -> Dialog {
    Dialog::new()
        .title("发生错误!")
        .content(TextView::new(content))
        .button("确定", |siv| {
            siv.pop_layer();
        })
}

fn dialog_quit() -> Dialog {
    Dialog::new()
        .title("Quit?")
        .content(
            LinearLayout::vertical()
                .child(TextView::new("你确定要退出吗?"))
                .child(
                    LinearLayout::horizontal()
                        .child(Checkbox::new().with_name("checkbox_tip_again"))
                        .child(TextView::new(" 不再提醒")),
                ),
        )
        .button("确定", |siv| {
            siv.quit();
        })
        .button("取消", |siv| {
            siv.pop_layer();
        })
}

fn dialog_main() -> Dialog {
    let change_name_submit = move |siv: &mut Cursive| {
        // 创建新的可变字符串副本
        let temp = match siv.call_on_name("edit_player_name", |view: &mut EditView| {
            (*view.get_content()).clone()
        }) {
            Some(result) => result,
            None => String::new(),
        };

        let pattern = Regex::new(r"[^\w]").unwrap();

        if pattern.is_match(&temp) {
            siv.add_layer(dialog_error("玩家名只能由英文, 数字, 下划线组成."));
        } else {
            siv.call_on_name("player_name", |view: &mut TextView| {
                view.set_content(&temp);
            });

            (*PLAYER_NAME.lock().unwrap()) = temp;

            siv.pop_layer();
        }
    };

    let change_name = move |siv: &mut Cursive| {
        siv.add_layer(
            Dialog::new()
                .title("更改名字")
                .content(EditView::default().with_name("edit_player_name"))
                .button("确定", change_name_submit)
                .button("取消", |siv| {
                    siv.pop_layer();
                }),
        )
    };

    let start_game = move |siv: &mut Cursive| {
        siv.add_layer(
            Dialog::new()
                .title("Start")
                .content(
                    LinearLayout::vertical()
                        .child(
                            LinearLayout::horizontal()
                                .child(TextView::new("版本名称: "))
                                .child(
                                    EditView::new()
                                        .with_name("edit_version_name")
                                        .fixed_width(10),
                                ),
                        )
                        .child(
                            LinearLayout::horizontal()
                                .child(Checkbox::new().with_name("checkbox_demo"))
                                .child(TextView::new("是 demo 版")),
                        ),
                )
                .button("启动!", |siv| {
                    let launcher = Launcher::new();

                    let name = match siv.call_on_name("edit_version_name", |view: &mut EditView| {
                        (*view.get_content()).clone()
                    }) {
                        Some(result) => result,
                        None => String::new(),
                    };

                    let demo = match siv.call_on_name("checkbox_demo", |checkbox: &mut Checkbox| {
                        checkbox.is_checked()
                    }) {
                        Some(result) => result,
                        None => false,
                    };

                    let player_name = PLAYER_NAME.lock().unwrap().to_string();

                    let info = LaunchInfo {
                        player_name: player_name.clone(),
                        uuid: generate_uuid_without_hyphens(&player_name),
                        version: String::from(""),
                        name,
                        demo,
                    };

                    match launcher.start(info) {
                        Ok(_) => {
                            siv.add_layer(
                                Dialog::new()
                                    .title("完成!")
                                    .content(TextView::new("操作成功地完成."))
                                    .button("确定", |siv| {
                                        siv.pop_layer();
                                        siv.pop_layer();
                                    }),
                            );
                        }
                        Err(err) => {
                            siv.add_layer(
                                Dialog::new()
                                    .title("发生错误!")
                                    .content(TextView::new(&err))
                                    .button("确定", |siv| {
                                        siv.pop_layer();
                                        siv.pop_layer();
                                    }),
                            );
                        }
                    }
                })
                .button("取消", |siv| {
                    siv.pop_layer();
                }),
        )
    };

    let login = Dialog::new()
        .title("步骤 1")
        .content(TextView::new(&format!("用浏览器打开 {}", LOGIN_SUPER_LINK)))
        .button("下一步", move |siv| {
            siv.pop_layer();
            siv.add_layer(
                Dialog::new()
                    .title("步骤 2")
                    .content(
                        LinearLayout::vertical()
                            .child(TextView::new(
                                "现在在重定向的链接中提取出 code 的参数并写在下面的输入框中",
                            ))
                            .child(EditView::new().with_name("edit_login_code")),
                    )
                    .button("确定", |siv| {
                        let code = match siv
                            .call_on_name("edit_login_code", |view: &mut EditView| {
                                (*view.get_content()).clone()
                            }) {
                            Some(result) => result,
                            None => String::new(),
                        };

                        let poster = Post::new();

                        let mut headers = HeaderMap::new();
                        headers.insert(
                            "Content-Type",
                            "application/x-www-form-urlencoded".parse().unwrap(),
                        );

                        let data = json!({
                            "client_id": "00000000402b5328",
                            "code": &code,
                            "grant_type": "authorization_code",
                            "redirect_uri": "https://login.live.com/oauth20_desktop.srf",
                            "scope": "service::user.auth.xboxlive.com::MBI_SSL",
                        });

                        let runtime = tokio::runtime::Builder::new_multi_thread()
                            .enable_all()
                            .build()
                            .unwrap();

                        match runtime.block_on(poster.post(
                            "https://login.live.com/oauth20_token.srf",
                            headers,
                            data.to_string(),
                        )) {
                            Ok(_) => (),
                            Err(err) => {
                                siv.add_layer(dialog_error(&err.to_string()));
                                siv.pop_layer();
                                return;
                            }
                        };

                        let response = poster.get("https://login.live.com/oauth20_token.srf");

                        let response = match runtime.block_on(response) {
                            Ok(result) => result,
                            Err(err) => {
                                siv.add_layer(dialog_error(&err.to_string()));
                                siv.pop_layer();
                                return;
                            }
                        };
                    }),
            );
        });

    let login = move |siv: &mut Cursive| {
        siv.add_layer(
            Dialog::new()
                .content(TextView::new("注意: 现已不再支持 Mojang 登录!"))
                .button("从微软登录", |siv| {
                    siv.add_layer(
                        Dialog::new()
                            .title("Step.1")
                            .content(TextView::new(&format!(
                                "用你的浏览器打开 {}",
                                LOGIN_SUPER_LINK
                            )))
                            .button("下一步", |siv| {
                                siv.pop_layer();
                                siv.add_layer(Dialog::new());
                            }),
                    );
                })
                .button("离线游戏", |siv| {
                    siv.add_layer(
                        Dialog::new().title("Input").content(
                            LinearLayout::vertical()
                                .child(TextView::new("你的名字: "))
                                .child(EditView::new()),
                        ),
                    );
                }),
        )
    };

    // return //
    Dialog::new().title("Main").content(
        LinearLayout::horizontal()
            .child(
                LinearLayout::vertical()
                    .child(
                        LinearLayout::horizontal()
                            .child(TextView::new("玩家名 "))
                            .child(TextView::new("").with_name("player_name")),
                    )
                    .child(Button::new("登录...", login)),
            )
            .child(
                LinearLayout::vertical()
                    .child(Button::new("开始游戏", start_game))
                    .child(Button::new("下载一个版本", |siv| {
                        let versions = match GameVersion::build() {
                            Ok(result) => result,
                            Err(err) => {
                                siv.add_layer(dialog_error(&err.to_string()));
                                return;
                            }
                        };

                        siv.add_layer(
                            Dialog::new()
                                .title("Download")
                                .content(
                                    LinearLayout::vertical()
                                        .child(TextView::new("游戏信息").center())
                                        .child(
                                            LinearLayout::horizontal()
                                                .child(TextView::new("自定义名称: "))
                                                .child(
                                                    EditView::new()
                                                        .with_name("name")
                                                        .fixed_width(10),
                                                ),
                                        )
                                        .child(
                                            LinearLayout::horizontal()
                                                .child(Button::new(
                                                    "选择一个版本...",
                                                    move |siv| {
                                                        let mut select = SelectView::new();
                                                        select.add_all_str(
                                                            versions.iter().map(|x| &x.version_id),
                                                        );
                                                        select.set_on_submit(|siv, x: &str| {
                                                            siv.pop_layer();

                                                            siv.call_on_name(
                                                                "version_name",
                                                                |view: &mut TextView| {
                                                                    view.set_content(x);
                                                                },
                                                            )
                                                        });
                                                        siv.add_layer(select.scrollable());
                                                    },
                                                ))
                                                .child(
                                                    TextView::new("You didn't choose anything.")
                                                        .with_name("version_name"),
                                                ),
                                        ),
                                )
                                .button("确定", |siv| {
                                    let version_id = match siv.call_on_name(
                                        "version_name",
                                        |view: &mut TextView| {
                                            view.get_content().source().to_owned()
                                        },
                                    ) {
                                        Some(result) => result,
                                        None => String::new(),
                                    };

                                    let name = match siv
                                        .call_on_name("name", |view: &mut EditView| {
                                            view.get_content()
                                        }) {
                                        Some(result) => result.as_str().to_owned(),
                                        None => String::new(),
                                    };

                                    let download_manager = DownloadManager::new();
                                    let result =
                                        download_manager.download_version(&version_id, &name);

                                    match result {
                                        Ok(_) => {
                                            siv.add_layer(
                                                Dialog::new()
                                                    .title("完成!")
                                                    .content(TextView::new("操作成功地完成."))
                                                    .button("确定", |siv| {
                                                        siv.pop_layer();
                                                        siv.pop_layer();
                                                    }),
                                            );
                                        }
                                        Err(err) => {
                                            siv.add_layer(
                                                Dialog::new()
                                                    .title("发生错误!")
                                                    .content(TextView::new(&err))
                                                    .button("确定", |siv| {
                                                        siv.pop_layer();
                                                        siv.pop_layer();
                                                    }),
                                            );
                                        }
                                    }
                                })
                                .button("取消", |siv| {
                                    siv.pop_layer();
                                }),
                        );
                    }))
                    .child(Button::new("退出", |siv| {
                        siv.add_layer(dialog_quit());
                    })),
            ),
    )
}

fn main() {

    let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap();

    runtime.block_on(Login::login_from_microsoft(String::from("M.C103_BAY.2.0c333741-6c50-ed27-46c1-050fc992fd83")));

    // let mut cursive_main = Cursive::default();
    // cursive_main.set_fps(30);
    // cursive_main.add_layer(dialog_main());
    // cursive_main.run();
}
