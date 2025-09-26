use libappindicator_zbus::{
    tray,
    utils::{
        EventUpdate, IconPixmap, MenuItem, MenuProperty, MenuStatus, PropertyItem, ToggleState,
        ToggleType,
    },
};
use zbus::{fdo::Result, zvariant::OwnedValue};

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

struct Menu;

impl Menu {
    fn boot() -> Self {
        Menu
    }

    fn about_to_show(&mut self, id: i32) -> Result<bool> {
        println!("about {id}");
        Ok(true)
    }

    fn get_layout(
        &mut self,
        _parent_id: i32,
        _recursion_depth: i32,
        _property_name: Vec<String>,
    ) -> Result<(u32, MenuItem)> {
        Ok((
            1,
            MenuItem {
                id: 0,
                item: MenuProperty::submenu(),
                sub_menus: vec![
                    OwnedValue::try_from(MenuItem {
                        id: 1,
                        item: MenuProperty {
                            label: Some("Hello".to_owned()),
                            icon_name: Some("input-method".to_owned()),
                            enabled: Some(true),
                            toggle_type: Some(ToggleType::Checkmark),
                            toggle_state: Some(ToggleState::UnSelected),
                            ..Default::default()
                        },
                        sub_menus: vec![],
                    })
                    .unwrap(),
                    OwnedValue::try_from(MenuItem {
                        id: 2,
                        item: MenuProperty {
                            label: Some("World".to_owned()),
                            icon_name: Some("fcitx_pinyin".to_owned()),
                            enabled: Some(true),
                            ..Default::default()
                        },
                        sub_menus: vec![],
                    })
                    .unwrap(),
                ],
            },
        ))
    }

    fn get_group_properties(
        &mut self,
        ids: Vec<i32>,
        property_names: Vec<String>,
    ) -> zbus::fdo::Result<Vec<PropertyItem>> {
        println!("{ids:?},{property_names:?}");
        Ok(vec![
            PropertyItem {
                id: 1,
                item: MenuProperty {
                    label: Some("Hello".to_owned()),
                    icon_name: Some("input-method".to_owned()),
                    enabled: Some(true),
                    toggle_type: Some(ToggleType::Checkmark),
                    toggle_state: Some(ToggleState::UnSelected),
                    ..Default::default()
                },
            },
            PropertyItem {
                id: 2,
                item: MenuProperty {
                    label: Some("World".to_owned()),
                    icon_name: Some("fcitx_pinyin".to_owned()),
                    enabled: Some(true),
                    ..Default::default()
                },
            },
        ])
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
        Menu::about_to_show,
    )
    .with_item_is_menu(true)
    .with_icon_pixmap(Base::icon_pixmap)
    .with_activate(Base::activate)
    .with_category("ApplicationStatus")
    .with_context_menu(Base::context_menu)
    .with_scroll(Base::scroll)
    .with_secondary_activate(Base::secondary_activate)
    .with_layout(Menu::get_layout)
    .with_get_group_properties(Menu::get_group_properties)
    .with_menu_status(Menu::status)
    .with_on_clicked(Menu::on_clicked)
    .run()
    .await
    .unwrap();

    println!("{:?}", connection.unique_name());
    std::future::pending::<()>().await;
}
