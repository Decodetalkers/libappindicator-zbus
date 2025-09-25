use std::marker::PhantomData;

use crate::{
    dbusmenu::{
        AboutToShowFn, DBusMenuBootFn, DBusMenuInstance, DBusMenuItem, GetLayoutFn, MenuItem,
        StatusFn,
    },
    status_notifier_item::{
        ActivateFn, ContextMenuFn, IdFn, NotifierBootFn, ScrollFn, SecondaryActivateFn,
        StatusNotifierInstance, StatusNotifierItem,
    },
    status_notifier_watcher::StatusNotifierWatcherProxy,
};

pub mod dbusmenu;
pub mod status_notifier_item;
pub mod status_notifier_watcher;
use zbus::connection;

pub struct Tray<P: StatusNotifierItem, M: DBusMenuItem> {
    notifier_raw: P,
    menu_raw: M,
}

pub struct TrayConnection<P: StatusNotifierItem, M: DBusMenuItem> {
    conn: zbus::Connection,
    _notify_item: PhantomData<P>,
    _menu_item: PhantomData<M>,
}

impl<P: StatusNotifierItem, M: DBusMenuItem> TrayConnection<P, M>
where
    P::State: 'static + Send + Sync,
    P: Send + Sync + 'static,
    M::State: 'static + Send + Sync,
    M: Send + Sync + 'static,
{
    pub async fn update_notify_state<F>(&self, f: F) -> zbus::Result<()>
    where
        F: Fn(&mut P::State),
    {
        let iface_ref = self
            .conn
            .object_server()
            .interface::<_, StatusNotifierInstance<P>>("/StatusNotifierItem")
            .await?;
        let mut data = iface_ref.get_mut().await;
        f(&mut data.state);
        Ok(())
    }
    pub async fn update_menu_state<F>(&self, f: F) -> zbus::Result<()>
    where
        F: Fn(&mut M::State),
    {
        let iface_ref = self
            .conn
            .object_server()
            .interface::<_, DBusMenuInstance<M>>("/MenuBar")
            .await?;
        let mut data = iface_ref.get_mut().await;
        f(&mut data.state);
        Ok(())
    }

    pub async fn update_state<F>(&self, f: F) -> zbus::Result<()>
    where
        F: Fn(&mut P::State, &mut M::State),
    {
        let iface_ref = self
            .conn
            .object_server()
            .interface::<_, StatusNotifierInstance<P>>("/StatusNotifierItem")
            .await?;
        let menu_iface_ref = self
            .conn
            .object_server()
            .interface::<_, DBusMenuInstance<M>>("/MenuBar")
            .await?;
        let mut data = iface_ref.get_mut().await;
        let mut menu_data = menu_iface_ref.get_mut().await;
        f(&mut data.state, &mut menu_data.state);
        Ok(())
    }
    pub fn unique_name(&self) -> Option<&zbus::names::OwnedUniqueName> {
        self.conn.unique_name()
    }

    pub async fn notify_id_changed(&self) -> zbus::Result<()> {
        let iface_ref = self
            .conn
            .object_server()
            .interface::<_, StatusNotifierInstance<P>>("/StatusNotifierItem")
            .await?;
        let iface = iface_ref.get().await;
        iface.id_changed(iface_ref.signal_emitter()).await
    }

    pub async fn notify_icon_changed(&self) -> zbus::Result<()> {
        let iface_ref = self
            .conn
            .object_server()
            .interface::<_, StatusNotifierInstance<P>>("/StatusNotifierItem")
            .await?;
        StatusNotifierInstance::<P>::new_icon(iface_ref.signal_emitter()).await
    }

    pub async fn notify_layout_changed(&self, revision: u32, parent: i32) -> zbus::Result<()> {
        let iface_ref = self
            .conn
            .object_server()
            .interface::<_, DBusMenuInstance<M>>("/MenuBar")
            .await?;
        let _ = DBusMenuInstance::<M>::layout_updated(iface_ref.signal_emitter(), revision, parent)
            .await;

        Ok(())
    }
}

impl<P: StatusNotifierItem, M: DBusMenuItem> Tray<P, M>
where
    P::State: 'static + Send + Sync,
    P: Send + Sync + 'static,
    M::State: 'static + Send + Sync,
    M: Send + Sync + 'static,
{
    pub async fn run(self) -> zbus::Result<TrayConnection<P, M>> {
        let state = self.notifier_raw.boot();

        let instance = StatusNotifierInstance {
            program: self.notifier_raw,
            state,
        };

        let menu_state = self.menu_raw.boot();
        let instance_menu = DBusMenuInstance {
            program: self.menu_raw,
            state: menu_state,
        };
        let conn = connection::Builder::session()?
            .serve_at("/StatusNotifierItem", instance)?
            .serve_at("/MenuBar", instance_menu)?
            .build()
            .await?;
        let service = conn.unique_name().unwrap().to_string();
        StatusNotifierWatcherProxy::builder(&conn)
            .build()
            .await?
            .register_status_notifier_item(&service)
            .await?;
        Ok(TrayConnection {
            conn,
            _notify_item: PhantomData,
            _menu_item: PhantomData,
        })
    }

    pub fn with_icon_name(
        self,
        f: impl Fn(&P::State) -> zbus::fdo::Result<String>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: with_icon_name(self.notifier_raw, f),
            menu_raw: self.menu_raw,
        }
    }

    pub fn with_scroll(
        self,
        f: impl ScrollFn<P::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: with_scroll(self.notifier_raw, f),
            menu_raw: self.menu_raw,
        }
    }

    pub fn with_secondary_activate(
        self,
        f: impl SecondaryActivateFn<P::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: with_secondary_activate(self.notifier_raw, f),
            menu_raw: self.menu_raw,
        }
    }

    pub fn with_context_menu(
        self,
        f: impl ContextMenuFn<P::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: with_context_menu(self.notifier_raw, f),
            menu_raw: self.menu_raw,
        }
    }

    pub fn with_layout(
        self,
        f: impl GetLayoutFn<M::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: self.notifier_raw,
            menu_raw: with_layout(self.menu_raw, f),
        }
    }
}
fn with_icon_name<P: StatusNotifierItem>(
    program: P,
    icon: impl Fn(&P::State) -> zbus::fdo::Result<String>,
) -> impl StatusNotifierItem<State = P::State>
where
    P::State: 'static + Send + Sync,
{
    struct WithTheme<P, F> {
        program: P,
        icon: F,
    }
    impl<P: StatusNotifierItem, F> StatusNotifierItem for WithTheme<P, F>
    where
        F: Fn(&P::State) -> zbus::fdo::Result<String>,
    {
        type State = P::State;

        fn id(&self) -> String {
            self.program.id()
        }
        fn boot(&self) -> Self::State {
            self.program.boot()
        }
        fn scroll(
            &self,
            state: &mut Self::State,
            delta: i32,
            orientation: &str,
        ) -> zbus::fdo::Result<()> {
            self.program.scroll(state, delta, orientation)
        }
        fn context_menu(&self, state: &mut Self::State, x: i32, y: i32) -> zbus::fdo::Result<()> {
            self.program.context_menu(state, x, y)
        }
        fn activate(&self, state: &mut Self::State, x: i32, y: i32) -> zbus::fdo::Result<()> {
            self.program.activate(state, x, y)
        }
        fn secondary_activate(
            &self,
            state: &mut Self::State,
            x: i32,
            y: i32,
        ) -> zbus::fdo::Result<()> {
            self.program.secondary_activate(state, x, y)
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            (self.icon)(state)
        }
        fn category(&self) -> zbus::fdo::Result<String> {
            self.program.category()
        }
    }
    WithTheme { program, icon }
}

fn with_context_menu<P: StatusNotifierItem>(
    program: P,
    context_menu: impl ContextMenuFn<P::State>,
) -> impl StatusNotifierItem<State = P::State>
where
    P::State: 'static + Send + Sync,
{
    struct WithContextMenu<P, ContextMenuFn> {
        program: P,
        context_menu: ContextMenuFn,
    }
    impl<P: StatusNotifierItem, ContextMenuFn> StatusNotifierItem for WithContextMenu<P, ContextMenuFn>
    where
        ContextMenuFn: self::ContextMenuFn<P::State>,
    {
        type State = P::State;

        fn id(&self) -> String {
            self.program.id()
        }
        fn boot(&self) -> Self::State {
            self.program.boot()
        }
        fn scroll(
            &self,
            state: &mut Self::State,
            delta: i32,
            orientation: &str,
        ) -> zbus::fdo::Result<()> {
            self.program.scroll(state, delta, orientation)
        }
        fn context_menu(&self, state: &mut Self::State, x: i32, y: i32) -> zbus::fdo::Result<()> {
            self.context_menu.context_menu(state, x, y)
        }
        fn activate(&self, state: &mut Self::State, x: i32, y: i32) -> zbus::fdo::Result<()> {
            self.program.activate(state, x, y)
        }
        fn secondary_activate(
            &self,
            state: &mut Self::State,
            x: i32,
            y: i32,
        ) -> zbus::fdo::Result<()> {
            self.program.secondary_activate(state, x, y)
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_name(state)
        }
        fn category(&self) -> zbus::fdo::Result<String> {
            self.program.category()
        }
    }
    WithContextMenu {
        program,
        context_menu,
    }
}

fn with_scroll<P: StatusNotifierItem>(
    program: P,
    scroll: impl ScrollFn<P::State>,
) -> impl StatusNotifierItem<State = P::State>
where
    P::State: 'static + Send + Sync,
{
    struct WithScroll<P, ScrollFn> {
        program: P,
        scroll: ScrollFn,
    }
    impl<P: StatusNotifierItem, ScrollFn> StatusNotifierItem for WithScroll<P, ScrollFn>
    where
        ScrollFn: self::ScrollFn<P::State>,
    {
        type State = P::State;

        fn id(&self) -> String {
            self.program.id()
        }
        fn boot(&self) -> Self::State {
            self.program.boot()
        }
        fn scroll(
            &self,
            state: &mut Self::State,
            delta: i32,
            orientation: &str,
        ) -> zbus::fdo::Result<()> {
            self.scroll.scroll(state, delta, orientation)
        }
        fn context_menu(&self, state: &mut Self::State, x: i32, y: i32) -> zbus::fdo::Result<()> {
            self.program.context_menu(state, x, y)
        }
        fn activate(&self, state: &mut Self::State, x: i32, y: i32) -> zbus::fdo::Result<()> {
            self.program.activate(state, x, y)
        }
        fn secondary_activate(
            &self,
            state: &mut Self::State,
            x: i32,
            y: i32,
        ) -> zbus::fdo::Result<()> {
            self.program.secondary_activate(state, x, y)
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_name(state)
        }
        fn category(&self) -> zbus::fdo::Result<String> {
            self.program.category()
        }
    }
    WithScroll { program, scroll }
}

fn with_secondary_activate<P: StatusNotifierItem>(
    program: P,
    secondary_activate: impl SecondaryActivateFn<P::State>,
) -> impl StatusNotifierItem<State = P::State>
where
    P::State: 'static + Send + Sync,
{
    struct WithSecondaryActive<P, SecondaryActivateFn> {
        program: P,
        secondary_activate: SecondaryActivateFn,
    }
    impl<P: StatusNotifierItem, SecondaryActivateFn> StatusNotifierItem
        for WithSecondaryActive<P, SecondaryActivateFn>
    where
        SecondaryActivateFn: self::SecondaryActivateFn<P::State>,
    {
        type State = P::State;

        fn id(&self) -> String {
            self.program.id()
        }
        fn boot(&self) -> Self::State {
            self.program.boot()
        }
        fn scroll(
            &self,
            state: &mut Self::State,
            delta: i32,
            orientation: &str,
        ) -> zbus::fdo::Result<()> {
            self.program.scroll(state, delta, orientation)
        }
        fn context_menu(&self, state: &mut Self::State, x: i32, y: i32) -> zbus::fdo::Result<()> {
            self.program.context_menu(state, x, y)
        }
        fn activate(&self, state: &mut Self::State, x: i32, y: i32) -> zbus::fdo::Result<()> {
            self.program.activate(state, x, y)
        }
        fn secondary_activate(
            &self,
            state: &mut Self::State,
            x: i32,
            y: i32,
        ) -> zbus::fdo::Result<()> {
            self.secondary_activate.secondary_activate(state, x, y)
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_name(state)
        }
        fn category(&self) -> zbus::fdo::Result<String> {
            self.program.category()
        }
    }
    WithSecondaryActive {
        program,
        secondary_activate,
    }
}

fn with_layout<M: DBusMenuItem>(
    program: M,
    get_layout: impl GetLayoutFn<M::State>,
) -> impl DBusMenuItem<State = M::State> {
    struct WithLayout<M, GetLayout> {
        program: M,
        get_layout: GetLayout,
    }

    impl<M: DBusMenuItem, GetLayout> DBusMenuItem for WithLayout<M, GetLayout>
    where
        GetLayout: self::GetLayoutFn<M::State>,
    {
        type State = M::State;
        fn get_layout(
            &self,
            state: &mut Self::State,
            parent_id: i32,
            recursion_depth: i32,
            property_names: Vec<String>,
        ) -> zbus::fdo::Result<(u32, MenuItem)> {
            self.get_layout
                .get_layout(state, parent_id, recursion_depth, property_names)
        }
        fn boot(&self) -> Self::State {
            self.program.boot()
        }
        fn about_to_show(&self, state: &mut Self::State, id: i32) -> zbus::fdo::Result<bool> {
            self.program.about_to_show(state, id)
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.status(state)
        }
    }
    WithLayout {
        program,
        get_layout,
    }
}

pub fn tray<State, MenuState>(
    boot: impl NotifierBootFn<State>,
    id: impl IdFn,
    activate: impl ActivateFn<State>,
    icon_name: impl Fn(&State) -> zbus::fdo::Result<String>,
    category: &str,

    menu_boot: impl DBusMenuBootFn<MenuState>,
    about_to_show: impl AboutToShowFn<MenuState>,
    status: impl StatusFn<MenuState>,
) -> Tray<impl StatusNotifierItem<State = State>, impl DBusMenuItem<State = MenuState>>
where
    State: 'static + Send + Sync,
{
    use std::marker::PhantomData;
    struct Instance<State, IdFn, IconFn, BootFn, ActivateFn> {
        boot: BootFn,
        id: IdFn,
        icon_name: IconFn,
        category: String,
        activate: ActivateFn,
        _state: PhantomData<State>,
    }
    impl<State, IdFn, IconFn, BootFn, ActivateFn> StatusNotifierItem
        for Instance<State, IdFn, IconFn, BootFn, ActivateFn>
    where
        BootFn: self::NotifierBootFn<State>,
        IdFn: self::IdFn,
        IconFn: Fn(&State) -> zbus::fdo::Result<String>,
        ActivateFn: self::ActivateFn<State>,
    {
        type State = State;
        fn id(&self) -> String {
            self.id.id()
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            (self.icon_name)(state)
        }
        fn category(&self) -> zbus::fdo::Result<String> {
            Ok(self.category.clone())
        }

        fn boot(&self) -> Self::State {
            self.boot.boot()
        }

        fn activate(&self, state: &mut Self::State, x: i32, y: i32) -> zbus::fdo::Result<()> {
            self.activate.activate(state, x, y)
        }
    }
    struct MenuInstance<MenuState, MenuBootFn, AboutToShowFn, StatusFn> {
        boot: MenuBootFn,
        about_to_show: AboutToShowFn,
        status: StatusFn,
        _state: PhantomData<MenuState>,
    }

    impl<MenuState, MenuBootFn, AboutToShowFn, StatusFn> DBusMenuItem
        for MenuInstance<MenuState, MenuBootFn, AboutToShowFn, StatusFn>
    where
        MenuBootFn: self::DBusMenuBootFn<MenuState>,
        AboutToShowFn: self::AboutToShowFn<MenuState>,
        StatusFn: self::StatusFn<MenuState>,
    {
        type State = MenuState;
        fn about_to_show(&self, state: &mut Self::State, id: i32) -> zbus::fdo::Result<bool> {
            self.about_to_show.about_to_show(state, id)
        }
        fn boot(&self) -> Self::State {
            self.boot.boot()
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.status.status(state)
        }
    }
    Tray {
        notifier_raw: Instance {
            boot,
            id,
            icon_name,
            category: category.to_owned(),
            activate,
            _state: PhantomData,
        },
        menu_raw: MenuInstance {
            boot: menu_boot,
            about_to_show,
            status,
            _state: PhantomData,
        },
    }
}
