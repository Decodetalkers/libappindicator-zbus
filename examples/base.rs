use libappindicator_zbus::{
    tray,
    utils::{
        ButtonOptions, Category, EventUpdate, MenuStatus, MenuTree, MenuUnit, RadioGroupBuilder,
        RadioOptions, TextDirection, ToggleState, ToggleType,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Message {
    Clicked,
    Toggled(i32),
}

#[allow(unused)]
struct Menu {
    menu: MenuTree<Message>,
    revision: u32,
}

impl Menu {
    fn boot() -> Self {
        let menu = Self::menu();
        Menu { menu, revision: 0 }
    }

    fn revision(&self) -> u32 {
        self.revision
    }

    fn menu() -> MenuTree<Message> {
        let group = RadioGroupBuilder::new()
            .append(
                RadioOptions {
                    label: "abc".to_owned(),
                    enabled: true,
                    toggle_type: ToggleType::Checkmark,
                    toggle_state: ToggleState::Selected,
                    ..Default::default()
                },
                Message::Toggled(1),
            )
            .append(
                RadioOptions {
                    label: "efg".to_owned(),
                    enabled: true,
                    toggle_type: ToggleType::Checkmark,
                    toggle_state: ToggleState::UnSelected,
                    ..Default::default()
                },
                Message::Toggled(2),
            );
        MenuTree::new()
            .push(MenuUnit::button(
                ButtonOptions {
                    label: "Hello".to_owned(),
                    enabled: true,
                    icon_name: "nheko".to_owned(),
                },
                Message::Clicked,
            ))
            .push(MenuUnit::radio_group(group))
            .push(MenuUnit::button(
                ButtonOptions {
                    label: "World".to_owned(),
                    icon_name: "fcitx_pinyin".to_owned(),
                    enabled: true,
                },
                Message::Clicked,
            ))
            .push(MenuUnit::sub_menu("Next".to_owned()).push(MenuUnit::button(
                ButtonOptions {
                    label: "Good".to_owned(),
                    enabled: true,
                    icon_name: "wezterm".to_owned(),
                },
                Message::Clicked,
            )))
    }

    fn status(&self) -> MenuStatus {
        MenuStatus::Normal
    }

    fn on_clicked(
        &mut self,
        button: &mut MenuUnit<Message>,
        forward_message: Message,
        _timestamp: u32,
    ) -> EventUpdate {
        if let MenuUnit::RadioGroup { selections } = button {
            for selection in selections.iter_mut() {
                let MenuUnit::RadioButton {
                    options, message, ..
                } = selection
                else {
                    continue;
                };
                options.toggle_state = ToggleState::UnSelected;
                if forward_message == *message {
                    options.toggle_state = ToggleState::Selected;
                }
            }
            self.revision += 1;
            return EventUpdate::UpdateCurrent;
        }
        println!("message: {button:?}, {forward_message:?}");
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
        Menu::menu,
        Menu::revision,
    )
    .with_item_is_menu(false)
    .with_icon_name("nheko")
    .with_activate(Base::activate)
    .with_category(Category::ApplicationStatus)
    .with_text_direction(TextDirection::Rtl)
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
