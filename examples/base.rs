use status_notifier::tray;
use zbus::fdo::Result;

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
}

#[tokio::main]
async fn main() {
    let connection = tray(
        Base::boot,
        Base::id,
        Base::activate,
        Menu::boot,
        Menu::about_to_show,
    )
    .with_context_menu(Base::context_menu)
    .with_icon_name(Base::icon_name)
    .with_scroll(Base::scroll)
    .with_secondary_activate(Base::secondary_activate)
    .run()
    .await
    .unwrap();

    println!("{:?}", connection.unique_name());

    std::future::pending::<()>().await;
}
