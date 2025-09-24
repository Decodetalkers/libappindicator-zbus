use status_notifier::status_notifier_item::tray;
use zbus::fdo::Result;

struct Base;

impl Base {
    fn boot() -> Self {
        Base
    }
    fn id() -> String {
        "Hello".to_owned()
    }
    fn activate(&mut self, x: i32, y: i32) -> Result<()> {
        println!("active");
        Ok(())
    }
    fn context_menu(&mut self, x: i32, y: i32) -> Result<()> {
        Ok(())
    }
    fn scroll(&mut self, delta: i32, orientation: &str) -> Result<()> {
        Ok(())
    }
    fn secondary_activate(&mut self, x: i32, y: i32) -> Result<()> {
        Ok(())
    }
    fn icon_name(&self) -> Result<String> {
        Ok("nheko".to_owned())
    }
}

#[tokio::main]
async fn main() {
    let _connection = tray(Base::boot, Base::id, Base::activate, Base::context_menu)
        .with_icon_name(Base::icon_name)
        .with_scroll(Base::scroll)
        .with_secondary_activate(Base::secondary_activate)
        .run()
        .await
        .unwrap();

    println!("{:?}", _connection.unique_name());

    std::future::pending::<()>().await;
}
