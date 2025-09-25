use status_notifier::{
    dbusmenu::{MenuItem, MenuProperty},
    tray,
};
use zbus::{fdo::Result, zvariant::OwnedValue};

struct Base;

impl Base {
    fn boot() -> Self {
        Base
    }
    fn id() -> String {
        "Hello".to_owned()
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
    fn icon_name(&self) -> Result<String> {
        Ok("nheko".to_owned())
    }
}

struct Menu;

impl Menu {
    fn boot() -> Self {
        Menu
    }
    fn about_to_show(&mut self, id: i32) -> Result<bool> {
        println!("{id}");
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
                id: 1,
                item: MenuProperty::submenu(),
                sub_menus: vec![
                    OwnedValue::try_from(MenuItem {
                        id: 2,
                        item: MenuProperty {
                            label: Some("Hello".to_owned()),
                            icon_name: Some("input-method".to_owned()),
                            ..Default::default()
                        },
                        sub_menus: vec![],
                    })
                    .unwrap(),
                    OwnedValue::try_from(MenuItem {
                        id: 3,
                        item: MenuProperty {
                            label: Some("Hello".to_owned()),
                            icon_name: Some("input-method".to_owned()),
                            ..Default::default()
                        },
                        sub_menus: vec![],
                    })
                    .unwrap(),
                ],
            },
        ))
    }
    fn status(&self) -> Result<String> {
        Ok("normal".to_string())
    }
}

#[tokio::main]
async fn main() {
    let connection = tray(
        Base::boot,
        Base::id,
        Base::activate,
        Base::icon_name,
        "SystemService",
        Menu::boot,
        Menu::about_to_show,
        Menu::status,
    )
    .with_context_menu(Base::context_menu)
    .with_scroll(Base::scroll)
    .with_secondary_activate(Base::secondary_activate)
    .with_layout(Menu::get_layout)
    .run()
    .await
    .unwrap();

    println!("{:?}", connection.unique_name());

    let _ = connection.notify_layout_changed(0, 0).await;

    std::future::pending::<()>().await;
}
