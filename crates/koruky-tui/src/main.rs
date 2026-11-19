use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use koruky_core::{fetch_trending, search_anime, Anime};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use ratatui_image::{picker::Picker, protocol::Protocol, Image, Resize};
use std::time::Duration;
use std::{
    error::Error,
    fs,
    io::{self, stdout},
    path::PathBuf,
    process::Command,
};
use url::Url;

const MENU: [&str; 5] = ["Discover", "Search", "Watchlist", "Settings", "Quit"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Browser {
    System,
    Zen,
    Firefox,
    Chromium,
}

impl Browser {
    fn label(self) -> &'static str {
        match self {
            Self::System => "System default",
            Self::Zen => "Zen Browser",
            Self::Firefox => "Firefox",
            Self::Chromium => "Chromium",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::System => Self::Zen,
            Self::Zen => Self::Firefox,
            Self::Firefox => Self::Chromium,
            Self::Chromium => Self::System,
        }
    }

    fn config_value(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Zen => "zen",
            Self::Firefox => "firefox",
            Self::Chromium => "chromium",
        }
    }

    fn from_config(value: &str) -> Self {
        match value.trim() {
            "zen" => Self::Zen,
            "firefox" => Self::Firefox,
            "chromium" => Self::Chromium,
            _ => Self::System,
        }
    }

    fn open(self, target: &str) -> Result<(), String> {
        if self == Self::System {
            return open::that(target).map_err(|e| e.to_string());
        }
        let candidates: &[&str] = match self {
            Self::Zen => &["zen", "zen-browser"],
            Self::Firefox => &["firefox"],
            Self::Chromium => &["chromium", "chromium-browser"],
            Self::System => unreachable!(),
        };
        let mut last_error = String::from("Browser command not found");
        for program in candidates {
            match Command::new(program).arg(target).spawn() {
                Ok(_) => return Ok(()),
                Err(error) => last_error = error.to_string(),
            }
        }
        if self == Self::Zen {
            match Command::new("flatpak")
                .args(["run", "app.zen_browser.zen", target])
                .spawn()
            {
                Ok(_) => return Ok(()),
                Err(error) => last_error = error.to_string(),
            }
        }
        Err(format!("Could not start {}: {}", self.label(), last_error))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Theme {
    Dark,
    Snow,
    Midnight,
    Sakura,
    Mono,
}

impl Theme {
    fn label(self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Snow => "Snow",
            Self::Midnight => "Midnight",
            Self::Sakura => "Sakura",
            Self::Mono => "Mono",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Dark => Self::Snow,
            Self::Snow => Self::Midnight,
            Self::Midnight => Self::Sakura,
            Self::Sakura => Self::Mono,
            Self::Mono => Self::Dark,
        }
    }

    fn config_value(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Snow => "snow",
            Self::Midnight => "midnight",
            Self::Sakura => "sakura",
            Self::Mono => "mono",
        }
    }

    fn from_config(value: &str) -> Self {
        match value.trim() {
            "snow" => Self::Snow,
            "midnight" => Self::Midnight,
            "sakura" => Self::Sakura,
            "mono" => Self::Mono,
            _ => Self::Dark,
        }
    }

    fn background(self) -> Color {
        match self {
            Self::Dark => Color::Rgb(30, 30, 34),
            Self::Snow => Color::Rgb(220, 234, 245),
            Self::Midnight => Color::Rgb(23, 23, 39),
            Self::Sakura => Color::Rgb(41, 30, 37),
            Self::Mono => Color::Black,
        }
    }

    fn foreground(self) -> Color {
        match self {
            Self::Dark => Color::Rgb(235, 235, 235),
            Self::Snow => Color::Rgb(35, 55, 75),
            Self::Midnight => Color::Rgb(205, 200, 255),
            Self::Sakura => Color::Rgb(244, 197, 216),
            Self::Mono => Color::White,
        }
    }

    fn muted(self) -> Color {
        match self {
            Self::Dark => Color::Gray,
            Self::Snow => Color::Rgb(90, 110, 130),
            Self::Midnight => Color::Rgb(150, 145, 190),
            Self::Sakura => Color::Rgb(190, 145, 165),
            Self::Mono => Color::DarkGray,
        }
    }

    fn base_style(self) -> Style {
        Style::default().bg(self.background()).fg(self.foreground())
    }
}

fn settings_path() -> Option<PathBuf> {
    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(config_home).join("koruky/settings.conf"));
    }
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config/koruky/settings.conf"))
}

fn read_settings() -> String {
    settings_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .unwrap_or_default()
}

fn load_browser() -> Browser {
    let contents = read_settings();
    let value = contents
        .lines()
        .find_map(|line| line.strip_prefix("browser="))
        .unwrap_or("system");
    Browser::from_config(value)
}

fn load_theme() -> Theme {
    let contents = read_settings();
    let value = contents
        .lines()
        .find_map(|line| line.strip_prefix("theme="))
        .unwrap_or("dark");
    Theme::from_config(value)
}

fn save_settings(browser: Browser, theme: Theme) -> Result<(), String> {
    let path = settings_path().ok_or_else(|| "Could not find config directory".to_string())?;
    let parent = path
        .parent()
        .ok_or_else(|| "Invalid settings path".to_string())?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    fs::write(
        path,
        format!(
            "browser={}\ntheme={}\n",
            browser.config_value(),
            theme.config_value()
        ),
    )
    .map_err(|e| e.to_string())
}

struct App {
    menu_selected: usize,
    anime_selected: usize,
    nav_focus: bool,
    active: usize,

    trending: Vec<Anime>,
    search_results: Vec<Anime>,
    search_query: String,
    search_editing: bool,

    provider_selecting: bool,
    browser: Browser,
    theme: Theme,
    picker: Picker,
    cover: Option<Protocol>,
    cover_id: Option<u32>,
    cover_error: Option<String>,

    error: Option<String>,
    status_message: Option<String>,
    running: bool,
}

impl App {
    fn new(picker: Picker) -> Self {
        let mut app = Self {
            menu_selected: 0,
            anime_selected: 0,
            nav_focus: true,
            active: 0,

            trending: Vec::new(),
            search_results: Vec::new(),
            search_query: String::new(),
            search_editing: false,

            provider_selecting: false,
            browser: load_browser(),
            theme: load_theme(),
            picker,
            cover: None,
            cover_id: None,
            cover_error: None,

            error: None,
            status_message: None,
            running: true,
        };

        match fetch_trending() {
            Ok(anime) => app.trending = anime,
            Err(error) => app.error = Some(format!("AniList: {error}")),
        }

        app.refresh_cover();
        app
    }

    fn current_anime(&self) -> &[Anime] {
        match self.active {
            0 => &self.trending,
            1 => &self.search_results,
            _ => &[],
        }
    }

    fn selected_anime(&self) -> Option<&Anime> {
        self.current_anime().get(self.anime_selected)
    }

    fn refresh_cover(&mut self) {
        let selected = self.selected_anime().and_then(|anime| {
            anime
                .cover_image
                .large
                .as_ref()
                .map(|url| (anime.id, url.clone()))
        });

        let Some((id, url)) = selected else {
            self.cover = None;
            self.cover_id = None;
            self.cover_error = None;
            return;
        };

        if self.cover_id == Some(id) && self.cover.is_some() {
            return;
        }

        self.cover_id = Some(id);
        self.cover = None;
        self.cover_error = None;

        match load_cover(&self.picker, &url) {
            Ok(protocol) => self.cover = Some(protocol),
            Err(error) => self.cover_error = Some(error),
        }
    }

    fn run_search(&mut self) {
        let query = self.search_query.trim().to_string();

        if query.is_empty() {
            self.error = Some("Please enter an anime title.".to_string());
            return;
        }

        self.error = None;
        self.status_message = Some(format!("Searching for {query}..."));

        match search_anime(&query) {
            Ok(results) => {
                let count = results.len();
                self.search_results = results;
                self.anime_selected = 0;
                self.status_message = Some(format!("{count} results found."));
                self.refresh_cover();
            }
            Err(error) => {
                self.search_results.clear();
                self.anime_selected = 0;
                self.cover = None;
                self.cover_id = None;
                self.error = Some(format!("Search failed: {error}"));
                self.status_message = None;
            }
        }
    }

    fn open_provider(&mut self, provider: &str) {
        let Some(anime) = self.selected_anime() else {
            self.provider_selecting = false;
            return;
        };

        let title = anime.display_title().to_string();

        let base_url = match provider {
            "crunchyroll" => "https://www.crunchyroll.com/search",
            "netflix" => "https://www.netflix.com/search",
            _ => {
                self.provider_selecting = false;
                return;
            }
        };

        let mut url = match Url::parse(base_url) {
            Ok(url) => url,
            Err(error) => {
                self.error = Some(format!("Invalid provider URL: {error}"));
                self.provider_selecting = false;
                return;
            }
        };

        url.query_pairs_mut().append_pair("q", &title);

        match self.browser.open(url.as_str()) {
            Ok(()) => {
                self.status_message = Some(format!("Opened {provider} search for {title}."));
                self.error = None;
            }
            Err(error) => {
                self.error = Some(format!("Could not open browser: {error}"));
                self.status_message = None;
            }
        }

        self.provider_selecting = false;
    }

    fn handle_key(&mut self, key: KeyEvent) {
        // Streaming-Anbieter-Auswahl
        if self.provider_selecting {
            match key.code {
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    self.open_provider("crunchyroll");
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.open_provider("netflix");
                }
                KeyCode::Esc => {
                    self.provider_selecting = false;
                    self.status_message = Some("Provider selection cancelled.".into());
                }
                _ => {}
            }

            return;
        }

        // Suchfeld bearbeiten
        if self.search_editing {
            match key.code {
                KeyCode::Enter => {
                    self.search_editing = false;
                    self.run_search();
                }
                KeyCode::Esc => {
                    self.search_editing = false;
                    self.status_message = Some("Search cancelled.".into());
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                }
                KeyCode::Char(character) => {
                    self.search_query.push(character);
                }
                _ => {}
            }

            return;
        }

        if self.active == 3 && !self.nav_focus {
            match key.code {
                KeyCode::Up | KeyCode::Left => self.browser = self.browser.next(),
                KeyCode::Down | KeyCode::Right => self.theme = self.theme.next(),
                KeyCode::Char('b') | KeyCode::Char('B') => self.browser = self.browser.next(),
                KeyCode::Char('t') | KeyCode::Char('T') => self.theme = self.theme.next(),
                KeyCode::Tab | KeyCode::Esc => {
                    self.nav_focus = true;
                    return;
                }
                KeyCode::Char('q') => {
                    self.running = false;
                    return;
                }
                _ => return,
            }
            match save_settings(self.browser, self.theme) {
                Ok(()) => {
                    self.status_message = Some("Settings saved.".into());
                    self.error = None;
                }
                Err(error) => self.error = Some(error),
            }
            return;
        }

        match key.code {
            KeyCode::Char('q') => {
                self.running = false;
            }

            KeyCode::Esc => {
                self.running = false;
            }

            KeyCode::Tab => {
                self.nav_focus = !self.nav_focus;
            }

            KeyCode::Char('/') => {
                self.active = 1;
                self.menu_selected = 1;
                self.nav_focus = false;
                self.search_editing = true;
                self.search_query.clear();
                self.status_message = None;
                self.error = None;
            }

            KeyCode::Up => {
                if self.nav_focus {
                    self.menu_selected = self.menu_selected.saturating_sub(1);
                } else {
                    self.anime_selected = self.anime_selected.saturating_sub(1);
                    self.refresh_cover();
                }
            }

            KeyCode::Down => {
                if self.nav_focus {
                    self.menu_selected = (self.menu_selected + 1).min(MENU.len() - 1);
                } else {
                    let length = self.current_anime().len();

                    if length > 0 {
                        self.anime_selected = (self.anime_selected + 1).min(length - 1);
                        self.refresh_cover();
                    }
                }
            }

            KeyCode::Enter if self.nav_focus => {
                self.active = self.menu_selected;
                self.anime_selected = 0;
                self.error = None;
                self.status_message = None;

                match self.active {
                    1 => {
                        self.nav_focus = false;
                        self.search_editing = true;
                        self.search_query.clear();
                    }
                    3 => self.nav_focus = false,
                    4 => self.running = false,
                    _ => self.refresh_cover(),
                }
            }

            KeyCode::Enter if !self.nav_focus => {
                if self.selected_anime().is_some() {
                    self.provider_selecting = true;
                    self.status_message = None;
                    self.error = None;
                }
            }

            _ => {}
        }
    }
}

fn load_cover(picker: &Picker, url: &str) -> Result<Protocol, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .map_err(|error| error.to_string())?;

    let response = client
        .get(url)
        .send()
        .map_err(|error| format!("Cover download failed: {error}"))?
        .error_for_status()
        .map_err(|error| error.to_string())?;

    if response.content_length().unwrap_or(0) > 5 * 1024 * 1024 {
        return Err("Cover image is larger than 5 MB.".into());
    }

    let bytes = response.bytes().map_err(|error| error.to_string())?;
    if bytes.len() > 5 * 1024 * 1024 {
        return Err("Cover image is larger than 5 MB.".into());
    }

    let decoded =
        image::load_from_memory(&bytes).map_err(|error| format!("Invalid cover image: {error}"))?;

    picker
        .new_protocol(decoded, Rect::new(0, 0, 24, 10), Resize::Fit(None))
        .map_err(|error| format!("Could not render cover: {error}"))
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let picker = Picker::from_query_stdio().unwrap_or_else(|_| Picker::from_fontsize((8, 16)));
    let result = run_app(&mut terminal, picker);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    picker: Picker,
) -> Result<(), Box<dyn Error>> {
    let mut app = App::new(picker);

    while app.running {
        terminal.draw(|frame| draw(frame, &app))?;

        if let Event::Key(key) = event::read()? {
            app.handle_key(key);
        }
    }

    Ok(())
}

fn draw(frame: &mut Frame, app: &App) {
    frame.render_widget(
        Paragraph::new("").style(app.theme.base_style()),
        frame.area(),
    );
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " K O R U K Y ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  /  ANIME DISCOVERY", Style::default().fg(Color::Gray)),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(header, areas[0]);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(50),
            Constraint::Percentage(30),
        ])
        .split(areas[1]);

    draw_navigation(frame, app, columns[0]);
    draw_anime_list(frame, app, columns[1]);
    draw_details(frame, app, columns[2]);

    let footer_text = if app.provider_selecting {
        " C: Crunchyroll   N: Netflix   Esc: cancel "
    } else if app.search_editing {
        " Type anime title   Enter: search   Esc: cancel "
    } else if app.active == 3 && !app.nav_focus {
        " B: change browser   T: change theme   Tab: navigation   q: quit "
    } else {
        " Tab: focus   Up/Down: navigate   Enter: select   /: search   q: quit "
    };

    let mut footer_lines = vec![Line::from(footer_text)];

    if let Some(error) = &app.error {
        footer_lines.push(Line::from(Span::styled(
            error.as_str(),
            Style::default().fg(Color::Red),
        )));
    } else if let Some(message) = &app.status_message {
        footer_lines.push(Line::from(Span::styled(
            message.as_str(),
            Style::default().fg(Color::Gray),
        )));
    }

    let footer = Paragraph::new(footer_lines)
        .style(app.theme.base_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.foreground()))
                .style(app.theme.base_style()),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(footer, areas[2]);
}

fn draw_navigation(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = MENU
        .iter()
        .map(|item| ListItem::new(format!("  {item}")))
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.menu_selected));

    let list = List::new(items)
        .style(app.theme.base_style())
        .block(
            Block::default()
                .title(" NAVIGATION ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.foreground()))
                .style(app.theme.base_style()),
        )
        .highlight_style(
            Style::default()
                .fg(app.theme.foreground())
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");

    if app.nav_focus {
        frame.render_stateful_widget(list, area, &mut state);
    } else {
        frame.render_widget(list, area);
    }
}

fn draw_anime_list(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let mut content: Vec<ListItem> = Vec::new();

    if app.provider_selecting {
        content.push(ListItem::new(" SELECT STREAMING PROVIDER"));
        content.push(ListItem::new(""));
        content.push(ListItem::new(" [C] Crunchyroll"));
        content.push(ListItem::new(" [N] Netflix"));
        content.push(ListItem::new(""));
        content.push(ListItem::new(" Esc: cancel"));
    } else if app.active == 0 {
        content.push(ListItem::new(" TRENDING ANIME"));
        content.push(ListItem::new(""));

        for anime in &app.trending {
            content.push(ListItem::new(format!("  {}", anime.display_title())));
        }
    } else if app.active == 1 {
        content.push(ListItem::new(" SEARCH"));
        content.push(ListItem::new(""));

        if app.search_editing {
            content.push(ListItem::new(format!(" > {}_", app.search_query)));
        } else {
            content.push(ListItem::new(format!(" Query: {}", app.search_query)));
        }

        content.push(ListItem::new(""));

        for anime in &app.search_results {
            content.push(ListItem::new(format!("  {}", anime.display_title())));
        }

        if !app.search_editing && app.search_results.is_empty() {
            content.push(ListItem::new(" No results yet."));
            content.push(ListItem::new(" Press / to search."));
        }
    } else if app.active == 3 {
        content.push(ListItem::new(" SETTINGS"));
        content.push(ListItem::new(""));
        content.push(ListItem::new(format!(" BROWSER: {}", app.browser.label())));
        content.push(ListItem::new(" B: change browser"));
        content.push(ListItem::new(""));
        content.push(ListItem::new(format!(" THEME: {}", app.theme.label())));
        content.push(ListItem::new(" T: change theme"));
        content.push(ListItem::new(""));
        content.push(ListItem::new(" Tab: return to navigation"));
    } else {
        content.push(ListItem::new(" Not available yet."));
        content.push(ListItem::new(""));
        content.push(ListItem::new(" This section is planned."));
    }

    let list = List::new(content)
        .style(app.theme.base_style())
        .block(
            Block::default()
                .title(" ANIME ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.foreground()))
                .style(app.theme.base_style()),
        )
        .highlight_style(
            Style::default()
                .fg(app.theme.foreground())
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");

    let mut state = ListState::default();

    if !app.provider_selecting && !app.search_editing {
        if app.active == 0 || app.active == 1 {
            // Zwei zusätzliche Zeilen stehen vor den eigentlichen Anime-Einträgen.
            state.select(Some(app.anime_selected + 2));
        }
    }

    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_details(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(12), Constraint::Min(4)])
        .split(area);

    let cover_block = Block::default()
        .title(" COVER ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.foreground()))
        .style(app.theme.base_style());
    let cover_inner = cover_block.inner(sections[0]);
    frame.render_widget(cover_block, sections[0]);

    if let Some(protocol) = &app.cover {
        frame.render_widget(Image::new(protocol), cover_inner);
    } else {
        let message = app.cover_error.as_deref().unwrap_or("No cover selected.");
        frame.render_widget(
            Paragraph::new(message)
                .style(
                    Style::default()
                        .fg(app.theme.muted())
                        .bg(app.theme.background()),
                )
                .wrap(Wrap { trim: true }),
            cover_inner,
        );
    }

    let text = if app.provider_selecting {
        vec![
            Line::from("STREAMING"),
            Line::from(""),
            Line::from("Choose where to search."),
            Line::from("KORUKY opens the official site."),
        ]
    } else if let Some(anime) = app.selected_anime() {
        vec![
            Line::from(Span::styled(
                anime.display_title(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(format!(
                "Episodes: {}",
                anime
                    .episodes
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "?".into())
            )),
            Line::from(format!(
                "Status: {}",
                anime.status.as_deref().unwrap_or("Unknown")
            )),
            Line::from(format!("Genres: {}", anime.genres.join(", "))),
            Line::from(""),
            Line::from("Source: AniList"),
        ]
    } else {
        vec![
            Line::from("KORUKY"),
            Line::from(""),
            Line::from("Select an anime to see its details."),
        ]
    };

    frame.render_widget(
        Paragraph::new(text)
            .style(app.theme.base_style())
            .block(
                Block::default()
                    .title(" DETAILS ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(app.theme.foreground()))
                    .style(app.theme.base_style()),
            )
            .wrap(Wrap { trim: true }),
        sections[1],
    );
}
