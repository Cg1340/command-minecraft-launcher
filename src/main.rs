use std::sync::Mutex;

use cursive::{
    view::{Nameable, Resizable, Scrollable},
    views::{Button, Checkbox, Dialog, EditView, LinearLayout, SelectView, TextView},
    Cursive, CursiveExt,
};
use lazy_static::lazy_static;
use regex::Regex;
use command_minecraft_launcher::{minecraft_core::{DownloadManager, GameVersion, Launcher, LaunchInfo}, generate_uuid_without_hyphens};


lazy_static! {
    static ref NIGANMA: Mutex<u32> = Mutex::new(42);
    static ref PLAYER_NAME: Mutex<String> = Mutex::new(String::new());
}

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
                                        .fixed_width(10)
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

                    let info = LaunchInfo { player_name: player_name.clone(), uuid: generate_uuid_without_hyphens(&player_name), version: String::from(""), name, demo };

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

    // return //
    Dialog::new().title("Main").content(
        LinearLayout::horizontal()
            .child(
                LinearLayout::vertical()
                    .child(TextView::new("玩家名").center())
                    .child(
                        LinearLayout::horizontal()
                            .child(Button::new("更改名字", change_name))
                            .child(TextView::new("").with_name("player_name")),
                    ),
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
    let mut cursive_main = Cursive::default();
    cursive_main.set_fps(30);
    cursive_main.add_layer(dialog_main());
    cursive_main.run();
}
