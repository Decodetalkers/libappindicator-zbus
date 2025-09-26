use std::time::Duration;

use status_notifier::{
    dbusmenu::{MenuItem, MenuProperty, MenuStatus, PropertyItem},
    tray,
};
use zbus::{fdo::Result, zvariant::OwnedValue};

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

struct Menu {
    need_update: bool,
}

impl Menu {
    fn boot() -> Self {
        Menu { need_update: false }
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
                            enabled: Some(false),
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
        self.need_update = true;
        Ok(vec![
            PropertyItem {
                id: 1,
                item: MenuProperty {
                    label: Some("Hello".to_owned()),
                    icon_name: Some("input-method".to_owned()),
                    enabled: Some(true),
                    ..Default::default()
                },
            },
            PropertyItem {
                id: 2,
                item: MenuProperty {
                    label: Some("World".to_owned()),
                    icon_name: Some("fcitx_pinyin".to_owned()),
                    enabled: Some(false),
                    ..Default::default()
                },
            },
        ])
    }

    fn status(&self) -> MenuStatus {
        MenuStatus::Normal
    }
}

#[tokio::main]
async fn main() {
    let connection = tray(
        Base::boot,
        "hello",
        "nheko",
        "fake_nheko",
        Menu::boot,
        Menu::about_to_show,
    )
    .with_activate(Base::activate)
    .with_category("ApplicationStatus")
    .with_context_menu(Base::context_menu)
    .with_scroll(Base::scroll)
    .with_secondary_activate(Base::secondary_activate)
    .with_layout(Menu::get_layout)
    .with_get_group_properties(Menu::get_group_properties)
    .with_menu_status(Menu::status)
    .run()
    .await
    .unwrap();

    println!("{:?}", connection.unique_name());

    let mut revision = 0;
    let _ = connection.notify_layout_changed(revision, 0).await;

    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let need_update = connection
            .update_menu_state(|menu| {
                let need_update = menu.need_update;
                if need_update {
                    menu.need_update = false;
                }
                need_update
            })
            .await
            .unwrap();
        if need_update {
            revision += 1;
            let _ = connection.notify_layout_changed(revision, 0).await;
        }
    }

    //std::future::pending::<()>().await;
}
