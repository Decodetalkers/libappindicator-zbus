use libappindicator_zbus::{
    tray,
    utils::{ButtonOptions, Category, EventUpdate, MenuStatus, MenuTree, MenuUnit, TextDirection},
};
use zbus::fdo::Result;

struct Base;

impl Base {
    fn boot() -> Self {
        Base
    }

    fn activate(&mut self, _x: i32, _y: i32) -> Result<()> {
        println!("active");
        Ok(())
    }
    fn context_menu(&mut self, _x: i32, _y: i32) -> Result<()> {
        println!("receive");
        Ok(())
    }
    fn scroll(&mut self, _delta: i32, _orientation: &str) -> Result<()> {
        Ok(())
    }
    fn secondary_activate(&mut self, _x: i32, _y: i32) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Clicked,
    Toggled,
}

#[allow(unused)]
struct Menu {
    menu: MenuTree<Message>,
}

impl Menu {
    fn boot() -> Self {
        let menu = Self::menu();
        Menu { menu }
    }

    fn menu() -> MenuTree<Message> {
        MenuTree::new()
            .push_sub_menu(MenuUnit::button(
                ButtonOptions {
                    label: "Hello".to_owned(),
                    enabled: true,
                    icon_name: "nheko".to_owned(),
                },
                Message::Clicked,
            ))
            .push_sub_menu(MenuUnit::button(
                ButtonOptions {
                    label: "World".to_owned(),
                    icon_name: "fcitx_pinyin".to_owned(),
                    enabled: true,
                },
                Message::Toggled,
            ))
            .push_sub_menu(
                MenuUnit::sub_menu("Next".to_owned()).push_sub_menu(MenuUnit::button(
                    ButtonOptions {
                        label: "Good".to_owned(),
                        enabled: true,
                        icon_name: "wezterm".to_owned(),
                    },
                    Message::Clicked,
                )),
            )
    }

    fn status(&self) -> MenuStatus {
        MenuStatus::Normal
    }

    fn on_clicked(
        &mut self,
        button: &mut MenuUnit<Message>,
        message: Message,
        _timestamp: u32,
    ) -> EventUpdate {
        println!("message: {button:?}, {message:?}");
        EventUpdate::None
    }
}

#[tokio::main]
async fn main() {
    let connection = tray(Base::boot, "hello", "fake_nheko", Menu::boot, Menu::menu, 1)
        .with_item_is_menu(false)
        .with_icon_name("nheko")
        .with_activate(Base::activate)
        .with_category(Category::ApplicationStatus)
        .with_text_direction(TextDirection::Rtl)
        .with_context_menu(Base::context_menu)
        .with_scroll(Base::scroll)
        .with_secondary_activate(Base::secondary_activate)
        //.with_get_group_properties(Menu::get_group_properties)
        .with_menu_status(Menu::status)
        .with_on_clicked(Menu::on_clicked)
        .run()
        .await
        .unwrap();

    println!("{:?}", connection.unique_name());
    std::future::pending::<()>().await;
}
