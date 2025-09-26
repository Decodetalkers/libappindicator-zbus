use libappindicator_zbus::{
    tray,
    utils::{
        Category, EventUpdate, MenuItem, MenuProperty, MenuStatus, PropertyItem, TextDirection,
        ToggleState, ToggleType,
    },
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

struct Menu {
    menu: MenuItem,
}

impl Menu {
    fn boot() -> Self {
        let menu = MenuItem::new(0, MenuProperty::submenu())
            .push_sub_menu(MenuItem::new(
                1,
                MenuProperty {
                    label: Some("Hello".to_owned()),
                    icon_name: Some("input-method".to_owned()),
                    enabled: Some(true),
                    toggle_type: Some(ToggleType::Radio),
                    toggle_state: Some(ToggleState::UnSelected),
                    ..Default::default()
                },
            ))
            .push_sub_menu(MenuItem::new(
                2,
                MenuProperty {
                    label: Some("World".to_owned()),
                    icon_name: Some("fcitx_pinyin".to_owned()),
                    enabled: Some(true),
                    ..Default::default()
                },
            ));
        Menu { menu }
    }

    fn about_to_show(&mut self, _id: i32) -> Result<bool> {
        Ok(true)
    }

    fn get_layout(
        &mut self,
        _parent_id: i32,
        _recursion_depth: i32,
        _property_name: Vec<String>,
    ) -> Result<(u32, MenuItem)> {
        Ok((1, self.menu.clone()))
    }

    fn get_group_properties(
        &mut self,
        ids: Vec<i32>,
        _property_names: Vec<String>,
    ) -> zbus::fdo::Result<Vec<PropertyItem>> {
        Ok(self.menu.get_property_groups(ids))
    }

    fn status(&self) -> MenuStatus {
        MenuStatus::Normal
    }

    fn on_clicked(&mut self, _id: i32, _timestamp: u32) -> EventUpdate {
        println!("Yes, here!");
        EventUpdate::None
    }

    fn on_toggled(&mut self, id: i32, status: ToggleState, timestamp: u32) -> EventUpdate {
        println!("toggled, id = {id}, status = {status:?}, timestamp = {timestamp}");
        EventUpdate::None
    }
}

#[tokio::main]
async fn main() {
    let connection = tray(
        Base::boot,
        "hello",
        "fake_nheko",
        Menu::boot,
        Menu::about_to_show,
    )
    .with_item_is_menu(true)
    .with_icon_name("nheko")
    .with_activate(Base::activate)
    .with_category(Category::ApplicationStatus)
    .with_text_direction(TextDirection::Rtl)
    .with_context_menu(Base::context_menu)
    .with_scroll(Base::scroll)
    .with_secondary_activate(Base::secondary_activate)
    .with_layout(Menu::get_layout)
    .with_get_group_properties(Menu::get_group_properties)
    .with_menu_status(Menu::status)
    .with_on_clicked(Menu::on_clicked)
    .with_on_toggled(Menu::on_toggled)
    .run()
    .await
    .unwrap();

    println!("{:?}", connection.unique_name());
    std::future::pending::<()>().await;
}
