mod game;

use game::{
    analyze_puzzle, current_rank, group_words_by_length, load_dictionary, next_rank,
    parse_puzzle, rank_markers, rank_progress, shuffle_letters, validate_submission, WordEntry,
};
use leptos::ev::KeyboardEvent;
use leptos::prelude::*;
use std::collections::HashSet;
use wasm_bindgen::JsCast;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Setup,
    Play,
}

#[component]
fn App() -> impl IntoView {
    let screen = RwSignal::new(Screen::Setup);
    let letters_input = RwSignal::new(String::new());
    let center_letter = RwSignal::new(String::new());
    let setup_error = RwSignal::new(None::<String>);

    let puzzle = RwSignal::new(None::<game::Puzzle>);
    let stats = RwSignal::new(None::<game::PuzzleStats>);
    let found_words = RwSignal::new(Vec::<WordEntry>::new());
    let current_word = RwSignal::new(String::new());
    let feedback = RwSignal::new(None::<(bool, String)>);
    let shuffle_salt = RwSignal::new(0_u64);
    let show_answers = RwSignal::new(false);
    let show_word_list = RwSignal::new(true);

    let total_score = Memo::new(move |_| {
        found_words
            .with(|words| words.iter().map(|entry| entry.points).sum::<u32>())
    });

    let max_score = Memo::new(move |_| stats.with(|s| s.as_ref().map(|s| s.max_score).unwrap_or(0)));

    let rank_name = Memo::new(move |_| {
        current_rank(total_score.get(), max_score.get())
    });

    let progress = Memo::new(move |_| rank_progress(total_score.get(), max_score.get()));

    let next_rank_info = Memo::new(move |_| next_rank(total_score.get(), max_score.get()));

    let outer = Memo::new(move |_| {
        puzzle.with(|p| {
            p.as_ref()
                .map(|puzzle| shuffle_letters(puzzle, shuffle_salt.get()))
                .unwrap_or_default()
        })
    });

    let grouped_found = Memo::new(move |_| {
        let mut groups = group_words_by_length(&found_words.get());
        let mut keys: Vec<_> = groups.keys().copied().collect();
        keys.sort_unstable_by(|a, b| b.cmp(a));
        keys.into_iter()
            .map(|len| (len, groups.remove(&len).unwrap_or_default()))
            .collect::<Vec<_>>()
    });

    let start_game = move |_| {
        setup_error.set(None);

        let raw = letters_input.get().trim().to_ascii_uppercase();
        let center_value = center_letter.get();
        let center_raw = center_value.trim();

        if center_raw.len() != 1 {
            setup_error.set(Some("Pick exactly one center letter.".into()));
            return;
        }

        let center = center_raw.chars().next().unwrap();
        let Some(parsed) = parse_puzzle(&raw, center) else {
            setup_error.set(Some(
                "Enter exactly 7 unique letters (A–Z).".into(),
            ));
            return;
        };

        let analyzed = analyze_puzzle(&parsed);
        if analyzed.words.is_empty() {
            setup_error.set(Some("No valid words for this letter set.".into()));
            return;
        }

        puzzle.set(Some(parsed));
        stats.set(Some(analyzed));
        found_words.set(Vec::new());
        current_word.set(String::new());
        feedback.set(None);
        shuffle_salt.set(1);
        show_answers.set(false);
        screen.set(Screen::Play);
    };

    let reset_game = move |_| {
        screen.set(Screen::Setup);
        puzzle.set(None);
        stats.set(None);
        found_words.set(Vec::new());
        current_word.set(String::new());
        feedback.set(None);
    };

    let append_letter = move |letter: char| {
        feedback.set(None);
        current_word.update(|word| {
            word.push(letter.to_ascii_lowercase());
        });
    };

    let delete_letter = move || {
        feedback.set(None);
        current_word.update(|word| {
            word.pop();
        });
    };

    let shuffle = move |_| {
        shuffle_salt.update(|salt| *salt = salt.wrapping_add(1));
    };

    let submit_word = move || {
        let Some(puzzle) = puzzle.get() else {
            return;
        };

        let dict = load_dictionary();
        let found_set: HashSet<String> = found_words
            .with(|words| words.iter().map(|entry| entry.word.clone()).collect());

        match validate_submission(&current_word.get(), &puzzle, &found_set, &dict) {
            Ok(entry) => {
                let points = entry.points;
                let pangram = entry.pangram;
                let word = entry.word.clone();
                found_words.update(|words| {
                    words.push(entry);
                    words.sort_by(|a, b| a.word.cmp(&b.word));
                });
                current_word.set(String::new());
                feedback.set(Some((
                    true,
                    if pangram {
                        format!("Pangram! +{points} — {word}")
                    } else {
                        format!("+{points} — {word}")
                    },
                )));
            }
            Err(error) => {
                feedback.set(Some((false, error.message().into())));
                current_word.set(String::new());
            }
        }
    };

    // Capture keys globally so honeycomb button focus never steals typing.
    let _keydown = window_event_listener(leptos::ev::keydown, move |event: KeyboardEvent| {
        if screen.get() != Screen::Play {
            return;
        }

        if event.ctrl_key() || event.meta_key() || event.alt_key() {
            return;
        }

        // Ignore when typing into any text field (shouldn't happen in Play).
        if let Some(target) = event
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok())
        {
            let tag = target.tag_name().to_ascii_lowercase();
            if tag == "input" || tag == "textarea" || target.is_content_editable() {
                return;
            }
        }

        let key = event.key();
        match key.as_str() {
            "Enter" => {
                submit_word();
                event.prevent_default();
            }
            "Backspace" => {
                delete_letter();
                event.prevent_default();
            }
            "Escape" => {
                current_word.set(String::new());
                feedback.set(None);
                event.prevent_default();
            }
            _ if key.len() == 1 => {
                if let Some(letter) = key.chars().next() {
                    if letter.is_ascii_alphabetic() {
                        if let Some(puzzle) = puzzle.get() {
                            let lower = letter.to_ascii_lowercase();
                            if puzzle.letters.contains(&lower) {
                                append_letter(lower);
                                event.prevent_default();
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    });

    view! {
        <div class="app" tabindex="0">
            <header class="masthead">
                <p class="eyebrow">"Menhir Holdings"</p>
                <h1>"Waffle"</h1>
                <p class="subtitle">"Spelling Bee. NYC rules. Your letters. No paywall."</p>
            </header>

            <Show when=move || screen.get() == Screen::Setup>
                <section class="panel setup-panel">
                    <h2>"Today's Waffle"</h2>
                    <p class="hint">"Enter the seven letters from today's puzzle, then choose the center letter."</p>

                    <label class="field">
                        <span>"Seven letters"</span>
                        <input
                            type="text"
                            maxlength="7"
                            placeholder="e.g. AIRBNST"
                            prop:value=move || letters_input.get()
                            on:input=move |ev| {
                                let value = event_target_value(&ev).to_ascii_uppercase();
                                let filtered: String = value
                                    .chars()
                                    .filter(|c| c.is_ascii_alphabetic())
                                    .take(7)
                                    .collect();
                                letters_input.set(filtered);
                            }
                        />
                    </label>

                    <label class="field">
                        <span>"Center letter"</span>
                        <input
                            type="text"
                            maxlength="1"
                            placeholder="N"
                            prop:value=move || center_letter.get()
                            on:input=move |ev| {
                                let value = event_target_value(&ev).to_ascii_uppercase();
                                let filtered: String = value
                                    .chars()
                                    .filter(|c| c.is_ascii_alphabetic())
                                    .take(1)
                                    .collect();
                                center_letter.set(filtered);
                            }
                        />
                    </label>

                    <Show when=move || setup_error.get().is_some()>
                        <p class="feedback bad">{move || setup_error.get().unwrap_or_default()}</p>
                    </Show>

                    <button class="primary" on:click=start_game>"Start Waffle"</button>

                    <div class="setup-notes">
                        <p>"Words must be at least 4 letters and include the center letter."</p>
                        <p>"Dictionary: " {load_dictionary().len()} " words (ENABLE1)."</p>
                    </div>
                </section>
            </Show>

            <Show when=move || screen.get() == Screen::Play>
                <div class="play-layout">
                    <section class="panel play-panel">
                        <div class="score-row">
                            <div class="score-chip">
                                <span class="label">"Score"</span>
                                <strong>{move || total_score.get()}</strong>
                            </div>
                            <div class="score-chip rank-chip">
                                <span class="label">"Rank"</span>
                                <strong>{move || rank_name.get()}</strong>
                            </div>
                            <button class="ghost" on:click=reset_game>"New Waffle"</button>
                        </div>

                        <div class="progress-wrap">
                            <div class="progress-track">
                                <div class="progress-fill" style:width=move || format!("{}%", progress.get() * 100.0)></div>
                                {move || {
                                    rank_markers(max_score.get())
                                        .into_iter()
                                        .filter(|(_, points)| *points > 0 && *points < max_score.get())
                                        .map(|(name, points)| {
                                            let left = if max_score.get() == 0 {
                                                0.0
                                            } else {
                                                points as f64 / max_score.get() as f64 * 100.0
                                            };
                                            view! {
                                                <span
                                                    class="rank-marker"
                                                    title=name
                                                    style:left=format!("{left}%")
                                                ></span>
                                            }
                                        })
                                        .collect_view()
                                }}
                            </div>
                            <Show when=move || next_rank_info.get().is_some()>
                                <p class="next-rank">
                                    {move || {
                                        next_rank_info
                                            .get()
                                            .map(|(name, needed)| format!("{needed} to {name}"))
                                            .unwrap_or_default()
                                    }}
                                </p>
                            </Show>
                        </div>

                        <div class="word-display">
                            <p class="current-word" class:shake=move || feedback.get().map(|(ok, _)| !ok).unwrap_or(false)>
                                {move || current_word.get().to_uppercase()}
                            </p>
                            <Show when=move || feedback.get().is_some()>
                                <p
                                    class="feedback"
                                    class:good=move || feedback.get().map(|(ok, _)| ok).unwrap_or(false)
                                    class:bad=move || feedback.get().map(|(ok, _)| !ok).unwrap_or(false)
                                >
                                    {move || feedback.get().map(|(_, msg)| msg).unwrap_or_default()}
                                </p>
                            </Show>
                        </div>

                        <div class="honeycomb" aria-label="Letter honeycomb">
                            <button class="hex hex-nw" on:click=move |_| append_letter(outer.get()[0])>
                                {move || outer.get().get(0).map(|c| c.to_ascii_uppercase().to_string()).unwrap_or_default()}
                            </button>
                            <button class="hex hex-ne" on:click=move |_| append_letter(outer.get()[1])>
                                {move || outer.get().get(1).map(|c| c.to_ascii_uppercase().to_string()).unwrap_or_default()}
                            </button>
                            <button class="hex hex-w" on:click=move |_| append_letter(outer.get()[2])>
                                {move || outer.get().get(2).map(|c| c.to_ascii_uppercase().to_string()).unwrap_or_default()}
                            </button>
                            <button class="hex hex-center" on:click=move |_| {
                                if let Some(p) = puzzle.get() {
                                    append_letter(p.center);
                                }
                            }>
                                {move || puzzle.get().map(|p| p.center.to_ascii_uppercase().to_string()).unwrap_or_default()}
                            </button>
                            <button class="hex hex-e" on:click=move |_| append_letter(outer.get()[3])>
                                {move || outer.get().get(3).map(|c| c.to_ascii_uppercase().to_string()).unwrap_or_default()}
                            </button>
                            <button class="hex hex-sw" on:click=move |_| append_letter(outer.get()[4])>
                                {move || outer.get().get(4).map(|c| c.to_ascii_uppercase().to_string()).unwrap_or_default()}
                            </button>
                            <button class="hex hex-se" on:click=move |_| append_letter(outer.get()[5])>
                                {move || outer.get().get(5).map(|c| c.to_ascii_uppercase().to_string()).unwrap_or_default()}
                            </button>
                        </div>

                        <div class="controls">
                            <button class="control" on:click=move |_| shuffle(()) title="Shuffle">"Shuffle"</button>
                            <button class="control" on:click=move |_| delete_letter() title="Delete">"Delete"</button>
                            <button class="control enter" on:click=move |_| submit_word() title="Enter">"Enter"</button>
                        </div>

                        <div class="meta-row">
                            <span>{move || format!("{} words · {} max pts", stats.get().map(|s| s.words.len()).unwrap_or(0), max_score.get())}</span>
                            <span>{move || format!("{} pangrams", stats.get().map(|s| s.pangram_count).unwrap_or(0))}</span>
                            <button class="text-btn" on:click=move |_| show_answers.update(|v| *v = !*v)>
                                {move || if show_answers.get() { "Hide answers" } else { "Reveal all words" }}
                            </button>
                        </div>
                    </section>

                    <aside class="panel words-panel">
                        <div class="words-header">
                            <h2>"Found Words"</h2>
                            <button class="text-btn" on:click=move |_| show_word_list.update(|v| *v = !*v)>
                                {move || if show_word_list.get() { "Hide" } else { "Show" }}
                            </button>
                        </div>

                        <Show when=move || show_word_list.get()>
                            <Show
                                when=move || show_answers.get()
                                fallback=move || view! {
                                    <ul class="word-list">
                                        {move || grouped_found.get().into_iter().map(|(len, words)| {
                                            view! {
                                                <li class="word-group">
                                                    <span class="group-label">{format!("{len}-letter words")}</span>
                                                    <span class="group-words">
                                                        {words.into_iter().map(|entry| {
                                                            view! { <span class="found-word">{entry.word}</span> }
                                                        }).collect_view()}
                                                    </span>
                                                </li>
                                            }
                                        }).collect_view()}
                                    </ul>
                                }
                            >
                                <ul class="word-list answers">
                                    {move || {
                                        stats.get().map(|s| {
                                            let groups = group_words_by_length(&s.words);
                                            let mut keys: Vec<_> = groups.keys().copied().collect();
                                            keys.sort_unstable_by(|a, b| b.cmp(a));
                                            keys.into_iter().map(|len| {
                                                let words = groups.get(&len).cloned().unwrap_or_default();
                                                view! {
                                                    <li class="word-group">
                                                        <span class="group-label">{format!("{len}-letter words")}</span>
                                                        <span class="group-words">
                                                            {words.into_iter().map(|entry| {
                                                                let found = found_words.with(|found| {
                                                                    found.iter().any(|w| w.word == entry.word)
                                                                });
                                                                view! {
                                                                    <span class="found-word" class:revealed=found class:hidden-word=!found>
                                                                        {entry.word.clone()}
                                                                    </span>
                                                                }
                                                            }).collect_view()}
                                                        </span>
                                                    </li>
                                                }
                                            }).collect_view()
                                        }).unwrap_or_default()
                                    }}
                                </ul>
                            </Show>
                        </Show>
                    </aside>
                </div>
            </Show>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
