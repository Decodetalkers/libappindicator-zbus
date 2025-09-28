use crate::{
    dbusmenu::{
        AboutToShowFn, AboutToShowGroupFn, DBusMenuBootFn, DBusMenuInstance, DBusMenuItem,
        EventUpdate, IconThemePathFn, MenuFn, MenuStatus, MenuStatusFn, MenuUnit, OnClickedFn,
        OnToggledFn, RevisionFn, TextDirectionFn, ToggleState,
    },
    status_notifier_item::{
        ActivateFn, AttentionIconNameFn, AttentionIconPixmapFn, AttentionMovieNameFn, CategoryFn,
        ContextMenuFn, IconNameFn, IconPixmapFn, IconThemePathNotifierFn, IdFn, ItemIsMenuFn,
        NotifierBootFn, NotifierStatus, NotifierStatusFn, OverlayIconNameFn, OverlayIconPixmapFn,
        ScrollFn, SecondaryActivateFn, StatusNotifierInstance, StatusNotifierItem, TitleFn,
        ToolTipFn, WindowIdFn,
    },
    status_notifier_watcher::StatusNotifierWatcherProxy,
    utils::{Category, IconPixmap, TextDirection, ToolTip},
};
use std::marker::PhantomData;

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
    pub fn with_tool_tip(
        self,
        f: impl ToolTipFn<P::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: with_tool_tip(self.notifier_raw, f),
            menu_raw: self.menu_raw,
        }
    }
    pub fn with_tray_icon_theme_path(
        self,
        f: impl IconThemePathNotifierFn<P::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: with_tray_icon_theme_path(self.notifier_raw, f),
            menu_raw: self.menu_raw,
        }
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
    pub fn with_icon_pixmap(
        self,
        f: impl IconPixmapFn<P::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: with_icon_pixmap(self.notifier_raw, f),
            menu_raw: self.menu_raw,
        }
    }
    pub fn with_attention_icon_name(
        self,
        f: impl AttentionIconNameFn<P::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: with_attention_icon_name(self.notifier_raw, f),
            menu_raw: self.menu_raw,
        }
    }
    pub fn with_attention_icon_pixmap(
        self,
        f: impl AttentionIconPixmapFn<P::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: with_attention_icon_pixmap(self.notifier_raw, f),
            menu_raw: self.menu_raw,
        }
    }
    pub fn with_attention_movie_name(
        self,
        f: impl AttentionMovieNameFn<P::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: with_attention_movie_name(self.notifier_raw, f),
            menu_raw: self.menu_raw,
        }
    }
    pub fn with_overlay_icon_name(
        self,
        f: impl OverlayIconNameFn<P::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: with_overlay_icon_name(self.notifier_raw, f),
            menu_raw: self.menu_raw,
        }
    }
    pub fn with_overlay_icon_pixmap(
        self,
        f: impl OverlayIconPixmapFn<P::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: with_overlay_icon_pixmap(self.notifier_raw, f),
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
    pub fn with_window_id(
        self,
        f: impl WindowIdFn<P::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: with_window_id(self.notifier_raw, f),
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

    pub fn with_text_direction(
        self,
        f: impl TextDirectionFn<M::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: self.notifier_raw,
            menu_raw: with_text_direction(self.menu_raw, f),
        }
    }

    pub fn with_menu_icon_theme_path(
        self,
        f: impl IconThemePathFn<M::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: self.notifier_raw,
            menu_raw: with_menu_icon_theme_path(self.menu_raw, f),
        }
    }
    pub fn with_about_to_show(
        self,
        f: impl AboutToShowFn<M::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: self.notifier_raw,
            menu_raw: with_about_to_show(self.menu_raw, f),
        }
    }
    pub fn with_about_to_show_group(
        self,
        f: impl AboutToShowGroupFn<M::State>,
    ) -> Tray<impl StatusNotifierItem<State = P::State>, impl DBusMenuItem<State = M::State>> {
        Tray {
            notifier_raw: self.notifier_raw,
            menu_raw: with_about_to_show_group(self.menu_raw, f),
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
        fn tool_tip(&self, state: &Self::State) -> zbus::fdo::Result<ToolTip> {
            self.program.tool_tip(state)
        }
        fn icon_theme_path(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_theme_path(state)
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_name(state)
        }
        fn icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.icon_pixmap(state)
        }

        fn attention_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_icon_name(state)
        }
        fn attention_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.attention_icon_pixmap(state)
        }
        fn attention_movie_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_movie_name(state)
        }
        fn overlay_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.overlay_icon_name(state)
        }
        fn overlay_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.overlay_icon_pixmap(state)
        }
        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.title(state)
        }
        fn category(&self) -> Category {
            self.program.category()
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<NotifierStatus> {
            self.program.status(state)
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.is_menu.item_is_menu(state)
        }
        fn window_id(&self, state: &Self::State) -> zbus::fdo::Result<i32> {
            self.program.window_id(state)
        }
    }
    WithItemIsMenu { program, is_menu }
}
fn with_tool_tip<P: StatusNotifierItem>(
    program: P,
    tool_tip: impl ToolTipFn<P::State>,
) -> impl StatusNotifierItem<State = P::State> {
    struct WithToolTip<P, F> {
        program: P,
        tool_tip: F,
    }
    impl<P: StatusNotifierItem, F> StatusNotifierItem for WithToolTip<P, F>
    where
        F: ToolTipFn<P::State>,
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
        fn tool_tip(&self, state: &Self::State) -> zbus::fdo::Result<ToolTip> {
            self.tool_tip.tool_tip(state)
        }
        fn icon_theme_path(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_theme_path(state)
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_name(state)
        }
        fn icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.icon_pixmap(state)
        }
        fn attention_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_icon_name(state)
        }
        fn attention_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.attention_icon_pixmap(state)
        }
        fn attention_movie_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_movie_name(state)
        }
        fn overlay_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.overlay_icon_name(state)
        }
        fn overlay_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.overlay_icon_pixmap(state)
        }
        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.title(state)
        }
        fn category(&self) -> Category {
            self.program.category()
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<NotifierStatus> {
            self.program.status(state)
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.program.item_is_menu(state)
        }
        fn window_id(&self, state: &Self::State) -> zbus::fdo::Result<i32> {
            self.program.window_id(state)
        }
    }
    WithToolTip { program, tool_tip }
}
fn with_tray_icon_theme_path<P: StatusNotifierItem>(
    program: P,
    theme_path: impl IconThemePathNotifierFn<P::State>,
) -> impl StatusNotifierItem<State = P::State> {
    struct WithIconThemePath<P, F> {
        program: P,
        theme_path: F,
    }
    impl<P: StatusNotifierItem, F> StatusNotifierItem for WithIconThemePath<P, F>
    where
        F: IconThemePathNotifierFn<P::State>,
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
        fn tool_tip(&self, state: &Self::State) -> zbus::fdo::Result<ToolTip> {
            self.program.tool_tip(state)
        }
        fn icon_theme_path(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.theme_path.icon_theme_path(state)
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_name(state)
        }
        fn icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.icon_pixmap(state)
        }
        fn attention_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_icon_name(state)
        }
        fn attention_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.attention_icon_pixmap(state)
        }
        fn attention_movie_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_movie_name(state)
        }
        fn overlay_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.overlay_icon_name(state)
        }
        fn overlay_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.overlay_icon_pixmap(state)
        }
        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.title(state)
        }
        fn category(&self) -> Category {
            self.program.category()
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<NotifierStatus> {
            self.program.status(state)
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.program.item_is_menu(state)
        }
        fn window_id(&self, state: &Self::State) -> zbus::fdo::Result<i32> {
            self.program.window_id(state)
        }
    }
    WithIconThemePath {
        program,
        theme_path,
    }
}

fn with_icon_name<P: StatusNotifierItem>(
    program: P,
    icon: impl IconNameFn<P::State>,
) -> impl StatusNotifierItem<State = P::State> {
    struct WithIconName<P, F> {
        program: P,
        icon: F,
    }
    impl<P: StatusNotifierItem, F> StatusNotifierItem for WithIconName<P, F>
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
        fn tool_tip(&self, state: &Self::State) -> zbus::fdo::Result<ToolTip> {
            self.program.tool_tip(state)
        }
        fn icon_theme_path(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_theme_path(state)
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.icon.icon_name(state)
        }
        fn icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.icon_pixmap(state)
        }
        fn attention_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_icon_name(state)
        }
        fn attention_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.attention_icon_pixmap(state)
        }
        fn attention_movie_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_movie_name(state)
        }
        fn overlay_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.overlay_icon_name(state)
        }
        fn overlay_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.overlay_icon_pixmap(state)
        }
        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.title(state)
        }
        fn category(&self) -> Category {
            self.program.category()
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<NotifierStatus> {
            self.program.status(state)
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.program.item_is_menu(state)
        }
        fn window_id(&self, state: &Self::State) -> zbus::fdo::Result<i32> {
            self.program.window_id(state)
        }
    }
    WithIconName { program, icon }
}

fn with_icon_pixmap<P: StatusNotifierItem>(
    program: P,
    icon_pixmap: impl IconPixmapFn<P::State>,
) -> impl StatusNotifierItem<State = P::State> {
    struct WithIconPixmap<P, IconPixmapFn> {
        program: P,
        icon_pixmap: IconPixmapFn,
    }
    impl<P: StatusNotifierItem, IconPixmapFn> StatusNotifierItem for WithIconPixmap<P, IconPixmapFn>
    where
        IconPixmapFn: self::IconPixmapFn<P::State>,
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
        fn tool_tip(&self, state: &Self::State) -> zbus::fdo::Result<ToolTip> {
            self.program.tool_tip(state)
        }
        fn icon_theme_path(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_theme_path(state)
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_name(state)
        }
        fn icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.icon_pixmap.icon_pixmap(state)
        }
        fn overlay_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.overlay_icon_pixmap(state)
        }
        fn attention_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_icon_name(state)
        }
        fn attention_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.attention_icon_pixmap(state)
        }

        fn attention_movie_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_movie_name(state)
        }
        fn overlay_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.overlay_icon_name(state)
        }
        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.title(state)
        }
        fn category(&self) -> Category {
            self.program.category()
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<NotifierStatus> {
            self.program.status(state)
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.program.item_is_menu(state)
        }
        fn window_id(&self, state: &Self::State) -> zbus::fdo::Result<i32> {
            self.program.window_id(state)
        }
    }
    WithIconPixmap {
        program,
        icon_pixmap,
    }
}

fn with_attention_icon_name<P: StatusNotifierItem>(
    program: P,
    icon: impl AttentionIconNameFn<P::State>,
) -> impl StatusNotifierItem<State = P::State> {
    struct WithAttentionIconName<P, F> {
        program: P,
        icon: F,
    }
    impl<P: StatusNotifierItem, F> StatusNotifierItem for WithAttentionIconName<P, F>
    where
        F: AttentionIconNameFn<P::State>,
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
        fn tool_tip(&self, state: &Self::State) -> zbus::fdo::Result<ToolTip> {
            self.program.tool_tip(state)
        }
        fn icon_theme_path(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_theme_path(state)
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_name(state)
        }
        fn icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.icon_pixmap(state)
        }

        fn attention_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.icon.attention_icon_name(state)
        }

        fn attention_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.attention_icon_pixmap(state)
        }
        fn attention_movie_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_movie_name(state)
        }
        fn overlay_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.overlay_icon_name(state)
        }
        fn overlay_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.overlay_icon_pixmap(state)
        }
        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.title(state)
        }
        fn category(&self) -> Category {
            self.program.category()
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<NotifierStatus> {
            self.program.status(state)
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.program.item_is_menu(state)
        }
        fn window_id(&self, state: &Self::State) -> zbus::fdo::Result<i32> {
            self.program.window_id(state)
        }
    }
    WithAttentionIconName { program, icon }
}

fn with_attention_icon_pixmap<P: StatusNotifierItem>(
    program: P,
    pixmaps: impl AttentionIconPixmapFn<P::State>,
) -> impl StatusNotifierItem<State = P::State> {
    struct WithAttentionPixmap<P, F> {
        program: P,
        pixmaps: F,
    }
    impl<P: StatusNotifierItem, F> StatusNotifierItem for WithAttentionPixmap<P, F>
    where
        F: AttentionIconPixmapFn<P::State>,
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
        fn tool_tip(&self, state: &Self::State) -> zbus::fdo::Result<ToolTip> {
            self.program.tool_tip(state)
        }
        fn icon_theme_path(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_theme_path(state)
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_name(state)
        }
        fn icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.icon_pixmap(state)
        }
        fn attention_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_name(state)
        }

        fn attention_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.pixmaps.attention_icon_pixmap(state)
        }
        fn attention_movie_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_movie_name(state)
        }
        fn overlay_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.overlay_icon_name(state)
        }
        fn overlay_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.overlay_icon_pixmap(state)
        }
        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.title(state)
        }
        fn category(&self) -> Category {
            self.program.category()
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<NotifierStatus> {
            self.program.status(state)
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.program.item_is_menu(state)
        }
        fn window_id(&self, state: &Self::State) -> zbus::fdo::Result<i32> {
            self.program.window_id(state)
        }
    }
    WithAttentionPixmap { program, pixmaps }
}

fn with_attention_movie_name<P: StatusNotifierItem>(
    program: P,
    movie_name: impl AttentionMovieNameFn<P::State>,
) -> impl StatusNotifierItem<State = P::State> {
    struct WithAttentionMovieName<P, F> {
        program: P,
        movie_name: F,
    }
    impl<P: StatusNotifierItem, F> StatusNotifierItem for WithAttentionMovieName<P, F>
    where
        F: AttentionMovieNameFn<P::State>,
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
        fn tool_tip(&self, state: &Self::State) -> zbus::fdo::Result<ToolTip> {
            self.program.tool_tip(state)
        }
        fn icon_theme_path(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_theme_path(state)
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_name(state)
        }
        fn icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.icon_pixmap(state)
        }

        fn attention_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_icon_name(state)
        }

        fn attention_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.attention_icon_pixmap(state)
        }
        fn attention_movie_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.movie_name.attention_movie_name(state)
        }
        fn overlay_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.overlay_icon_name(state)
        }
        fn overlay_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.overlay_icon_pixmap(state)
        }
        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.title(state)
        }
        fn category(&self) -> Category {
            self.program.category()
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<NotifierStatus> {
            self.program.status(state)
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.program.item_is_menu(state)
        }
        fn window_id(&self, state: &Self::State) -> zbus::fdo::Result<i32> {
            self.program.window_id(state)
        }
    }
    WithAttentionMovieName {
        program,
        movie_name,
    }
}

fn with_overlay_icon_name<P: StatusNotifierItem>(
    program: P,
    icon: impl OverlayIconNameFn<P::State>,
) -> impl StatusNotifierItem<State = P::State> {
    struct WithOverlayIconName<P, F> {
        program: P,
        icon: F,
    }
    impl<P: StatusNotifierItem, F> StatusNotifierItem for WithOverlayIconName<P, F>
    where
        F: OverlayIconNameFn<P::State>,
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
        fn tool_tip(&self, state: &Self::State) -> zbus::fdo::Result<ToolTip> {
            self.program.tool_tip(state)
        }
        fn icon_theme_path(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_theme_path(state)
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_name(state)
        }
        fn icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.icon_pixmap(state)
        }
        fn attention_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_icon_name(state)
        }

        fn attention_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.attention_icon_pixmap(state)
        }
        fn attention_movie_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_movie_name(state)
        }
        fn overlay_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.icon.overlay_icon_name(state)
        }
        fn overlay_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.overlay_icon_pixmap(state)
        }
        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.title(state)
        }
        fn category(&self) -> Category {
            self.program.category()
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<NotifierStatus> {
            self.program.status(state)
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.program.item_is_menu(state)
        }
        fn window_id(&self, state: &Self::State) -> zbus::fdo::Result<i32> {
            self.program.window_id(state)
        }
    }
    WithOverlayIconName { program, icon }
}

fn with_overlay_icon_pixmap<P: StatusNotifierItem>(
    program: P,
    pixmaps: impl OverlayIconPixmapFn<P::State>,
) -> impl StatusNotifierItem<State = P::State> {
    struct WithOverlayIconPixmap<P, F> {
        program: P,
        pixmaps: F,
    }
    impl<P: StatusNotifierItem, F> StatusNotifierItem for WithOverlayIconPixmap<P, F>
    where
        F: OverlayIconPixmapFn<P::State>,
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
        fn tool_tip(&self, state: &Self::State) -> zbus::fdo::Result<ToolTip> {
            self.program.tool_tip(state)
        }
        fn icon_theme_path(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_theme_path(state)
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_name(state)
        }
        fn icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.icon_pixmap(state)
        }
        fn attention_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_name(state)
        }

        fn attention_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.attention_icon_pixmap(state)
        }
        fn attention_movie_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_movie_name(state)
        }
        fn overlay_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.overlay_icon_name(state)
        }
        fn overlay_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.pixmaps.overlay_icon_pixmap(state)
        }
        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.title(state)
        }
        fn category(&self) -> Category {
            self.program.category()
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<NotifierStatus> {
            self.program.status(state)
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.program.item_is_menu(state)
        }
        fn window_id(&self, state: &Self::State) -> zbus::fdo::Result<i32> {
            self.program.window_id(state)
        }
    }
    WithOverlayIconPixmap { program, pixmaps }
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
        fn tool_tip(&self, state: &Self::State) -> zbus::fdo::Result<ToolTip> {
            self.program.tool_tip(state)
        }
        fn icon_theme_path(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_theme_path(state)
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_name(state)
        }
        fn icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.icon_pixmap(state)
        }
        fn attention_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_icon_name(state)
        }
        fn attention_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.attention_icon_pixmap(state)
        }
        fn attention_movie_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_movie_name(state)
        }
        fn overlay_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.overlay_icon_name(state)
        }
        fn overlay_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.overlay_icon_pixmap(state)
        }
        fn category(&self) -> Category {
            self.program.category()
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<NotifierStatus> {
            self.program.status(state)
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.program.item_is_menu(state)
        }
        fn window_id(&self, state: &Self::State) -> zbus::fdo::Result<i32> {
            self.program.window_id(state)
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
        fn tool_tip(&self, state: &Self::State) -> zbus::fdo::Result<ToolTip> {
            self.program.tool_tip(state)
        }
        fn icon_theme_path(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_theme_path(state)
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_name(state)
        }
        fn icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.icon_pixmap(state)
        }
        fn attention_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_icon_name(state)
        }
        fn attention_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.attention_icon_pixmap(state)
        }
        fn attention_movie_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_movie_name(state)
        }
        fn overlay_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.overlay_icon_name(state)
        }
        fn overlay_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.overlay_icon_pixmap(state)
        }
        fn category(&self) -> Category {
            self.program.category()
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<NotifierStatus> {
            self.program.status(state)
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.program.item_is_menu(state)
        }
        fn window_id(&self, state: &Self::State) -> zbus::fdo::Result<i32> {
            self.program.window_id(state)
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
        fn tool_tip(&self, state: &Self::State) -> zbus::fdo::Result<ToolTip> {
            self.program.tool_tip(state)
        }
        fn icon_theme_path(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_theme_path(state)
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_name(state)
        }
        fn icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.icon_pixmap(state)
        }
        fn attention_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_icon_name(state)
        }
        fn attention_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.attention_icon_pixmap(state)
        }
        fn attention_movie_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_movie_name(state)
        }
        fn overlay_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.overlay_icon_name(state)
        }
        fn overlay_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.overlay_icon_pixmap(state)
        }
        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.title(state)
        }
        fn category(&self) -> Category {
            self.category.category()
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<NotifierStatus> {
            self.program.status(state)
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.program.item_is_menu(state)
        }
        fn window_id(&self, state: &Self::State) -> zbus::fdo::Result<i32> {
            self.program.window_id(state)
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
        fn tool_tip(&self, state: &Self::State) -> zbus::fdo::Result<ToolTip> {
            self.program.tool_tip(state)
        }
        fn icon_theme_path(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_theme_path(state)
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_name(state)
        }
        fn icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.icon_pixmap(state)
        }
        fn attention_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_icon_name(state)
        }
        fn attention_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.attention_icon_pixmap(state)
        }
        fn attention_movie_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_movie_name(state)
        }
        fn overlay_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.overlay_icon_name(state)
        }
        fn overlay_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.overlay_icon_pixmap(state)
        }
        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.title(state)
        }
        fn category(&self) -> Category {
            self.program.category()
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<NotifierStatus> {
            self.program.status(state)
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.program.item_is_menu(state)
        }
        fn window_id(&self, state: &Self::State) -> zbus::fdo::Result<i32> {
            self.program.window_id(state)
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
        fn tool_tip(&self, state: &Self::State) -> zbus::fdo::Result<ToolTip> {
            self.program.tool_tip(state)
        }
        fn icon_theme_path(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_theme_path(state)
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_name(state)
        }
        fn icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.icon_pixmap(state)
        }
        fn attention_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_icon_name(state)
        }
        fn attention_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.attention_icon_pixmap(state)
        }
        fn attention_movie_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_movie_name(state)
        }
        fn overlay_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.overlay_icon_name(state)
        }
        fn overlay_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.overlay_icon_pixmap(state)
        }
        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.title(state)
        }
        fn category(&self) -> Category {
            self.program.category()
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<NotifierStatus> {
            self.program.status(state)
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.program.item_is_menu(state)
        }
        fn window_id(&self, state: &Self::State) -> zbus::fdo::Result<i32> {
            self.program.window_id(state)
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
        fn tool_tip(&self, state: &Self::State) -> zbus::fdo::Result<ToolTip> {
            self.program.tool_tip(state)
        }
        fn icon_theme_path(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_theme_path(state)
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_name(state)
        }
        fn icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.icon_pixmap(state)
        }
        fn attention_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_icon_name(state)
        }
        fn attention_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.attention_icon_pixmap(state)
        }
        fn attention_movie_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_movie_name(state)
        }
        fn overlay_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.overlay_icon_name(state)
        }
        fn overlay_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.overlay_icon_pixmap(state)
        }
        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.title(state)
        }
        fn category(&self) -> Category {
            self.program.category()
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<NotifierStatus> {
            Ok(self.status.status(state))
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.program.item_is_menu(state)
        }
        fn window_id(&self, state: &Self::State) -> zbus::fdo::Result<i32> {
            self.program.window_id(state)
        }
    }
    WithStatus { program, status }
}
fn with_window_id<P: StatusNotifierItem>(
    program: P,
    window_id: impl WindowIdFn<P::State>,
) -> impl StatusNotifierItem<State = P::State> {
    struct WithWindowId<P, F> {
        program: P,
        window_id: F,
    }
    impl<P: StatusNotifierItem, F> StatusNotifierItem for WithWindowId<P, F>
    where
        F: WindowIdFn<P::State>,
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
        fn tool_tip(&self, state: &Self::State) -> zbus::fdo::Result<ToolTip> {
            self.program.tool_tip(state)
        }
        fn icon_theme_path(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_theme_path(state)
        }
        fn icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.icon_name(state)
        }
        fn icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.icon_pixmap(state)
        }
        fn attention_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_icon_name(state)
        }

        fn attention_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.attention_icon_pixmap(state)
        }
        fn attention_movie_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.attention_movie_name(state)
        }
        fn overlay_icon_name(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.overlay_icon_name(state)
        }
        fn overlay_icon_pixmap(&self, state: &Self::State) -> zbus::fdo::Result<Vec<IconPixmap>> {
            self.program.overlay_icon_pixmap(state)
        }
        fn title(&self, state: &Self::State) -> zbus::fdo::Result<String> {
            self.program.title(state)
        }
        fn category(&self) -> Category {
            self.program.category()
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<NotifierStatus> {
            self.program.status(state)
        }
        fn item_is_menu(&self, state: &Self::State) -> bool {
            self.program.item_is_menu(state)
        }
        fn window_id(&self, state: &Self::State) -> zbus::fdo::Result<i32> {
            Ok(self.window_id.window_id(state))
        }
    }
    WithWindowId { program, window_id }
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
        type Message = M::Message;

        fn boot(&self) -> Self::State {
            self.program.boot()
        }
        fn revision(&self, state: &Self::State) -> u32 {
            self.program.revision(state)
        }
        fn menu(&self, state: &Self::State) -> MenuUnit<M::Message> {
            self.program.menu(state)
        }
        fn about_to_show(&self, state: &mut Self::State, id: i32) -> zbus::fdo::Result<bool> {
            self.program.about_to_show(state, id)
        }
        fn about_to_show_group(
            &self,
            state: &mut Self::State,
            ids: Vec<i32>,
        ) -> zbus::fdo::Result<(Vec<i32>, Vec<i32>)> {
            self.program.about_to_show_group(state, ids)
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<MenuStatus> {
            Ok(self.menu_status.status(state))
        }

        fn on_clicked(&self, state: &mut Self::State, id: i32, timestamp: u32) -> EventUpdate {
            self.program.on_clicked(state, id, timestamp)
        }
        fn on_toggled(
            &self,
            state: &mut Self::State,
            id: i32,
            status: ToggleState,
            timestamp: u32,
        ) -> EventUpdate {
            self.program.on_toggled(state, id, status, timestamp)
        }
        fn text_direction(&self, state: &Self::State) -> TextDirection {
            self.program.text_direction(state)
        }
        fn icon_theme_path(&self, state: &Self::State) -> Vec<String> {
            self.program.icon_theme_path(state)
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
        type Message = M::Message;

        fn boot(&self) -> Self::State {
            self.program.boot()
        }
        fn revision(&self, state: &Self::State) -> u32 {
            self.program.revision(state)
        }
        fn menu(&self, state: &Self::State) -> MenuUnit<M::Message> {
            self.program.menu(state)
        }
        fn about_to_show(&self, state: &mut Self::State, id: i32) -> zbus::fdo::Result<bool> {
            self.program.about_to_show(state, id)
        }
        fn about_to_show_group(
            &self,
            state: &mut Self::State,
            ids: Vec<i32>,
        ) -> zbus::fdo::Result<(Vec<i32>, Vec<i32>)> {
            self.program.about_to_show_group(state, ids)
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<MenuStatus> {
            self.program.status(state)
        }

        fn on_clicked(&self, state: &mut Self::State, id: i32, timestamp: u32) -> EventUpdate {
            self.on_clicked.on_clicked(state, id, timestamp)
        }
        fn on_toggled(
            &self,
            state: &mut Self::State,
            id: i32,
            status: ToggleState,
            timestamp: u32,
        ) -> EventUpdate {
            self.program.on_toggled(state, id, status, timestamp)
        }
        fn text_direction(&self, state: &Self::State) -> TextDirection {
            self.program.text_direction(state)
        }
        fn icon_theme_path(&self, state: &Self::State) -> Vec<String> {
            self.program.icon_theme_path(state)
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
) -> impl DBusMenuItem<State = M::State, Message = M::Message> {
    struct WithOnToggled<M, OnToggledFn> {
        program: M,
        on_toggled: OnToggledFn,
    }

    impl<M: DBusMenuItem, OnToggledFn> DBusMenuItem for WithOnToggled<M, OnToggledFn>
    where
        OnToggledFn: self::OnToggledFn<M::State>,
    {
        type State = M::State;
        type Message = M::Message;

        fn boot(&self) -> Self::State {
            self.program.boot()
        }
        fn revision(&self, state: &Self::State) -> u32 {
            self.program.revision(state)
        }
        fn menu(&self, state: &Self::State) -> MenuUnit<M::Message> {
            self.program.menu(state)
        }
        fn about_to_show(&self, state: &mut Self::State, id: i32) -> zbus::fdo::Result<bool> {
            self.program.about_to_show(state, id)
        }
        fn about_to_show_group(
            &self,
            state: &mut Self::State,
            ids: Vec<i32>,
        ) -> zbus::fdo::Result<(Vec<i32>, Vec<i32>)> {
            self.program.about_to_show_group(state, ids)
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<MenuStatus> {
            self.program.status(state)
        }

        fn on_clicked(&self, state: &mut Self::State, id: i32, timestamp: u32) -> EventUpdate {
            self.program.on_clicked(state, id, timestamp)
        }
        fn on_toggled(
            &self,
            state: &mut Self::State,
            id: i32,
            status: ToggleState,
            timestamp: u32,
        ) -> EventUpdate {
            self.on_toggled.on_toggled(state, id, status, timestamp)
        }
        fn icon_theme_path(&self, state: &Self::State) -> Vec<String> {
            self.program.icon_theme_path(state)
        }
    }
    WithOnToggled {
        program,
        on_toggled,
    }
}

fn with_text_direction<M: DBusMenuItem>(
    program: M,
    text_direction: impl TextDirectionFn<M::State>,
) -> impl DBusMenuItem<State = M::State, Message = M::Message> {
    struct WithTextDirection<M, TextDirectionFn> {
        program: M,
        text_direction: TextDirectionFn,
    }

    impl<M: DBusMenuItem, TextDirectionFn> DBusMenuItem for WithTextDirection<M, TextDirectionFn>
    where
        TextDirectionFn: self::TextDirectionFn<M::State>,
    {
        type State = M::State;
        type Message = M::Message;

        fn boot(&self) -> Self::State {
            self.program.boot()
        }
        fn revision(&self, state: &Self::State) -> u32 {
            self.program.revision(state)
        }
        fn menu(&self, state: &Self::State) -> MenuUnit<M::Message> {
            self.program.menu(state)
        }
        fn about_to_show(&self, state: &mut Self::State, id: i32) -> zbus::fdo::Result<bool> {
            self.program.about_to_show(state, id)
        }
        fn about_to_show_group(
            &self,
            state: &mut Self::State,
            ids: Vec<i32>,
        ) -> zbus::fdo::Result<(Vec<i32>, Vec<i32>)> {
            self.program.about_to_show_group(state, ids)
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<MenuStatus> {
            self.program.status(state)
        }

        fn on_clicked(&self, state: &mut Self::State, id: i32, timestamp: u32) -> EventUpdate {
            self.program.on_clicked(state, id, timestamp)
        }
        fn on_toggled(
            &self,
            state: &mut Self::State,
            id: i32,
            status: ToggleState,
            timestamp: u32,
        ) -> EventUpdate {
            self.program.on_toggled(state, id, status, timestamp)
        }
        fn text_direction(&self, state: &Self::State) -> TextDirection {
            self.text_direction.text_direction(state)
        }
        fn icon_theme_path(&self, state: &Self::State) -> Vec<String> {
            self.program.icon_theme_path(state)
        }
    }
    WithTextDirection {
        program,
        text_direction,
    }
}

fn with_menu_icon_theme_path<M: DBusMenuItem>(
    program: M,
    icon_theme_path: impl IconThemePathFn<M::State>,
) -> impl DBusMenuItem<State = M::State, Message = M::Message> {
    struct WithIconThemePath<M, IconThemePathFn> {
        program: M,
        icon_theme_path: IconThemePathFn,
    }

    impl<M: DBusMenuItem, IconThemePathFn> DBusMenuItem for WithIconThemePath<M, IconThemePathFn>
    where
        IconThemePathFn: self::IconThemePathFn<M::State>,
    {
        type State = M::State;
        type Message = M::Message;

        fn boot(&self) -> Self::State {
            self.program.boot()
        }
        fn revision(&self, state: &Self::State) -> u32 {
            self.program.revision(state)
        }
        fn menu(&self, state: &Self::State) -> MenuUnit<M::Message> {
            self.program.menu(state)
        }
        fn about_to_show(&self, state: &mut Self::State, id: i32) -> zbus::fdo::Result<bool> {
            self.program.about_to_show(state, id)
        }
        fn about_to_show_group(
            &self,
            state: &mut Self::State,
            ids: Vec<i32>,
        ) -> zbus::fdo::Result<(Vec<i32>, Vec<i32>)> {
            self.program.about_to_show_group(state, ids)
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<MenuStatus> {
            self.program.status(state)
        }

        fn on_clicked(&self, state: &mut Self::State, id: i32, timestamp: u32) -> EventUpdate {
            self.program.on_clicked(state, id, timestamp)
        }
        fn on_toggled(
            &self,
            state: &mut Self::State,
            id: i32,
            status: ToggleState,
            timestamp: u32,
        ) -> EventUpdate {
            self.program.on_toggled(state, id, status, timestamp)
        }
        fn text_direction(&self, state: &Self::State) -> TextDirection {
            self.program.text_direction(state)
        }
        fn icon_theme_path(&self, state: &Self::State) -> Vec<String> {
            self.icon_theme_path.icon_theme_path(state)
        }
    }
    WithIconThemePath {
        program,
        icon_theme_path,
    }
}
fn with_about_to_show<M: DBusMenuItem>(
    program: M,
    about_to_show: impl AboutToShowFn<M::State>,
) -> impl DBusMenuItem<State = M::State, Message = M::Message> {
    struct WithAboutToShow<M, AboutToShowFn> {
        program: M,
        about_to_show: AboutToShowFn,
    }

    impl<M: DBusMenuItem, AboutToShowFn> DBusMenuItem for WithAboutToShow<M, AboutToShowFn>
    where
        AboutToShowFn: self::AboutToShowFn<M::State>,
    {
        type State = M::State;
        type Message = M::Message;

        fn boot(&self) -> Self::State {
            self.program.boot()
        }
        fn revision(&self, state: &Self::State) -> u32 {
            self.program.revision(state)
        }
        fn menu(&self, state: &Self::State) -> MenuUnit<M::Message> {
            self.program.menu(state)
        }
        fn about_to_show(&self, state: &mut Self::State, id: i32) -> zbus::fdo::Result<bool> {
            Ok(self.about_to_show.about_to_show(state, id))
        }
        fn about_to_show_group(
            &self,
            state: &mut Self::State,
            ids: Vec<i32>,
        ) -> zbus::fdo::Result<(Vec<i32>, Vec<i32>)> {
            self.program.about_to_show_group(state, ids)
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<MenuStatus> {
            self.program.status(state)
        }

        fn on_clicked(&self, state: &mut Self::State, id: i32, timestamp: u32) -> EventUpdate {
            self.program.on_clicked(state, id, timestamp)
        }
        fn on_toggled(
            &self,
            state: &mut Self::State,
            id: i32,
            status: ToggleState,
            timestamp: u32,
        ) -> EventUpdate {
            self.program.on_toggled(state, id, status, timestamp)
        }
        fn text_direction(&self, state: &Self::State) -> TextDirection {
            self.program.text_direction(state)
        }
        fn icon_theme_path(&self, state: &Self::State) -> Vec<String> {
            self.program.icon_theme_path(state)
        }
    }
    WithAboutToShow {
        program,
        about_to_show,
    }
}
fn with_about_to_show_group<M: DBusMenuItem>(
    program: M,
    about_to_show_group: impl AboutToShowGroupFn<M::State>,
) -> impl DBusMenuItem<State = M::State, Message = M::Message> {
    struct WithAboutToShowGroup<M, AboutToShowGroupFn> {
        program: M,
        about_to_show_group: AboutToShowGroupFn,
    }

    impl<M: DBusMenuItem, AboutToShowGroupFn> DBusMenuItem
        for WithAboutToShowGroup<M, AboutToShowGroupFn>
    where
        AboutToShowGroupFn: self::AboutToShowGroupFn<M::State>,
    {
        type State = M::State;
        type Message = M::Message;

        fn boot(&self) -> Self::State {
            self.program.boot()
        }
        fn revision(&self, state: &Self::State) -> u32 {
            self.program.revision(state)
        }
        fn menu(&self, state: &Self::State) -> MenuUnit<M::Message> {
            self.program.menu(state)
        }
        fn about_to_show(&self, state: &mut Self::State, id: i32) -> zbus::fdo::Result<bool> {
            self.program.about_to_show(state, id)
        }
        fn about_to_show_group(
            &self,
            state: &mut Self::State,
            ids: Vec<i32>,
        ) -> zbus::fdo::Result<(Vec<i32>, Vec<i32>)> {
            self.about_to_show_group.about_to_show_group(state, ids)
        }
        fn status(&self, state: &Self::State) -> zbus::fdo::Result<MenuStatus> {
            self.program.status(state)
        }

        fn on_clicked(&self, state: &mut Self::State, id: i32, timestamp: u32) -> EventUpdate {
            self.program.on_clicked(state, id, timestamp)
        }
        fn on_toggled(
            &self,
            state: &mut Self::State,
            id: i32,
            status: ToggleState,
            timestamp: u32,
        ) -> EventUpdate {
            self.program.on_toggled(state, id, status, timestamp)
        }
        fn text_direction(&self, state: &Self::State) -> TextDirection {
            self.program.text_direction(state)
        }
        fn icon_theme_path(&self, state: &Self::State) -> Vec<String> {
            self.program.icon_theme_path(state)
        }
    }
    WithAboutToShowGroup {
        program,
        about_to_show_group,
    }
}

// NOTE: main function
pub fn tray<State, MenuState, Message>(
    boot: impl NotifierBootFn<State>,
    id: impl IdFn,
    title: impl TitleFn<State>,

    menu_boot: impl DBusMenuBootFn<MenuState>,
    menu: impl MenuFn<MenuState, Message>,
    revision: impl RevisionFn<MenuState>,
) -> Tray<
    impl StatusNotifierItem<State = State>,
    impl DBusMenuItem<State = MenuState, Message = Message>,
>
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
    struct MenuInstance<MenuState, Message, MenuBootFn, MenuFn, RevisionFn> {
        boot: MenuBootFn,
        menu: MenuFn,
        revision: RevisionFn,
        _state: PhantomData<MenuState>,
        _message: PhantomData<Message>,
    }

    impl<MenuState, Message, MenuBootFn, MenuFn, RevisionFn> DBusMenuItem
        for MenuInstance<MenuState, Message, MenuBootFn, MenuFn, RevisionFn>
    where
        MenuBootFn: self::DBusMenuBootFn<MenuState>,
        MenuFn: self::MenuFn<MenuState, Message>,
        RevisionFn: self::RevisionFn<MenuState>,
    {
        type State = MenuState;
        type Message = Message;

        fn boot(&self) -> Self::State {
            self.boot.boot()
        }
        fn revision(&self, state: &Self::State) -> u32 {
            self.revision.revision(state)
        }
        fn menu(&self, state: &Self::State) -> MenuUnit<Message> {
            self.menu.menu(state)
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
            menu,
            revision,
            _state: PhantomData,
            _message: PhantomData,
        },
    }
}
