use libappindicator_zbus::{
    tray,
    utils::{
        Category, EventUpdate, IconPixmap, MenuItem, MenuProperty, MenuStatus, ToggleState,
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

struct Menu {
    menu: MenuItem,
}
impl Menu {
    fn boot() -> Self {
        let menu = MenuItem::new(MenuProperty::submenu())
            .push_sub_menu(MenuItem::new(MenuProperty {
                label: Some("Hello".to_owned()),
                icon_name: Some("input-method".to_owned()),
                enabled: Some(true),
                toggle_type: Some(ToggleType::Radio),
                toggle_state: Some(ToggleState::UnSelected),
                ..Default::default()
            }))
            .push_sub_menu(MenuItem::new(MenuProperty {
                label: Some("World".to_owned()),
                icon_name: Some("fcitx_pinyin".to_owned()),
                enabled: Some(true),
                ..Default::default()
            }));
        Menu { menu }
    }
    fn about_to_show(&mut self, id: i32) -> Result<bool> {
        println!("about {id}");
        Ok(true)
    }

    fn menu(&self) -> MenuItem {
        self.menu.clone()
    }

    fn status(&self) -> MenuStatus {
        MenuStatus::Normal
    }

    fn on_clicked(&mut self, _id: i32, _timestamp: u32) -> EventUpdate {
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
        Menu::about_to_show,
    )
    .with_item_is_menu(true)
    .with_icon_pixmap(Base::icon_pixmap)
    .with_activate(Base::activate)
    .with_category(Category::ApplicationStatus)
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
