use libappindicator_zbus::{
    tray,
    utils::{
        Category, EventUpdate, IconPixmap, MenuProperty, MenuStatus, MenuUnit, ToggleState,
        ToggleType,
    },
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
#[derive(Clone, Copy)]
struct Message;

struct Menu {
    menu: MenuUnit<Message>,
}

impl Menu {
    fn boot() -> Self {
        let menu = MenuUnit::new(MenuProperty::submenu(), Message)
            .push_sub_menu(MenuUnit::new(
                MenuProperty {
                    label: Some("Hello".to_owned()),
                    icon_name: Some("input-method".to_owned()),
                    enabled: Some(true),
                    toggle_type: Some(ToggleType::Radio),
                    toggle_state: Some(ToggleState::UnSelected),
                    ..Default::default()
                },
                Message,
            ))
            .push_sub_menu(MenuUnit::new(
                MenuProperty {
                    label: Some("World".to_owned()),
                    icon_name: Some("fcitx_pinyin".to_owned()),
                    enabled: Some(true),
                    ..Default::default()
                },
                Message,
            ));
        Menu { menu }
    }

    fn menu(&self) -> MenuUnit<Message> {
        self.menu.clone()
    }

    fn status(&self) -> MenuStatus {
        MenuStatus::Normal
    }

    fn on_clicked(&mut self, _message: Message, _timestamp: u32) -> EventUpdate {
        println!("Yes, here!");
        EventUpdate::None
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
        1,
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
