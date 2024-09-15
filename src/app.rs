use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, KeyEvent};
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicBool, Arc};
use tracing::{debug, error, info};

use ratatui::{prelude::*, widgets::*};

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIs};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::action::Action;
use crate::config;
use crate::events::{Event, Events};
use crate::tui::Tui;

#[derive(
    Default, Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIs,
)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Common,
    #[default]
    Summary,
    PickerShowCrateInfo,
    PickerHideCrateInfo,
    Search,
    Filter,
    Popup,
    Help,
    Quit,
}

struct AppWidget;

#[derive(Debug)]
pub struct App {
    /// Receiver end of an asynchronous channel for actions that the app needs
    /// to process.
    rx: UnboundedReceiver<Action>,

    /// Sender end of an asynchronous channel for dispatching actions from
    /// various parts of the app to be handled by the event loop.
    tx: UnboundedSender<Action>,

    /// A thread-safe indicator of whether data is currently being loaded,
    /// allowing different parts of the app to know if it's in a loading state.
    loading_status: Arc<AtomicBool>,

    /// The active mode of the application, which could change how user inputs
    /// and commands are interpreted.
    mode: Mode,

    /// The active mode of the application, which could change how user inputs
    /// and commands are interpreted.
    last_mode: Mode,

    /// A list of key events that have been held since the last tick, useful for
    /// interpreting sequences of key presses.
    last_tick_key_events: Vec<KeyEvent>,

    /// frame counter
    frame_count: usize,
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let loading_status = Arc::new(AtomicBool::default());
        Self {
            rx,
            tx,
            mode: Mode::default(),
            last_mode: Mode::default(),
            loading_status,
            last_tick_key_events: Default::default(),
            frame_count: Default::default(),
        }
    }

    /// Runs the main loop of the application, handling events and actions
    pub async fn run(&mut self, mut tui: Tui, mut events: Events) -> Result<()> {
        // uncomment to test error handling
        // panic!("test panic");
        // Err(color_eyre::eyre::eyre!("Error"))?;
        self.tx.send(Action::Init)?;

        loop {
            if let Some(e) = events.next().await {
                self.handle_event(e)?.map(|action| self.tx.send(action));
            }
            while let Ok(action) = self.rx.try_recv() {
                self.handle_action(action.clone())?;
                if matches!(action, Action::Resize(_, _) | Action::Render) {
                    self.draw(&mut tui)?;
                }
            }
            if self.should_quit() {
                break;
            }
        }
        Ok(())
    }

    /// Handles an event by producing an optional `Action` that the application
    /// should perform in response.
    ///
    /// This method maps incoming events from the terminal user interface to
    /// specific `Action` that represents tasks or operations the
    /// application needs to carry out.
    fn handle_event(&mut self, e: Event) -> Result<Option<Action>> {
        let maybe_action = match e {
            Event::Quit => Some(Action::Quit),
            Event::Tick => Some(Action::Tick),
            Event::KeyRefresh => Some(Action::KeyRefresh),
            Event::Render => Some(Action::Render),
            Event::Crossterm(CrosstermEvent::Resize(x, y)) => Some(Action::Resize(x, y)),
            Event::Crossterm(CrosstermEvent::Key(key)) => self.handle_key_event(key)?,
            _ => None,
        };
        Ok(maybe_action)
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        debug!("Received key {:?}", key);
        match self.mode {
            Mode::Search => {}
            Mode::Filter => {}
            _ => (),
        };
        Ok(self.handle_key_events_from_config(key))
    }

    /// Evaluates a sequence of key events against user-configured key bindings
    /// to determine if an `Action` should be triggered.
    ///
    /// This method supports user-configurable key sequences by collecting key
    /// events over time and then translating them into actions according to the
    /// current mode.
    fn handle_key_events_from_config(&mut self, key: KeyEvent) -> Option<Action> {
        self.last_tick_key_events.push(key);
        let config = config::get();
        config
            .key_bindings
            .event_to_command(self.mode, &self.last_tick_key_events)
            .or_else(|| {
                config
                    .key_bindings
                    .event_to_command(Mode::Common, &self.last_tick_key_events)
            })
            .map(|command| config.key_bindings.command_to_action(command))
    }

    /// Performs the `Action` by calling on a respective app method.
    ///
    /// Upon receiving an action, this function updates the application state, performs necessary
    /// operations like drawing or resizing the view, or changing the mode. Actions that affect the
    /// navigation within the application, are also handled. Certain actions generate a follow-up
    /// action which will be to be processed in the next iteration of the main event loop.
    fn handle_action(&mut self, action: Action) -> Result<()> {
        if action != Action::Tick && action != Action::Render && action != Action::KeyRefresh {
            info!("{action:?}");
        }
        match action {
            Action::Quit => self.quit(),
            Action::KeyRefresh => self.key_refresh_tick(),
            Action::Init => self.init()?,
            Action::Tick => self.tick(),

            Action::NextTab => self.goto_next_tab(),
            Action::PreviousTab => self.goto_previous_tab(),
            Action::SwitchMode(mode) => self.switch_mode(mode),
            Action::SwitchToLastMode => self.switch_to_last_mode(),
            Action::ShowErrorPopup(ref err) => self.show_error_popup(err.clone()),
            Action::ShowInfoPopup(ref info) => self.show_info_popup(info.clone()),
            Action::ClosePopup => self.close_popup(),
            _ => {}
        };
        Ok(())
    }

    // Render the `AppWidget` as a stateful widget using `self` as the `State`
    fn draw(&mut self, tui: &mut Tui) -> Result<()> {
        tui.draw(|frame| {
            frame.render_stateful_widget(AppWidget, frame.area(), self);
            self.update_frame_count(frame);
            self.update_cursor(frame);
        })?;
        Ok(())
    }

    fn tick(&mut self) {
        //self.search.update_search_table_results();
    }

    fn init(&mut self) -> Result<()> {
        //self.summary.request()?;
        Ok(())
    }

    fn key_refresh_tick(&mut self) {
        self.last_tick_key_events.drain(..);
    }

    fn should_quit(&self) -> bool {
        self.mode == Mode::Quit
    }

    fn quit(&mut self) {
        self.mode = Mode::Quit
    }

    fn switch_mode(&mut self, mode: Mode) {
        self.last_mode = self.mode;
        self.mode = mode;
        match self.mode {
            Mode::Search => {
                //self.selected_tab.select(SelectedTab::Search);
                //self.search.enter_search_insert_mode();
            }
            Mode::Filter => {
                //self.selected_tab.select(SelectedTab::Search);
                //self.search.enter_filter_insert_mode();
            }
            Mode::Summary => {
                //self.search.enter_normal_mode();
                //self.selected_tab.select(SelectedTab::Summary);
            }
            Mode::Help => {
                //self.search.enter_normal_mode();
                //self.help.mode = Some(self.last_mode);
                //self.selected_tab.select(SelectedTab::None)
            }
            Mode::PickerShowCrateInfo | Mode::PickerHideCrateInfo => {
                //self.search.enter_normal_mode();
                //self.selected_tab.select(SelectedTab::Search)
            }
            _ => {
                //self.search.enter_normal_mode();
                //self.selected_tab.select(SelectedTab::None)
            }
        }
    }

    fn switch_to_last_mode(&mut self) {
        self.switch_mode(self.last_mode);
    }

    fn goto_next_tab(&mut self) {
        match self.mode {
            Mode::Summary => self.switch_mode(Mode::Search),
            Mode::Search => self.switch_mode(Mode::Summary),
            _ => self.switch_mode(Mode::Summary),
        }
    }

    fn goto_previous_tab(&mut self) {
        match self.mode {
            Mode::Summary => self.switch_mode(Mode::Search),
            Mode::Search => self.switch_mode(Mode::Summary),
            _ => self.switch_mode(Mode::Summary),
        }
    }

    fn show_error_popup(&mut self, message: String) {
        error!("Error: {message}");
        //self.popup = Some((
        //    PopupMessageWidget::new("Error".into(), message),
        //    PopupMessageState::default(),
        //));
        self.switch_mode(Mode::Popup);
    }

    fn show_info_popup(&mut self, info: String) {
        info!("Info: {info}");
        //self.popup = Some((
        //    PopupMessageWidget::new("Info".into(), info),
        //    PopupMessageState::default(),
        //));
        self.switch_mode(Mode::Popup);
    }

    fn close_popup(&mut self) {
        //self.popup = None;
        if self.last_mode.is_popup() {
            self.switch_mode(Mode::Search);
        } else {
            self.switch_mode(self.last_mode);
        }
    }

    // Sets the frame count
    fn update_frame_count(&mut self, frame: &mut Frame<'_>) {
        self.frame_count = frame.count();
    }

    // Sets cursor for the prompt
    fn update_cursor(&mut self, frame: &mut Frame<'_>) {
        //if self.mode.is_prompt() {
        //    if let Some(cursor_position) = self.search.cursor_position() {
        //        frame.set_cursor_position(cursor_position);
        //    }
        //}
    }

    fn loading(&self) -> bool {
        self.loading_status.load(Ordering::SeqCst)
    }
}

impl StatefulWidget for AppWidget {
    type State = App;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Background color
        Block::default()
            //.bg(config::get().color.base00)
            .render(area, buf);

        use Constraint::*;
        let [header, main] = Layout::vertical([Length(1), Fill(1)]).areas(area);
        let [tabs, events] = Layout::horizontal([Min(15), Fill(1)]).areas(header);

        state.render_tabs(tabs, buf);
        //state.events_widget().render(events, buf);

        let mode = if matches!(state.mode, Mode::Popup | Mode::Quit) {
            state.last_mode
        } else {
            state.mode
        };
        match mode {
            Mode::Summary => {}
            Mode::Help => {}
            Mode::Search => {}
            Mode::Filter => {}
            Mode::PickerShowCrateInfo => {}
            Mode::PickerHideCrateInfo => {}

            Mode::Common => {}
            Mode::Popup => {}
            Mode::Quit => {}
        };

        if state.loading() {
            Line::from(state.spinner())
                .right_aligned()
                .render(main, buf);
        }

        //if let Some((popup, popup_state)) = &mut state.popup {
        //    popup.render(area, buf, popup_state);
        //}
    }
}

impl App {
    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        use strum::IntoEnumIterator;
        //let titles = SelectedTab::iter().map(|tab| tab.title());
        //let highlight_style = SelectedTab::highlight_style();

        //let selected_tab_index = self.selected_tab as usize;
        //Tabs::new(titles)
        //    .highlight_style(highlight_style)
        //    .select(selected_tab_index)
        //    .padding("", "")
        //    .divider(" ")
        //    .render(area, buf);
    }

    //fn render_summary(&mut self, area: Rect, buf: &mut Buffer) {
    //    let [main, status_bar] =
    //        Layout::vertical([Constraint::Fill(0), Constraint::Length(1)]).areas(area);
    //    SummaryWidget.render(main, buf, &mut self.summary);
    //    self.render_status_bar(status_bar, buf);
    //}

    //fn render_help(&mut self, area: Rect, buf: &mut Buffer) {
    //    let [main, status_bar] =
    //        Layout::vertical([Constraint::Fill(0), Constraint::Length(1)]).areas(area);
    //    HelpWidget.render(main, buf, &mut self.help);
    //    self.render_status_bar(status_bar, buf);
    //}
    //
    //fn render_search(&mut self, area: Rect, buf: &mut Buffer) {
    //    let prompt_height = if self.mode.is_prompt() && self.search.is_prompt() {
    //        5
    //    } else {
    //        0
    //    };
    //    let [main, prompt, status_bar] = Layout::vertical([
    //        Constraint::Min(0),
    //        Constraint::Length(prompt_height),
    //        Constraint::Length(1),
    //    ])
    //    .areas(area);
    //
    //    SearchPageWidget.render(main, buf, &mut self.search);
    //
    //    self.render_prompt(prompt, buf);
    //    self.render_status_bar(status_bar, buf);
    //}
    //
    //fn render_prompt(&mut self, area: Rect, buf: &mut Buffer) {
    //    let p = SearchFilterPromptWidget::new(
    //        self.mode,
    //        self.search.sort.clone(),
    //        &self.search.input,
    //        self.search.search_mode,
    //    );
    //    p.render(area, buf, &mut self.search.prompt);
    //}
    //
    //fn render_status_bar(&mut self, area: Rect, buf: &mut Buffer) {
    //    let s = StatusBarWidget::new(
    //        self.mode,
    //        self.search.sort.clone(),
    //        self.search.input.value().to_string(),
    //    );
    //    s.render(area, buf);
    //}

    fn spinner(&self) -> String {
        let spinner = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let index = self.frame_count % spinner.len();
        let symbol = spinner[index];
        symbol.into()
    }
}
