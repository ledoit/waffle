# Waffle

Free **NYT Spelling Bee** clone — enter today's letters and play the full game. No paywall.

**Live:** https://waffle.menhir-holdings.com

**Stack:** Rust → WebAssembly · [Leptos](https://leptos.dev/) 0.7 · ENABLE1 dictionary · NYC scoring & ranks

## Run

```bash
# Dev (hot reload)
trunk serve --open

# Production static build → web/
./scripts/build.sh
python -m http.server 8080 --directory web
```

## Play

1. Enter the **7 unique letters** from today's NYT puzzle (or invent a hive).
2. Pick the **center letter**.
3. Click the honeycomb or type on the keyboard.
4. Words must be **4+ letters**, use only hive letters, and include the center.
5. **Pangrams** (all 7 letters) get a +7 bonus.

## Scoring

| Word | Points |
|------|--------|
| 4 letters | 1 |
| 5+ letters | length |
| Pangram | length + 7 |

Ranks: Beginner → Good Start → Moving Up → Good → Solid → Nice → Great → Amazing → Genius → Queen Bee

## Tests

```bash
cargo test
```
