use std::marker::PhantomData;

use crate::{
    dbusmenu::{
        AboutToShowFn, DBusMenuBootFn, DBusMenuInstance, DBusMenuItem, EventUpdate,
        GetGroupPropertiesFn, GetLayoutFn, MenuItem, MenuStatus, MenuStatusFn, OnClickedFn,
        OnToggledFn,
    },
    status_notifier_item::{
        ActivateFn, CategoryFn, ContextMenuFn, IconNameFn, IdFn, ItemIsMenuFn, NotifierBootFn,
        NotifierStatusFn, ScrollFn, SecondaryActivateFn, StatusNotifierInstance,
        StatusNotifierItem, TitleFn,
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
    pub async fn update_notify_state<F, R>(&self, f: F) -> zbus::Result<R>
    where
        F: Fn(&mut P::State) -> R,
    {
        let iface_ref = self
            .conn
            .object_server()
            .interface::<_, StatusNotifierInstance<P>>("/StatusNotifierItem")
            .await?;
        let mut data = iface_ref.get_mut().await;
        Ok(f(&mut data.state))
    }
    pub async fn update_menu_state<F, R>(&self, f: F) -> zbus::Result<R>
    where
        F: Fn(&mut M::State) -> R,
    {
        let iface_ref = self
            .conn
            .object_server()
            .interface::<_, DBusMenuInstance<M>>("/MenuBar")
            .await?;
        let mut data = iface_ref.get_mut().await;
        Ok(f(&mut data.state))
    }

    pub async fn update_state<F, R>(&self, f: F) -> zbus::Result<R>
    where
        F: Fn(&mut P::State, &mut M::State) -> R,
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
        Ok(f(&mut data.state, &mut menu_data.state))
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
        f: impl IconNameFn<P::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: with_icon_name(self.notifier_raw, f),
            menu_raw: self.menu_raw,
        }
    }

    pub fn with_item_is_menu(
        self,
        f: impl ItemIsMenuFn<P::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: with_item_is_menu(self.notifier_raw, f),
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
    pub fn with_activate(
        self,
        f: impl ActivateFn<P::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: with_activate(self.notifier_raw, f),
            menu_raw: self.menu_raw,
        }
    }
    pub fn with_category(
        self,
        f: impl CategoryFn,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: with_category(self.notifier_raw, f),
            menu_raw: self.menu_raw,
        }
    }

    pub fn with_tray_status(
        self,
        f: impl NotifierStatusFn<P::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: with_tray_status(self.notifier_raw, f),
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
    pub fn with_get_group_properties(
        self,
        f: impl GetGroupPropertiesFn<M::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: self.notifier_raw,
            menu_raw: with_get_group_properties(self.menu_raw, f),
        }
    }

    pub fn with_menu_status(
        self,
        f: impl MenuStatusFn<M::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: self.notifier_raw,
            menu_raw: with_menu_status(self.menu_raw, f),
        }
    }

    pub fn with_on_clicked(
        self,
        f: impl OnClickedFn<M::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: self.notifier_raw,
            menu_raw: with_on_clicked(self.menu_raw, f),
        }
    }

    pub fn with_on_toggled(
        self,
        f: impl OnToggledFn<M::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: self.notifier_raw,
            menu_raw: with_on_toggled(self.menu_raw, f),
        }
    }
}
fn with_item_is_menu<P: StatusNotifierItem>(
    program: P,
    is_menu: impl ItemIsMenuFn<P::State>,
) -> impl StatusNotifierItem<State = P::State> {
    struct WithItemIsMenu<P, F> {
        program: P,
        is_menu: F,
    }
    impl<P: StatusNotifierItem, F> StatusNotifierItem for WithItemIsMenu<P, F>
    where
        F: ItemIsMenuFn<P::State>,
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
            self.program.icon_name(state)
        }
        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.title(state)
        }
        fn category(&self) -> zbus::fdo::Result<String> {
            self.program.category()
        }
        fn status(
            &self,
            state: &Self::State,
        ) -> zbus::fdo::Result<status_notifier_item::NotifierStatus> {
            self.program.status(state)
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.is_menu.item_is_menu(state)
        }
    }
    WithItemIsMenu { program, is_menu }
}

fn with_icon_name<P: StatusNotifierItem>(
    program: P,
    icon: impl IconNameFn<P::State>,
) -> impl StatusNotifierItem<State = P::State> {
    struct WithTheme<P, F> {
        program: P,
        icon: F,
    }
    impl<P: StatusNotifierItem, F> StatusNotifierItem for WithTheme<P, F>
    where
        F: IconNameFn<P::State>,
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
            self.icon.icon_name(state)
        }
        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.title(state)
        }
        fn category(&self) -> zbus::fdo::Result<String> {
            self.program.category()
        }
        fn status(
            &self,
            state: &Self::State,
        ) -> zbus::fdo::Result<status_notifier_item::NotifierStatus> {
            self.program.status(state)
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.program.item_is_menu(state)
        }
    }
    WithTheme { program, icon }
}

fn with_context_menu<P: StatusNotifierItem>(
    program: P,
    context_menu: impl ContextMenuFn<P::State>,
) -> impl StatusNotifierItem<State = P::State> {
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
        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.title(state)
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_name(state)
        }
        fn category(&self) -> zbus::fdo::Result<String> {
            self.program.category()
        }
        fn status(
            &self,
            state: &Self::State,
        ) -> zbus::fdo::Result<status_notifier_item::NotifierStatus> {
            self.program.status(state)
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.program.item_is_menu(state)
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
) -> impl StatusNotifierItem<State = P::State> {
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
        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.title(state)
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_name(state)
        }
        fn category(&self) -> zbus::fdo::Result<String> {
            self.program.category()
        }
        fn status(
            &self,
            state: &Self::State,
        ) -> zbus::fdo::Result<status_notifier_item::NotifierStatus> {
            self.program.status(state)
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.program.item_is_menu(state)
        }
    }
    WithScroll { program, scroll }
}

fn with_category<P: StatusNotifierItem>(
    program: P,
    category: impl CategoryFn,
) -> impl StatusNotifierItem<State = P::State> {
    struct WithCategory<P, CategoryFn> {
        program: P,
        category: CategoryFn,
    }

    impl<P: StatusNotifierItem, CategoryFn> StatusNotifierItem for WithCategory<P, CategoryFn>
    where
        CategoryFn: self::CategoryFn,
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
            self.program.icon_name(state)
        }
        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.title(state)
        }
        fn category(&self) -> zbus::fdo::Result<String> {
            Ok(self.category.category())
        }
        fn status(
            &self,
            state: &Self::State,
        ) -> zbus::fdo::Result<status_notifier_item::NotifierStatus> {
            self.program.status(state)
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.program.item_is_menu(state)
        }
    }
    WithCategory { program, category }
}

fn with_activate<P: StatusNotifierItem>(
    program: P,
    activate: impl ActivateFn<P::State>,
) -> impl StatusNotifierItem<State = P::State> {
    struct WithActive<P, ActivateFn> {
        program: P,
        activate: ActivateFn,
    }
    impl<P: StatusNotifierItem, ActivateFn> StatusNotifierItem for WithActive<P, ActivateFn>
    where
        ActivateFn: self::ActivateFn<P::State>,
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
            self.activate.activate(state, x, y)
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
        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.title(state)
        }
        fn category(&self) -> zbus::fdo::Result<String> {
            self.program.category()
        }
        fn status(
            &self,
            state: &Self::State,
        ) -> zbus::fdo::Result<status_notifier_item::NotifierStatus> {
            self.program.status(state)
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.program.item_is_menu(state)
        }
    }
    WithActive { program, activate }
}

fn with_secondary_activate<P: StatusNotifierItem>(
    program: P,
    secondary_activate: impl SecondaryActivateFn<P::State>,
) -> impl StatusNotifierItem<State = P::State> {
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
        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.title(state)
        }
        fn category(&self) -> zbus::fdo::Result<String> {
            self.program.category()
        }
        fn status(
            &self,
            state: &Self::State,
        ) -> zbus::fdo::Result<status_notifier_item::NotifierStatus> {
            self.program.status(state)
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.program.item_is_menu(state)
        }
    }
    WithSecondaryActive {
        program,
        secondary_activate,
    }
}

fn with_tray_status<P: StatusNotifierItem>(
    program: P,
    status: impl NotifierStatusFn<P::State>,
) -> impl StatusNotifierItem<State = P::State> {
    struct WithStatus<P, StatusFn> {
        program: P,
        status: StatusFn,
    }
    impl<P: StatusNotifierItem, StatusFn> StatusNotifierItem for WithStatus<P, StatusFn>
    where
        StatusFn: self::NotifierStatusFn<P::State>,
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
            self.program.icon_name(state)
        }
        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.title(state)
        }
        fn category(&self) -> zbus::fdo::Result<String> {
            self.program.category()
        }
        fn status(
            &self,
            state: &Self::State,
        ) -> zbus::fdo::Result<status_notifier_item::NotifierStatus> {
            Ok(self.status.status(state))
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.program.item_is_menu(state)
        }
    }
    WithStatus { program, status }
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
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<MenuStatus> {
            self.program.status(state)
        }
        fn get_group_properties(
            &self,
            state: &mut Self::State,
            ids: Vec<i32>,
            property_names: Vec<String>,
        ) -> zbus::fdo::Result<Vec<dbusmenu::PropertyItem>> {
            self.program
                .get_group_properties(state, ids, property_names)
        }
        fn on_clicked(&self, state: &mut Self::State, id: i32, timestamp: u32) -> EventUpdate {
            self.program.on_clicked(state, id, timestamp)
        }
        fn on_toggled(
            &self,
            state: &mut Self::State,
            id: i32,
            status: dbusmenu::ToggleState,
            timestamp: u32,
        ) -> EventUpdate {
            self.program.on_toggled(state, id, status, timestamp)
        }
    }
    WithLayout {
        program,
        get_layout,
    }
}

fn with_get_group_properties<M: DBusMenuItem>(
    program: M,
    get_group_properties: impl GetGroupPropertiesFn<M::State>,
) -> impl DBusMenuItem<State = M::State> {
    struct WithGetGroupProperties<M, GetGroupProperties> {
        program: M,
        get_group_properties: GetGroupProperties,
    }

    impl<M: DBusMenuItem, GetGroupProperties> DBusMenuItem
        for WithGetGroupProperties<M, GetGroupProperties>
    where
        GetGroupProperties: self::GetGroupPropertiesFn<M::State>,
    {
        type State = M::State;
        fn get_layout(
            &self,
            state: &mut Self::State,
            parent_id: i32,
            recursion_depth: i32,
            property_names: Vec<String>,
        ) -> zbus::fdo::Result<(u32, MenuItem)> {
            self.program
                .get_layout(state, parent_id, recursion_depth, property_names)
        }
        fn boot(&self) -> Self::State {
            self.program.boot()
        }
        fn about_to_show(&self, state: &mut Self::State, id: i32) -> zbus::fdo::Result<bool> {
            self.program.about_to_show(state, id)
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<MenuStatus> {
            self.program.status(state)
        }
        fn get_group_properties(
            &self,
            state: &mut Self::State,
            ids: Vec<i32>,
            property_names: Vec<String>,
        ) -> zbus::fdo::Result<Vec<dbusmenu::PropertyItem>> {
            self.get_group_properties
                .get_group_properties(state, ids, property_names)
        }
        fn on_clicked(&self, state: &mut Self::State, id: i32, timestamp: u32) -> EventUpdate {
            self.program.on_clicked(state, id, timestamp)
        }
        fn on_toggled(
            &self,
            state: &mut Self::State,
            id: i32,
            status: dbusmenu::ToggleState,
            timestamp: u32,
        ) -> EventUpdate {
            self.program.on_toggled(state, id, status, timestamp)
        }
    }
    WithGetGroupProperties {
        program,
        get_group_properties,
    }
}

fn with_menu_status<M: DBusMenuItem>(
    program: M,
    menu_status: impl MenuStatusFn<M::State>,
) -> impl DBusMenuItem<State = M::State> {
    struct WithMenuStatus<M, MenuStatusFn> {
        program: M,
        menu_status: MenuStatusFn,
    }

    impl<M: DBusMenuItem, MenuStatusFn> DBusMenuItem for WithMenuStatus<M, MenuStatusFn>
    where
        MenuStatusFn: self::MenuStatusFn<M::State>,
    {
        type State = M::State;
        fn get_layout(
            &self,
            state: &mut Self::State,
            parent_id: i32,
            recursion_depth: i32,
            property_names: Vec<String>,
        ) -> zbus::fdo::Result<(u32, MenuItem)> {
            self.program
                .get_layout(state, parent_id, recursion_depth, property_names)
        }
        fn boot(&self) -> Self::State {
            self.program.boot()
        }
        fn about_to_show(&self, state: &mut Self::State, id: i32) -> zbus::fdo::Result<bool> {
            self.program.about_to_show(state, id)
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<MenuStatus> {
            Ok(self.menu_status.status(state))
        }
        fn get_group_properties(
            &self,
            state: &mut Self::State,
            ids: Vec<i32>,
            property_names: Vec<String>,
        ) -> zbus::fdo::Result<Vec<dbusmenu::PropertyItem>> {
            self.program
                .get_group_properties(state, ids, property_names)
        }
        fn on_clicked(&self, state: &mut Self::State, id: i32, timestamp: u32) -> EventUpdate {
            self.program.on_clicked(state, id, timestamp)
        }
        fn on_toggled(
            &self,
            state: &mut Self::State,
            id: i32,
            status: dbusmenu::ToggleState,
            timestamp: u32,
        ) -> EventUpdate {
            self.program.on_toggled(state, id, status, timestamp)
        }
    }
    WithMenuStatus {
        program,
        menu_status,
    }
}
fn with_on_clicked<M: DBusMenuItem>(
    program: M,
    on_clicked: impl OnClickedFn<M::State>,
) -> impl DBusMenuItem<State = M::State> {
    struct WithOnClicked<M, OnClickedFn> {
        program: M,
        on_clicked: OnClickedFn,
    }

    impl<M: DBusMenuItem, OnClickedFn> DBusMenuItem for WithOnClicked<M, OnClickedFn>
    where
        OnClickedFn: self::OnClickedFn<M::State>,
    {
        type State = M::State;
        fn get_layout(
            &self,
            state: &mut Self::State,
            parent_id: i32,
            recursion_depth: i32,
            property_names: Vec<String>,
        ) -> zbus::fdo::Result<(u32, MenuItem)> {
            self.program
                .get_layout(state, parent_id, recursion_depth, property_names)
        }
        fn boot(&self) -> Self::State {
            self.program.boot()
        }
        fn about_to_show(&self, state: &mut Self::State, id: i32) -> zbus::fdo::Result<bool> {
            self.program.about_to_show(state, id)
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<MenuStatus> {
            self.program.status(state)
        }
        fn get_group_properties(
            &self,
            state: &mut Self::State,
            ids: Vec<i32>,
            property_names: Vec<String>,
        ) -> zbus::fdo::Result<Vec<dbusmenu::PropertyItem>> {
            self.program
                .get_group_properties(state, ids, property_names)
        }
        fn on_clicked(&self, state: &mut Self::State, id: i32, timestamp: u32) -> EventUpdate {
            self.on_clicked.on_clicked(state, id, timestamp)
        }
        fn on_toggled(
            &self,
            state: &mut Self::State,
            id: i32,
            status: dbusmenu::ToggleState,
            timestamp: u32,
        ) -> EventUpdate {
            self.program.on_toggled(state, id, status, timestamp)
        }
    }
    WithOnClicked {
        program,
        on_clicked,
    }
}

fn with_on_toggled<M: DBusMenuItem>(
    program: M,
    on_toggled: impl OnToggledFn<M::State>,
) -> impl DBusMenuItem<State = M::State> {
    struct WithOnToggled<M, OnToggledFn> {
        program: M,
        on_toggled: OnToggledFn,
    }

    impl<M: DBusMenuItem, OnToggledFn> DBusMenuItem for WithOnToggled<M, OnToggledFn>
    where
        OnToggledFn: self::OnToggledFn<M::State>,
    {
        type State = M::State;
        fn get_layout(
            &self,
            state: &mut Self::State,
            parent_id: i32,
            recursion_depth: i32,
            property_names: Vec<String>,
        ) -> zbus::fdo::Result<(u32, MenuItem)> {
            self.program
                .get_layout(state, parent_id, recursion_depth, property_names)
        }
        fn boot(&self) -> Self::State {
            self.program.boot()
        }
        fn about_to_show(&self, state: &mut Self::State, id: i32) -> zbus::fdo::Result<bool> {
            self.program.about_to_show(state, id)
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<MenuStatus> {
            self.program.status(state)
        }
        fn get_group_properties(
            &self,
            state: &mut Self::State,
            ids: Vec<i32>,
            property_names: Vec<String>,
        ) -> zbus::fdo::Result<Vec<dbusmenu::PropertyItem>> {
            self.program
                .get_group_properties(state, ids, property_names)
        }
        fn on_clicked(&self, state: &mut Self::State, id: i32, timestamp: u32) -> EventUpdate {
            self.program.on_clicked(state, id, timestamp)
        }
        fn on_toggled(
            &self,
            state: &mut Self::State,
            id: i32,
            status: dbusmenu::ToggleState,
            timestamp: u32,
        ) -> EventUpdate {
            self.on_toggled.on_toggled(state, id, status, timestamp)
        }
    }
    WithOnToggled {
        program,
        on_toggled,
    }
}

pub fn tray<State, MenuState>(
    boot: impl NotifierBootFn<State>,
    id: impl IdFn,
    title: impl TitleFn<State>,

    menu_boot: impl DBusMenuBootFn<MenuState>,
    about_to_show: impl AboutToShowFn<MenuState>,
) -> Tray<impl StatusNotifierItem<State = State>, impl DBusMenuItem<State = MenuState>>
where
    State: 'static + Send + Sync,
{
    use std::marker::PhantomData;
    struct Instance<State, IdFn, TitleFn, BootFn> {
        boot: BootFn,
        id: IdFn,
        title: TitleFn,
        _state: PhantomData<State>,
    }
    impl<State, IdFn, TitleFn, BootFn> StatusNotifierItem for Instance<State, IdFn, TitleFn, BootFn>
    where
        BootFn: self::NotifierBootFn<State>,
        IdFn: self::IdFn,
        TitleFn: self::TitleFn<State>,
    {
        type State = State;
        fn id(&self) -> String {
            self.id.id()
        }
        fn boot(&self) -> Self::State {
            self.boot.boot()
        }

        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.title.title(state)
        }
    }
    struct MenuInstance<MenuState, MenuBootFn, AboutToShowFn> {
        boot: MenuBootFn,
        about_to_show: AboutToShowFn,
        _state: PhantomData<MenuState>,
    }

    impl<MenuState, MenuBootFn, AboutToShowFn> DBusMenuItem
        for MenuInstance<MenuState, MenuBootFn, AboutToShowFn>
    where
        MenuBootFn: self::DBusMenuBootFn<MenuState>,
        AboutToShowFn: self::AboutToShowFn<MenuState>,
    {
        type State = MenuState;
        fn about_to_show(&self, state: &mut Self::State, id: i32) -> zbus::fdo::Result<bool> {
            self.about_to_show.about_to_show(state, id)
        }
        fn boot(&self) -> Self::State {
            self.boot.boot()
        }
    }
    Tray {
        notifier_raw: Instance {
            boot,
            id,
            title,
            _state: PhantomData,
        },
        menu_raw: MenuInstance {
            boot: menu_boot,
            about_to_show,
            _state: PhantomData,
        },
    }
}
