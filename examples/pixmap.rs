use libappindicator_zbus::{
    tray,
    utils::{ButtonOptions, Category, EventUpdate, IconPixmap, MenuStatus, MenuUnit},
};
use zbus::fdo::Result;

const IMAGE_DATA: &[u8] = include_bytes!("../misc/logo.png");

struct Base {
    pixmap: IconPixmap,
}

impl Base {
    fn boot() -> Self {
        let data = image::load_from_memory(IMAGE_DATA).unwrap();
        let pixmap = IconPixmap {
            width: data.width() as i32,
            height: data.height() as i32,
            data: data.as_bytes().to_vec(),
        };
        Self { pixmap }
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
    fn icon_pixmap(&self) -> Result<Vec<IconPixmap>> {
        Ok(vec![self.pixmap.clone()])
    }
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Clicked,
    Toggled,
}

struct Menu {
    time: u32,
    reversion: u32,
}

impl Menu {
    fn boot() -> Self {
        Menu {
            time: 0,
            reversion: 0,
        }
    }

    fn menu() -> MenuUnit<Message> {
        MenuUnit::root()
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
    }
    fn status(&self) -> MenuStatus {
        MenuStatus::Normal
    }
    fn reversion(&self) -> u32 {
        self.reversion
    }
    fn on_clicked(&mut self, button: &mut MenuUnit<Message>, _timestamp: u32) -> EventUpdate {
        println!("Yes, here!");
        self.reversion += 1;
        self.time += 1;
        if let Some(label) = &mut button.property.label {
            println!("{label}");
            *label = format!("Hello{}", self.time);
        }
        println!("{:?}", button.property);
        EventUpdate::UpdateAll
    }
}

#[tokio::main]
async fn main() {
    let connection = tray(
        Base::boot,
        "pixmap_text",
        "pixmap_test",
        Menu::boot,
        Menu::menu,
        Menu::reversion,
    )
    .with_item_is_menu(false)
    .with_icon_pixmap(Base::icon_pixmap)
    .with_activate(Base::activate)
    .with_category(Category::ApplicationStatus)
    .with_context_menu(Base::context_menu)
    .with_scroll(Base::scroll)
    .with_secondary_activate(Base::secondary_activate)
    .with_menu_status(Menu::status)
    .with_on_clicked(Menu::on_clicked)
    .run()
    .await
    .unwrap();

    println!("{:?}", connection.unique_name());
    std::future::pending::<()>().await;
}
