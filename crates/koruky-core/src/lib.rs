use serde::Deserialize;
use serde_json::json;
use std::time::Duration;

const ANILIST_API: &str = "https://graphql.anilist.co";

// KORUKY command and episode request
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpisodeRequest {
    pub title: String,
    pub season: Option<u32>,
    pub episode: Option<u32>,
    pub audio_language: Option<String>,
    pub subtitle_language: Option<String>,
}

// Anime data
#[derive(Debug, Clone, Deserialize)]
pub struct Anime {
    pub id: u32,
    pub title: AnimeTitle,
    pub description: Option<String>,
    pub episodes: Option<u32>,
    pub genres: Vec<String>,
    pub status: Option<String>,
    #[serde(rename = "coverImage")]
    pub cover_image: CoverImage,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnimeTitle {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
}

impl Anime {
    pub fn display_title(&self) -> &str {
        self.title
            .english
            .as_deref()
            .or(self.title.romaji.as_deref())
            .or(self.title.native.as_deref())
            .unwrap_or("Unknown title")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CoverImage {
    pub large: Option<String>,
}

// AniList API response
#[derive(Debug, Deserialize)]
struct ApiResponse {
    data: Option<ApiData>,
    errors: Option<Vec<ApiError>>,
}

#[derive(Debug, Deserialize)]
struct ApiData {
    #[serde(rename = "Page")]
    page: ApiPage,
}

#[derive(Debug, Deserialize)]
struct ApiPage {
    media: Vec<Anime>,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    message: String,
}

// Fetch trending anime
pub fn fetch_trending() -> Result<Vec<Anime>, String> {
    let query = r#"
        query {
            Page(page: 1, perPage: 15) {
                media(
                    type: ANIME
                    sort: TRENDING_DESC
                    isAdult: false
                ) {
                    id
                    title {
                        romaji
                        english
                        native
                    }
                    description(asHtml: false)
                    episodes
                    genres
                    status
                    coverImage {
                        large
                    }
                    siteUrl
                }
            }
        }
    "#;

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| error.to_string())?;

    let response = client
        .post(ANILIST_API)
        .json(&json!({ "query": query }))
        .send()
        .map_err(|error| format!("Network error: {error}"))?
        .error_for_status()
        .map_err(|error| format!("AniList response error: {error}"))?;

    let result: ApiResponse = response
        .json()
        .map_err(|error| format!("Invalid API response: {error}"))?;

    if let Some(errors) = result.errors {
        let message = errors
            .into_iter()
            .map(|error| error.message)
            .collect::<Vec<_>>()
            .join("; ");

        return Err(format!("AniList API: {message}"));
    }

    result
        .data
        .map(|data| data.page.media)
        .ok_or_else(|| "AniList returned no anime data.".to_string())
}

/// Search anime using multiple queries and rank approximate title matches.
pub fn search_anime(search: &str) -> Result<Vec<Anime>, String> {
    let search = search.trim();

    if search.is_empty() {
        return Ok(Vec::new());
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| error.to_string())?;

    // Try the full title first, then a distinctive final word.
    let mut queries = vec![search.to_string()];

    if let Some(last_word) = search.split_whitespace().last() {
        if last_word.len() >= 3 && !last_word.eq_ignore_ascii_case(search) {
            queries.push(last_word.to_string());
        }
    }

    let mut results: Vec<Anime> = Vec::new();

    for query in queries {
        let found = fetch_search_page(&client, &query)?;

        for anime in found {
            if !results.iter().any(|existing| existing.id == anime.id) {
                results.push(anime);
            }
        }
    }

    // Rank results by similarity to the original search.
    results.sort_by(|a, b| title_similarity(search, b).total_cmp(&title_similarity(search, a)));

    Ok(results)
}

fn fetch_search_page(
    client: &reqwest::blocking::Client,
    search: &str,
) -> Result<Vec<Anime>, String> {
    let query = r#"
        query ($search: String) {
            Page(page: 1, perPage: 50) {
                media(
                    type: ANIME
                    search: $search
                    sort: SEARCH_MATCH
                    isAdult: false
                ) {
                    id
                    title {
                        romaji
                        english
                        native
                    }
                    description(asHtml: false)
                    episodes
                    genres
                    status
                    coverImage {
                        large
                    }
                    siteUrl
                }
            }
        }
    "#;

    let response = client
        .post(ANILIST_API)
        .json(&json!({
            "query": query,
            "variables": {
                "search": search
            }
        }))
        .send()
        .map_err(|error| format!("Network error: {error}"))?
        .error_for_status()
        .map_err(|error| format!("AniList response error: {error}"))?;

    let result: ApiResponse = response
        .json()
        .map_err(|error| format!("Invalid API response: {error}"))?;

    if let Some(errors) = result.errors {
        let message = errors
            .into_iter()
            .map(|error| error.message)
            .collect::<Vec<_>>()
            .join("; ");

        return Err(format!("AniList API: {message}"));
    }

    result
        .data
        .map(|data| data.page.media)
        .ok_or_else(|| "AniList returned no anime data.".to_string())
}

/// Compare the search query against all available anime titles.
fn title_similarity(search: &str, anime: &Anime) -> f64 {
    let titles = [
        anime.title.english.as_deref(),
        anime.title.romaji.as_deref(),
        anime.title.native.as_deref(),
    ];

    titles
        .into_iter()
        .flatten()
        .map(|title| similarity(search, title))
        .fold(0.0, f64::max)
}

fn similarity(a: &str, b: &str) -> f64 {
    let a: Vec<char> = a
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();

    let b: Vec<char> = b
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();

    let max_len = a.len().max(b.len());

    if max_len == 0 {
        return 1.0;
    }

    let mut previous: Vec<usize> = (0..=b.len()).collect();

    for (i, ca) in a.iter().enumerate() {
        let mut current = vec![i + 1; b.len() + 1];

        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);

            current[j + 1] = (current[j] + 1)
                .min(previous[j + 1] + 1)
                .min(previous[j] + cost);
        }

        previous = current;
    }

    1.0 - previous[b.len()] as f64 / max_len as f64
}
