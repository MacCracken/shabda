# Changelog

## [3.2.0] — 2026-11-17

**Dictionary stress.** shabdakosh 3.2.0 keeps CMUdict's stress digits, and a dictionary word now
takes them. Through 3.1.3 the engine predicted every word's stress by rule, dictionary words
included, because the dictionary had none to give.

- Requires **shabdakosh 3.2.0**.
- One function is added, `shabda_assign_stress_dictionary`. None is removed, and no signature
  changes.
- **1,404 assertions / 11 suites** (1,342 in 3.1.3). `cyrius fuzz`, `--poison` and `--bounds` pass.

### How it was measured

Every word of shabdakosh's `data/cmudict-5k.txt` (10,617) was converted alone through the engine,
under 3.1.3 and under 3.2.0, and scored against CMUdict's digits for the variant whose phonemes the
engine gave. Function words are left out (see below).

| Content words | 3.1.3 | 3.2.0 |
|---|---:|---:|
| Polysyllables (8,702), primary stress on the right syllable | 87.8% | 100% |
| Polysyllables, whole pattern (primary, secondary, unstressed) | 62.9% | 100% |
| All content words (10,582), whole pattern | 69.5% | 100% |

- **The primary moves in 1,058 words**: for example *absolutely*, *accuracy*, *agriculture*,
  *already*, *apparatus*, *around*, *automobile* and *brochure*.
- **2,636 words gain a secondary stress**, which the rules never gave: *photograph*
  F OW1 T AH0 G R AE2 F, *abbreviation*.
- **Only stress changes.** Over the 10,855-input conversion differential, 3,237 English lines
  change, and every one of the 4,472 changed events differs in stress alone.
  - No phoneme or duration changes.
  - No Spanish, German, Hindi, Arabic or Sanskrit output changes.

### Changed (output differs from 3.1.3)

- **A dictionary word's stress is the dictionary's.** CMUdict's 1 is Primary and 2 is Secondary;
  0, and a consonant, are Unstressed. This covers every path to the dictionary:
  - the direct lookup;
  - a heteronym's selected variant: *the record* is REcord (R EH1 K ER0 D), *to record* reCORD
    (R IH0 K AO1 R D);
  - the foreign-word path, which strips the accents and looks again: *fiancé* reads fiance's
    F IY0 AA0 N S EY1, and *protégé* protege's P R OW1 T AH0 ZH EY2;
  - `shabda_convert_streaming`.
- **Function words stay unstressed, as on every other path.** CMUdict gives a word's citation form
  (*of* AH1 V, *to* T UW1), but English function words are weak in running speech.
- **Words without dictionary stress keep the 3.1.3 stress rules.** These are the words the rules
  convert, and entries a caller adds without stress (`shbdk_dict_insert`).

### Added

- **`shabda_assign_stress_dictionary(phonemes, stress, is_content_word)`**, and its `_a` form. It
  builds the events of a dictionary pronunciation from its phonemes and its stress vec
  (`shbdk_pronunciation_stress`), and gives a function word no stress.

### Dependencies

- **shabdakosh 3.1.4 → 3.2.0**: the release that adds lexical stress. A pronunciation keeps
  CMUdict's digit for each vowel, and all 10,617 built-in words have it.
  - `lib/shabdakosh.cyr` is that release's `dist/shabdakosh.cyr`.
  - `cyrius.lock` changes only that file's hash.
  - `dist/shabda.cyr` now calls `shbdk_pronunciation_stress` and `shbdk_dict_entry_primary`, so a
    consumer must supply shabdakosh 3.2.0 or later. It no longer calls `shbdk_dict_lookup`.

### Allocator

A conversion still allocates only through its Allocator.

- The stress is read from shabdakosh's own vec and never copied.
- The one new allocation, the slot a lookup writes the stress to, comes from `a`.
- The 10,808-input differential is clean: the same events, 0 bytes on the default heap, and arena
  bytes equal to what the default form keeps.
- The static walk of the convert path still finds only the known `shbdk_dict_lookup_entry`
  delegation, for a word that is not lowercase.

### Performance

Medians of 12 alternated runs of 3.1.3 and 3.2.0, in one session:

| Benchmark | 3.1.3 | 3.2.0 | |
|---|---:|---:|---:|
| g2p_hello_world | 16.95 µs | 15.15 µs | -10.6% |
| g2p_sentence | 70.72 µs | 66.70 µs | -5.7% |
| g2p_sentence_arena | 67.42 µs | 64.51 µs | -4.3% |
| dict_english_construction | 7.88 ms | 9.86 ms | +25.2% |

- Conversions are faster: a dictionary word skips syllabification and the stress rules.
- `shabda_g2p_new` pays about 2 ms more, once per engine, because shabdakosh decodes the stress of
  10,640 pronunciations as it builds its English map.
- `speak_*` and the lookups are within noise.

## [3.1.3] — 2026-11-17

Accuracy release: the sweep [ADR-0002](docs/adr/0002-accuracy-over-rust-parity.md) calls for, over every
module, with accuracy as the oracle and `rust-old/` as the list of capabilities. No dependency or toolchain
change (the 3.1.2 pins). No function is removed and no signature changes. Four are added:
`shabda_assign_stress_english`, `shabda_syllabify_for`, `shabda_ssml_prosody_percent` and
`shabda_ssml_node_rate_percent`, with the constants `SHABDA_EMPH_NONE` and `SHABDA_RATE_PERCENT`; an SSML
Prosody node grows from 24 to 32 bytes. Output changes in every language, deliberately; consumers that pin
phonemes, stress or SSML timings should read *Changed*. **1,342 assertions / 11 suites** (879 in 3.1.2).
`cyrius fuzz`, `--poison` and `--bounds` pass, and `cyrius audit` exits 0.

### How it was measured

Each change was checked against gold pronunciations, and kept only where it was right on the gold and no
gold word got worse:

| Gold | 3.1.2 | 3.1.3 |
|---|---:|---:|
| English rules, exact words: the held-out half of `data/cmudict-5k.txt` (5,308 words) | 13.7% | 46.4% |
| English rules, phoneme error rate, same half | 31.3% | 13.9% |
| English primary stress, polysyllabic content words through the engine (8,702, CMUdict's digits) | 70.9% | 87.9% |
| Spanish rules, exact words, varna's Swadesh list (25) | 16% | 72% |
| German, same | 28% | 80% |
| Hindi, same | 40% | 88% |
| Arabic, same | 0% | 28% |

- **The Swadesh lists are the independent check** for the other languages, and they are small and narrow:
  their remaining Spanish and German misses are allophones (ð β ɣ) and symbols shabdakosh's IPA parser
  drops (x ɣ ɐ ʏ), not rule errors.
- **Larger lists were used to tune the rules** and are not independent: 360 Spanish and 410 German words
  with eSpeak NG 1.52.0 pronunciations and 27 documented corrections, plus 216 / 336 held-out words scored
  once at the end (213 / 325 exact); 139 Hindi, 67 Sanskrit and 86 Arabic words transcribed from Ohala
  (1983) for Hindi, classical values for Sanskrit, and Watson (2002) / Ryding (2005) for Arabic.
- **Through the engine**, over the 10,855-input conversion differential: 2,015 of the 10,617 dictionary
  words change, 2,009 of them in stress only and 6 because a heteronym's isolated reading moved (see
  *Heteronyms*). Every sentence, SSML and other-language input with a word the rules spell changes.

### Changed — English letter-to-sound rules (src/rules.cyr)

The English rules are the fallback for every word the dictionary does not have, in every language
(an unknown language tag falls back to them too). Through 3.1.2 they were the Rust's: one phoneme
per letter, a few 2- to 4-letter patterns with no context, a prefix list and an -ed step. Put through
them, **13.5%** of the 10,617 words of shabdakosh's `data/cmudict-5k.txt` came out exactly as CMUdict
spells them, with a phoneme error rate of 31.5%: an r after every r-coloured vowel, a full vowel in
every unstressed syllable, /oʊ/ for every short o, doubled consonants pronounced twice.

- **A context-sensitive rule engine** replaces them. A rule is LEFT [MATCH] RIGHT -> phonemes, with
  NRL's context classes (vowels, consonants, voiced consonants, front vowels, suffixes, sibilants).
  The rule set is NRL Report 7948 (Elovitz, Johnson, McHugh and Shore, 1976; a US Government work,
  public domain), written as ARPABET and mapped to svara exactly as shabdakosh maps CMUdict
  (`tests/engine.tcyr` checks all 44 symbols).
- **Additions, each measured on CMUdict**: a doubled consonant is one sound; the unstressed suffixes
  -ity, -ment, -ance / -ence, -ancy / -ency, -ant / -ent, -ible, -ture, -tive / -sive; re- is /rɪ/
  (CMUdict's vowel for 83 of 130 re- words); "ear" / "ere" before a vowel or at the end are /ɪr/;
  tch, dg, dj; a silent t in -stle / -sten; a silent b in -mb and -bt; initial gn, ps, pn; a before x
  is /æ/; "alm" is /ɑm/; the y digraphs (ay, ey, oy) before the rules that would take y as a
  consonant ("played" was P L EY IH D).
- **Vowel reduction.** Letter rules cannot see stress, so a pass after them predicts the primary
  stress (stress-taking and stress-shifting suffixes, unstressed prefixes in two-syllable words, else
  the Latin heavy-penult rule), keeps a secondary two syllables before it, and reduces AA, AE, AH,
  AO and EH elsewhere to schwa, a medial /ɪ/ after a full vowel too, and an unstressed schwa + r +
  vowel to CMUdict's ER0.
- **-ed voicing** after a silent e follows the sound before it, as the Rust's engine did: /t/ after a
  voiceless consonant, /d/ otherwise.
- **Measured** on the 10,617 words, split into two halves (even and odd positions in the list); the
  rules were tuned on the first and the second was only measured:

  | | exact words, tuning half | exact words, held-out half | phoneme error rate (held out) |
  |---|---:|---:|---:|
  | 3.1.2 | 13.3% | 13.7% | 31.3% |
  | NRL rules alone | 33.6% | 34.5% | 17.4% |
  | 3.1.3 | 46.5% | 46.4% | 13.9% |

  The remaining errors are mostly stress guesses (a stressed vowel reduced, or the reverse).
- **No dictionary word's phonemes change**: only words that reach the rules do.
- **Cost**: a word through the English rules takes 17–22 µs, against 3.6–4.9 µs through 3.1.2's,
  over the same 10,617 words in three alternated runs. The rule table is built once per process, in
  `shabda_g2p_new` (on the default heap, like the heteronym rules), so a conversion through
  `shabda_convert_a` still keeps nothing on the default heap.
- **Tests**: `tests/rules.tcyr` assertions that pinned the Rust's output now pin CMUdict's: "nation"
  is N EY SH AH0 N (was /æ/), "walked" W AO K T (the l is silent), "unhappy" starts with schwa,
  "axe" is AE K S (the Rust dropped the x), "through" is TH R UW (was /oʊ/), re- is /rɪ/.

### Changed — Spanish and German rules

- **Spanish vowels are /a e i o u/** (`VOWEL_A/E/I/O/U`). They were English stand-ins, `OPEN_A`, `OPEN_E`,
  `CUP_V`, so casa was k ɑ s ɑ and every u was synthesized as [ɛ] (svara gives `CUP_V` `OPEN_E`'s formants).
- **Spanish glides**: an unaccented i or u next to a vowel is /j w/ (quién k j e n, fuego f w e g o, aire,
  auto, ciudad); a written accent marks hiatus (día, baúl); ü is /w/.
- **Spanish consonants**: ñ is `NASAL_PALATAL`; ll and consonantal y are `FRIC_PALATAL_J` /ʝ/ (yeísmo);
  y after a vowel is /j/ and the word y /i/; c, g and gu before é / í are no longer ignored; initial x is /s/;
  the jota is `FRIC_UVULAR_CH` (χ), the Castilian realization and closer to [x] than the [h] it was (svara
  has no velar /x/; the rules already use distinción θ).
- **German vowel length and quality**: a nucleus pass gives long and short vowels by German's spelling rules
  (long before h, ß, a vowel, the end or one consonant + vowel; short before double consonants and
  clusters), with the pairs iː/ɪ, eː/ɛ, oː/ɔ, uː/ʊ and øː/œ (svara's front rounded vowels); every e was ɛ,
  every o and u long, a was ɑ, ö was ɛ.
- **German schwa**: e in be- / ge- and in final -e -en -el -er -em -es -et -est -ern -eln -ens -ert -elt
  -end is /ə/ (Sonne z ɔ n ə, was z o n ɛ).
- **German r is `FRIC_UVULAR_R` /ʁ/**, not the English approximant the table relabeled ʁ.
- **German ch**: ç after front vowels and consonants and in -ig; the ach-Laut after a o u au is χ; chs is
  ks; initial ch before a o u l r is k. It was ʃ everywhere.
- **German affricates and clusters**: z, tz, zz are `AFFR_TS`, pf `AFFR_PF`, tsch `AFFR_CH`; ng and n before k
  are /ŋ/; x is k s; au is `DIPH_AU` (Frau, Baum were u); c before e i ä y is t͡s; sp / st are ʃp / ʃt only
  at the start of a word or stem (Fenster was fɛnʃtɛɹ); h after a vowel is silent; b d g devoice before an
  obstruent and before -lich -ling -los -nis; s is /z/ before a vowel unless an obstruent precedes it.
- **The Spanish and German IPA tables** map the accurate phonemes to varna's symbols and the retired
  stand-ins to their own IPA, so validation reports a stand-in that reaches it; German diphthongs map to
  nothing, as varna's German has none.

### Changed — Hindi, Sanskrit and Arabic rules

- **Anusvara and chandrabindu** keep the vowel before them (they suppressed it: लंबा was l n b ɑ) and take
  the place of a following stop (ŋ ɲ ɳ n m); elsewhere they are n in Hindi and m in Sanskrit, the closest
  svara can say to a nasal vowel.
- **Hindi schwa deletion** is Ohala's: the medial VC_CV rule after the final one (आदमी ɑ d m iː, मछली
  m ə tʃ l iː); a lone syllable keeps its schwa; ə is [ɛ] before a ɦ that closes its syllable (कहना, यह).
- **ह is `FRIC_GLOTTAL_H` /ɦ/** in both languages; Hindi visarga is ɦ, Sanskrit's stays [h].
- **Retroflexes and nukta**: ट ठ ड ढ ण ष are the retroflex phonemes, ञ is ɲ; the nukta letters ड़ ढ़
  क़ ख़ ग़ ज़ फ़ are ɽ ɽ q χ ʁ z f, decomposed or precomposed, and a nukta no longer adds a schwa
  (बड़ा was b ə d ə ɑ); Hindi ज्ञ is /ɡj/.
- **Vowels**: उ / ु is ʊ (was ʌ, synthesized as [ɛ]); ए / े is e; Hindi ऐ is ɛː; Sanskrit ऐ / औ are one
  diphthong each; ॉ is ɔ; ॐ and ळ are voiced (they gave nothing).
- **IAST and romanized input**: the precomposed IAST letters are read (kṛṣṇa gave k ə); romanized Hindi,
  Sanskrit and Arabic read their digraphs (aa ii ee uu oo ai au, kh gh ch jh th dh ph bh, sh, Arabic sh kh
  gh th dh).
- **Arabic vowels**: /a/ is `VOWEL_A`, backed to [ɑ] beside an emphatic or uvular, the main acoustic cue of
  emphasis svara can give (one function, `_shabda_arabic_a`, decides it); a long vowel is one phoneme
  (كِتَاب was k ɪ t ɑ ɑ b); damma is ʊ; unvocalized ي / و between consonants or at the end are /iː uː/.
- **Arabic consonants**: ح is ħ, ع is ʕ (was a glottal stop), ق is q, خ is χ, غ is ʁ (was the stop g), ر is
  the trill; shadda doubles the consonant in either mark order (بَّ was b a a).
- **The article and spelling**: a sun letter takes the article's l (الشمس a ʃ ʃ m s), also after و ف ب ك;
  a word-initial bare alif is wasl /i/ unless before l; the alif after tanwin al-fath is silent; ة is a or t;
  hamza seats imply their vowel.
- **IPA tables**: Hindi `OPEN_A` is ɑː and `OPEN_E` ɛː (they were ɛː and e); the new phonemes map to varna's
  symbols; Sanskrit `NASAL_N` is n̪, which clears the false n flags; Arabic `PLOSIVE_G` is ɡ (was ɣ). Hindi
  and Sanskrit validation now reports varna's real gaps instead of hiding them: Hindi has no /aː/ (varna's
  own Swadesh writes ɑː), ɽ, z, q, x, ɣ; Sanskrit no h.

### Changed — stress, syllables, text, heteronyms and SSML

- **English stress**: `shabda_assign_stress_english` (the engine uses it for English) applies four standard
  English rules (Chomsky and Halle 1968, Kiparsky 1982, Fudge 1984): /ə/ never takes stress; stress-neutral
  suffixes (-ly -ness -less -ful -ing -er -ted / -ded) are set aside when both the spelling and the phonemes
  show them; pre-stressing suffixes (-ic, -ical, -ity, -ify, -ion, -ian, -ial, -ious, -ient, -eous) stress
  the syllable before them and the French ones (-ee -eer -ese -ette -esque -ique -oon -aire) themselves; a
  disyllable whose first syllable is an unstressed prefix with /ɪ/ is stressed on the root. Primary stress
  is right on 87.9% of polysyllabic content words (70.9%); 1,618 words fixed, 141 regressed. shabdakosh
  keeps no stress digits, so dictionary stress needs an upstream capability (see *Not fixed*).
- **Sonority and onsets**: sonority comes from svara's class for all 100 phonemes, with taps, flaps,
  trills and ʁ ranked as liquids; through 3.1.2 everything outside the 1.x set had the stops' 1, so otro
  split ot.ɾo. Legal onsets are per language (`shabda_syllabify_for`, used by the engine): English uses
  a table of the onsets English allows, as shabdakosh's syllabifier does (atlas æt.lə, singer sɪŋ.ɚ),
  Spanish obstruent + l / ɾ without tl / dl (otro o.tɾo, honra hon.ra), Arabic a single consonant
  (madrasa mad.ra.sa), the others the sonority rule. German Abreise is a.bʁaɪ.zə.
- **Diacritics are stripped for English only**: the foreign-loan path ran in every language, so über reached
  the German rules as uber and niño as nino.
- **Combining marks stay in the word**: `shabda_normalize` deleted every non-alphabetic mark, the virama,
  nukta, sukun, maddah and superscript alef included, so every Sanskrit conjunct gained a vowel through a
  conversion (धर्मः said dəɾəməh). A mark (Mn / Mc / Me) after a letter now stays.
- **Numbers** read as an English reader says them: 1st first, 21st twenty first, $3.50 three dollars and fifty
  cents, $5 million five million dollars, 50% fifty percent, 3:05 three oh five, 1999 nineteen ninety nine
  in a date context, 1990s nineteen nineties, 007 zero zero seven, 10-20 ten to twenty, COVID-19 covid
  nineteen (was covid negative nineteen), 5km five km.
- **Boundaries and abbreviations**: ? and ! give a sentence pause, a run of boundaries is one pause (... was
  0.9 s), & is "and"; St. and Dr. after a capitalized word are street and drive; the sentence period is kept
  after a non-title abbreviation ("etc. Then") but not after e.g. and i.e., which are new; dotted
  initialisms are spelled.
- **Acronyms**: the read-as-word test follows English letter phonotactics (IBM, ATM, ESPN, CIA and USA were
  read as words), a spelled acronym is one token ("f-b-i"), and the English engine says spelled and lone
  letters by their names from a CMUdict letter table: FBI is /ɛf biː aɪ/, it was /f b aɪ/.
- **Heteronyms**: seven rules picked the other sense in every context, because their variant indices
  assumed an order the dictionary does not have (read = [riːd, rɛd], tear = [tɛr, tɪr], bow = [baʊ, boʊ],
  record / refuse / produce = [verb, noun]): "I will read" said /rɛd/. The indices now match the rules'
  documented intent. lead is /liːd/ by default as well as after its verb triggers: the verb and the nouns
  for a position and a leash are /liːd/, and only a following word, which the window cannot see, marks
  the metal. "a" and "the" no longer trigger minute and bass, nor "a" bow. `tests/heteronym.tcyr` checks
  every rule's two picks against the dictionary. In isolation, read is now /rɛd/, tear /tɪr/, bow /boʊ/
  and record, refuse and produce the nouns.
- **SSML 1.1 and XML 1.0**: an XML declaration, comments and a DOCTYPE are skipped (a document starting
  with `<?xml?>` said one word) and CDATA is text; an unknown element keeps all its children
  (`<p><s>One.</s><s>Two.</s></p>` dropped "Two."); entities and character references are decoded ("AT&amp;T"
  said "amp"); white space around = is allowed and attribute names match whole; a bare `<break/>` and an
  unknown strength are medium, 400 ms (was 300); real-number times parse ("1.5ms" failed the document);
  `<emphasis level="none">` exists and "moderate" now gives primary stress without lengthening;
  `<prosody rate="N%">` scales the default rate.

### Not fixed

Each is listed in the roadmap's *Accuracy sweep* item.

- **Dictionary stress.** shabdakosh's packed data keeps no stress digits, so English stress is predicted
  even for dictionary words. shabdakosh would need a per-pronunciation stress vector (CMUdict's 0 / 1 / 2,
  IPA ˈ ˌ) and an accessor such as `shbdk_pronunciation_stress(p)`.
- **varna's inventory gaps**, which validation now reports: English /ɜ aɪ aʊ ɔɪ/; Hindi /aː ɽ z q x ɣ/;
  Sanskrit h.
- **Stress outside English** still follows the Latin rule: Spanish needs its orthographic rule, German stem
  stress with unstressed prefixes. **Number and abbreviation expansion is English in every language.**
- **Unvocalized Arabic** has no short vowels or shadda to read; diphthongs in unvocalized monosyllables
  (بيت يوم) are misread as long vowels. Aspiration and breathy voice have no svara phoneme.
- **German lexical exceptions** (Mond, vielleicht, final-stress loans, compounds such as Beispiel) and Spanish
  x as the jota (México); English 2-letter acronyms and AT&T; `<phoneme>`, `<sub>` and `<say-as>`;
  intonation is computed and discarded; the pause durations (0.15 s comma, 0.30 s period, 0.04 s between
  words) have no source and are unchanged.

### Tests

- 1,342 assertions in 11 suites (879): engine 309, rules 294, validate 235, normalize 120, ssml 97,
  syllable 96, prosody 72, heteronym 48, shabda 37, error 25, allocator 9.
- Assertions that pinned an inaccurate result were rewritten to the accurate one, each named in its
  section above.
- **`cyrius fuzz`** now converts every case through all six engines (each against `shabda_convert_a`) and
  feeds the raw bytes to all six rule sets, with a corpus of native-script inputs carrying virama, nukta,
  anusvara, visarga, IAST, harakat, shadda in both orders, sukun, the article and ta marbuta.
- **The allocator rule holds**: across the 10,808 convert inputs `shabda_convert_with_a` keeps nothing on the
  default heap and its arena holds exactly what the default-heap form kept; the new English rule table,
  like the heteronym table, is built once in `shabda_g2p_new`.

### Benchmarks

`tests/shabda.bcyr`, x86_64. Medians of **8** alternated runs of 3.1.2 and 3.1.3 in one session.

| Benchmark | 3.1.2 | 3.1.3 | Δ |
|-----------|-------|-------|---|
| g2p_hello_world | 15.75 µs | 17.05 µs | +8.3% |
| g2p_sentence | 62.88 µs | 71.50 µs | +13.7% |
| g2p_sentence_arena | 59.56 µs | 67.90 µs | +14.0% |
| speak_hello | 3.56 ms | 3.74 ms | +4.9% |
| speak_sentence | 11.05 ms | 11.04 ms | noise |
| dict_english_construction | 7.96 ms | 7.96 ms | noise |
| dict_lookup_hit | 148 ns | 143 ns | noise |
| dict_lookup_miss | 162 ns | 150 ns | noise |

- **Converting dictionary words costs 8–14% more**, and the ranges do not overlap. These sentences use no
  letter-to-sound rules, so the time is in the new stress rules, per-language syllabification and the
  larger normalization pass; which of them was not profiled. shabda's bar is accuracy over speed.
- **A word through the English rules** costs 17–22 µs, from 3.6–4.9 (see *English letter-to-sound rules*);
  German rules about 1.1 µs a word, from 0.4; Spanish about 7% more. No bench row covers the rules, so
  these come from separate timing programs over the 10,617-word list.

## [3.1.2] — 2026-11-17

Dependency release, plus the allocator-taking conversion hoosh asked for. `shabdakosh` 3.1.1 ->
**3.1.4**, `svara` 3.6.3 -> **3.14.0**, `varna` 2.5.2 -> **2.6.0**, and transitively `hisab` 3.4.1
-> **3.5.0**, `naad` 2.3.3 -> **2.3.5**, `goonj` 2.3.1 -> **2.3.2**. Each is its dep's newest tag and
pins cyrius **6.7.18**, which stays the pin: it is still cyrius's newest tag. sakshi stays 2.5.10, now
as the stdlib leaf only. svara 3.6.4 dropped its sakshi package, so `cyrius.lock` commit-pins 5 deps,
not 6: the same five commits shabdakosh 3.1.4's lock carries. `lib/` and `cyrius.lock` were regenerated
from scratch. Every vendored bundle is byte-identical to its tag's `dist/`, every stdlib leaf to the
6.7.18 snapshot, `cyrius deps --verify` passes 61 of 61, and `cyrius deps --locked` is clean.

No function is removed and no signature changes. Two are added: `shabda_convert_a` and
`shabda_convert_with_a`. Output changes in two ways (see *Changed*): ARPABET `IY` is /iː/, inherited
from shabdakosh, and the letter-to-sound rules and IPA tables drop three of the Rust's stand-ins,
under a new rule: `rust-old/` is the list of capabilities and accuracy is the oracle
([ADR-0002](docs/adr/0002-accuracy-over-rust-parity.md)). **879 assertions / 11 suites** (727 / 10
in 3.1.1). `cyrius fuzz` passes plain, `--poison` and `--bounds`, and `cyrius audit` exits 0.

### Added — allocator-taking conversion ([issue](docs/development/issues/archived/2026-11-11-conversion-allocates-on-the-default-heap.md), hoosh)

- **`shabda_convert_a(a, e, text, out_events)` and `shabda_convert_with_a(a, e, text, options,
  out_events)`** draw every allocation from Allocator `a`, in the stdlib's `vec_new_a` / `alloc_via`
  convention: the events vec, the events, and everything the conversion builds on the way. A caller
  that passes an arena and resets it after reading the events gets all of it back. On the issue's three
  inputs a conversion keeps **0 bytes** on the default heap, and its arena holds exactly the bytes the
  default-heap form keeps there (4,608 / 60,744 / 250,760). The engine stays where `shabda_g2p_new` put
  it.
- **The existing forms are unchanged.** `shabda_convert` and `shabda_convert_with` are now one-line
  wrappers over `default_alloc()`, and they allocate exactly what 3.1.1 did: the same three figures,
  and output byte-identical across the differential below.
- **How.** Each of the 73 functions `shabda_convert_with` reaches that allocates, directly or through a
  callee, got an `_a` twin that takes the Allocator first: rules 25, normalize 25, engine 10, validate
  5, syllable 3, prosody 3, heteronym 2. All 73 twins are private apart from the two above. The 26
  public functions and 7 private ones that still have callers outside a conversion (SSML, streaming)
  stay as wrappers. The other 40 private ones had no caller left and are gone. The twin keeps the
  original body, with `alloc` -> `alloc_via(a, …)`, `vec_new()` -> `vec_new_a(a)`, and so on, and the
  rewrite touched code only, never strings or comments.
- **Where a dependency allocated on the default heap with no `_a` form**, shabda stands in:
  - `shbdk_dict_lookup` / `shbdk_dict_lookup_all` allocate a 16-byte `Str` key on every call.
    `_shabda_dict_entry_a` takes shabdakosh's lowercase fast path with the key from `a`, user overlay
    first, then the base map, through shabdakosh's public accessors. A word that is not already
    lowercase goes to `shbdk_dict_lookup_entry` itself. `shabda_normalize` lowers every letter, so a
    conversion never takes that branch, and the fuzz harness checks it on random bytes.
  - `svara_phoneme_event_new` takes no Allocator. `_shabda_event_new_a` builds the same record through
    svara's public struct (`sizeof(SvPhonemeEvent)`, svara's setters). `tests/prosody.tcyr` compares
    the two byte for byte, on the default heap and on an arena filled with 0xA5, so a field svara adds
    fails a test.
  - stdlib `str_cstr` and `str_lower_cstr` copy onto the default heap: `_shabda_str_cstr_a` and
    `_shabda_ascii_lower_a`. `vec_push` and `str_builder_add_cstr` end the process on a refused grow,
    while their `_a` forms return -1: `_shabda_vec_push_a` and `_shabda_sb_add_cstr_a` keep the ending
    and the message.
- **`shabda_g2p_new` builds the process-wide tables a conversion reads**: the heteronym rules, and
  varna's inventory and phonotactics for the language. Through 3.1.1 the first conversion built them,
  so through `shabda_convert_a` they would have landed in the caller's arena and been read again after
  it was reset. The English engine grows from 6,139,024 to 6,147,200 bytes. Process memory overall
  does not change, since the first conversion no longer builds them.
- **An arena reused after a reset is not zeroed**, unlike the default heap. Every allocation a
  conversion makes was checked to be written before it is read, and the tests run each conversion on
  an arena filled with 0xA5 first.
- **A refused allocation is not an error.** A refused vec or builder grow ends the process, as
  `vec_push` does. Pass an Allocator that grows, such as `arena_allocator_growable()`. A growable arena
  chains chunks from the default heap until it reaches its high-water mark, and then keeps nothing.
- **Not covered:** `shabda_convert_ssml`, `shabda_convert_streaming`, `shabda_speak` and
  `shabda_speak_with` still allocate on the default heap.
- **Threads.** The issue asked whether a conversion writes the engine. It does not. A conversion reads
  the engine, the tables above and the dictionary, and writes only its Allocator and the event vec it
  returns. So two threads can convert at once on one engine, each with its own Allocator, since an
  arena is not thread-safe. The default-heap forms are safe too, because `alloc` is locked. Changing
  the dictionary (`shbdk_dict_insert*` through `shabda_g2p_dictionary_mut`) while another thread
  converts is still the caller's to serialize.

### Fixed

- **A refused builder grow wrote past the buffer.** `_shabda_sb_reserve` returned 0 when the larger
  buffer was refused, and each push then wrote past the old one anyway: one byte past it for
  `_shabda_sb_push_byte`, the whole string for `_shabda_sb_push_cstr`. On the default heap only a 2 GiB
  request or an mmap failure got there; a fixed-size arena gets there as soon as it fills. Reserve now
  returns -1 and the pushes stop, which gives the truncated build the comment there always described.
  `tests/normalize.tcyr` places a canary after a 16-byte buffer in a 48-byte arena, and it survives
  20 pushes.

### Changed — output, inherited from shabdakosh 3.1.3

- **ARPABET `IY` is now /iː/ (`SVARA_PH_VOWEL_LONG_I`, 14), not /e/ (`SVARA_PH_VOWEL_E`, 1).** A
  differential over 10,855 inputs (the 10,617 words of shabdakosh's `data/cmudict-5k.txt`, 39
  sentences under default, emphasis, 200 wpm, timing-profile and streaming options, 8 SSML documents
  and 35 inputs in the other five languages) was built under 3.1.1 and 3.1.2. 2,389 rows differ, and in
  every one the only change is phoneme 1 -> 14: durations and stress are bit-identical. That is 2,306
  of the 10,617 words; the other 83 rows are sentences. Every other row is byte-identical.

### Changed — accuracy over Rust parity ([ADR-0002](docs/adr/0002-accuracy-over-rust-parity.md))

Through 3.1.1 the bar was "matches what the Rust did". It is now accuracy, with `rust-old/` as the list
of capabilities the port must have. After the deps above, the Rust-faithful rules disagreed with the
dictionary on /iː/, and the Rust-faithful Spanish table flagged every trill once varna spelled it as IPA
does. So this release retires the stand-ins behind both:

- **/iː/ is `SVARA_PH_VOWEL_LONG_I`, not `VOWEL_E` (svara's /e/)**, wherever the rules spell it: English
  "ee", "ea", a magic-e "e", word-final "y" (CMUdict's `IY0`) and the "re-" / "pre-" prefixes; German
  "ie"; Hindi and Sanskrit ई / ी; Sanskrit ॠ / ॄ (tap + /iː/). The Rust spelled them all `VowelE`,
  although svara 1.0 already had `VowelLongI`. A dictionary word and an unknown word with the same
  spelling now get the same vowel.
- **German ü and y are `SVARA_PH_VOWEL_Y`** (/y/), not `VOWEL_E`. svara 1.0 had no /y/; svara has it now.
- **Spanish trills with `SVARA_PH_TRILL_ALVEOLAR`**: rr, a word-initial r, and an r after n, l or s
  (perro, rojo, honra, alrededor, Israel). The Rust gave rr svara's English approximant /ɹ/ and every
  single r the tap; svara 1.0 had no trill. Every other r is still the tap (pero, caro).
- **The IPA tables follow.** `VOWEL_E` is `e` in the English, German, Hindi and Sanskrit tables, where it
  was `iː`; `VOWEL_LONG_I` is `iː` in each; German `VOWEL_Y` is `yː`. The Spanish tap is `ɾ` and the
  trill `r`, IPA's and varna 2.6.0's spelling (the Rust had `r` / `rr`), and `APPROX_R` is `ɹ`. So each
  of these phonemes is found in varna's inventory for the language. Through 3.1.2's deps alone, the old
  table had flagged 157 of the 10,617 words' Spanish rule output for `rr`.
- **Measured**, each list word put through each language's rules before and after:
  - English: 2,094 of the 10,617 words change, all `VOWEL_E` -> `VOWEL_LONG_I`;
  - German: 1,855, `VOWEL_E` -> `VOWEL_Y` (ü, y) or `VOWEL_LONG_I` (ie);
  - Spanish: 428, the tap or `APPROX_R` -> the trill.

  No word changes length, nothing else moves, and validation gains no flag in any language and loses
  the 157 Spanish `rr` ones. Hindi and Sanskrit native-script words are covered by new tests, since the
  list is Latin script. A conversion through the dictionary does not change. Across the 10,855-input
  conversion differential 18 rows change, all sentences or other-language inputs with a word the rules
  spell, and only the trill's duration differs, svara's own for that phoneme.
- **Tests that pinned the Rust's strings now pin the accurate ones.** `tests/rules.tcyr` (+23) and
  `tests/validate.tcyr` (+16) add the cases above, including that every aligned phoneme is in varna's
  inventory and that English still reports /e/ as not its own.
- **CLAUDE.md** states the new bar, and the roadmap's *Accuracy sweep* item lists the stand-ins found
  while doing this and not yet fixed: English `th`, German ö and r, Hindi ए and the retroflex and
  aspirated stops, Arabic ي and the pharyngeals, Sanskrit's `n` against varna's `n̪`, and the
  syllabifier's sonority for the tap and the phonemes svara added later.

### Changed

- **svara 3.9.0–3.14.0 changed synthesis**, so `shabda_speak` output changes too. It was not compared
  sample for sample here. The suites' speak checks pass, and both speak rows run 33–38% faster (see
  *Benchmarks*).
- **`cyrius.cyml` declares `thread`** for `tests/allocator.tcyr` alone. Nothing in `src/` or the
  bundle creates a thread. `lib/` gains the 8 thread and sync leaves, and the lock hashes 61 files.
- **`dist/shabda.cyr`** gains the `_a` forms. `cyrius distlib` now lists 10 `shbdk_*` names for the
  consumer to supply, up from 5. They are all shabdakosh's dictionary surface: `shbdk_dict_new`,
  `shbdk_dict_english`, `shbdk_dict_lookup`, `shbdk_pronunciation_phonemes`, plus `shbdk_is_lower_cstr`,
  `shbdk_dict_lookup_entry`, `shbdk_dict_entries`, `shbdk_dict_user_entries`,
  `shbdk_dict_entry_primary_phonemes` and `shbdk_dict_entry_all`, which the `_a` lookup calls.
  `shbdk_dict_lookup_all` leaves the list, because the conversion now reads the entry itself. There
  is still no `dist/shabda.deps`. The repro builds against the bundle as a consumer would.
- **`build/shabda` grew from 4,842,184 to 5,570,456 bytes.** Unreachable functions are 4.19 MB of it
  (8,349 fns), up from 3.62 MB (7,715), most of that from the new dependency versions.
- **`src/rules.cyr`**: two locals named `a` are renamed (`ca` in `_shabda_english_match`, `ch` in
  `_shabda_german_double`), since `a` is now the Allocator. 20 lines that the longer push call took
  past 120 columns are wrapped.

### Tests

- **`tests/allocator.tcyr`** (new, 9 assertions). It runs in its own process, because its first group
  must come before any conversion: the first `shabda_convert_a` after `shabda_g2p_new` keeps nothing on
  the default heap. Then 300 requests on one reset arena keep nothing there and never grow the arena.
  Then four threads convert at once on one engine, 600 times on their own arenas and 160 times through
  the default heap, and every result matches one converted alone. A scratch soak at 13,600 concurrent
  conversions found no mismatch.
- **`tests/engine.tcyr`** (+95). 16 inputs through `shabda_convert_with_a` on a 0xA5-filled arena:
  pauses, abbreviations, numbers, acronyms, heteronyms, foreign words, unknown words, a phonotactic
  violation, emphasis, a speaking rate, a timing profile, and all five other languages in both scripts.
  Each gives the default-heap form's events, keeps 0 default-heap bytes, and holds exactly the bytes the
  default-heap form kept. Also `shabda_convert_a`, an empty input, and the `_a` lookup against
  shabdakosh's, user overlay included.
- **`tests/prosody.tcyr`** (+4): the event record against svara's. **`tests/normalize.tcyr`** (+5):
  the builder canary under *Fixed*.
- **`cyrius fuzz`**: every case also converts through `shabda_convert_a`, twice on a dirtied arena, and
  must match `shabda_convert` and keep nothing on the default heap the second time.
- **Mutants**: five changes to the new code each fail a test: no table warm-up in `shabda_g2p_new`, the
  event's tone left unset, the lookup key on the default heap, the reserve fix undone, and one vec on
  the default heap.
- **Differential**: the 10,808 convert inputs above go through both forms: every event is identical,
  no input keeps a default-heap byte through `shabda_convert_with_a`, and every arena holds exactly what
  the default-heap form kept. A first run found 249 words, each with a phonotactic violation, whose
  message `str_cstr` had copied onto the default heap. That is fixed, and pinned in
  `tests/engine.tcyr`.

### Benchmarks

`tests/shabda.bcyr`, x86_64. Medians of **8** alternated runs of three builds in one session: 3.1.1,
the new deps with 3.1.1's source, and 3.1.2. This session ran slow: 3.1.1's own rows read about 19%
above the figures recorded for it, so compare within the table only.

| Benchmark | 3.1.1 | new deps, 3.1.1 source | 3.1.2 |
|-----------|-------|------------------------|-------|
| g2p_hello_world | 18.22 µs | 18.45 µs | 18.95 µs |
| g2p_sentence | 72.77 µs | 73.18 µs | 75.27 µs |
| g2p_sentence_arena (new) | — | — | 69.69 µs |
| speak_hello | 6.32 ms | 4.21 ms | 4.18 ms |
| speak_sentence | 20.71 ms | 12.89 ms | 12.94 ms |
| dict_english_construction | 9.40 ms | 9.34 ms | 9.56 ms |
| dict_lookup_hit | 164 ns | 172 ns | 167 ns |
| dict_lookup_miss | 193 ns | 192 ns | 190 ns |

- **The deps make speak 33–38% faster** (svara 3.8.x's per-sample work) and leave the rest within noise.
- **The `_a` refactor costs the default-heap forms at most a few percent.** In 12 more alternated runs of
  the last two builds, in a quieter stretch, `g2p_hello_world` read 16.92 against 17.00 µs (+0.5%) and
  `g2p_sentence` 66.53 against 68.43 µs (+2.9%). Their interquartile ranges overlap, and
  `dict_lookup_hit`, whose code is the same in both builds, moved +2.2%. Each raw allocation now goes
  through `alloc_via`, which adds one indirect call.
- **`g2p_sentence_arena`**, the new row, converts the `g2p_sentence` sentence through `shabda_convert_a`
  into an arena reset before each conversion. It read 64.47 µs in those 12 runs: 3–6% faster than either
  default-heap form.

## [3.1.1] — 2026-11-07

Toolchain and dependency release, plus an audit cleanup. `cyrius` 6.7.9 -> **6.7.18**,
`shabdakosh` 3.1.0 -> **3.1.1**, `svara` 3.6.0 -> **3.6.3**, `varna` 2.5.0 -> **2.5.2**, and
transitively `hisab` 3.4.0 -> **3.4.1**, `naad` 2.3.0 -> **2.3.3**, `goonj` 2.1.0 -> **2.3.1**,
`sakshi` 2.5.8 -> **2.5.10**. Each is its dep's newest tag and pins 6.7.18, and the lock's six
commit pins are the ones shabdakosh 3.1.1 ships. `lib/` and `cyrius.lock` were regenerated from
scratch. Every vendored dep is byte-identical to its tag's `dist/` bundle, every stdlib leaf is
byte-identical to the 6.7.18 snapshot, `cyrius deps --verify` passes 53 of 53, and
`cyrius deps --locked` is clean.

No behavior, API or output change. **727 assertions / 10 suites**, the same as 3.1.0, and
`cyrius fuzz` passes. svara 3.6.3 keeps the consumer floor at cyrius 6.7.2. **`cyrius audit` exits
0**, which it last did at 3.0.3.

### Fixed — `cyrius audit` is clean

- **fmt**: `cyrius fmt` re-indented 3 continuation lines in `tests/engine.tcyr` and
  `tests/syllable.tcyr`, from 4 spaces to 2.
- **lint**: the 14 warnings in `tests/` are gone. 12 assert calls over 120 columns are split before
  their message argument, and a doubled blank line in `tests/engine.tcyr` is removed. The
  271-column expected string for `expand i64::MAX` in `tests/normalize.tcyr` is now built from
  three literals with `str_builder` at run time, the way `tests/engine.tcyr` and `tests/ssml.tcyr`
  already build their nesting inputs. Cyrius cannot split one literal across lines yet:
  adjacent-literal concatenation is still a 6.7.x proposal. The assertion compares the same string.
- **docs**: the 40 undocumented public fns now have doc comments. 37 are in `src/`: engine 12,
  ssml 10, error 4, heteronym 4, normalize 4, syllable 3. Most are ported from the Rust doc comment
  of the same item, and each states the Cyrius encoding: f64 bit patterns, sentinels, and the
  fallback on an unknown name or code. The other 3 are entry points: `main` in `tests/shabda.bcyr`,
  and `fuzz_main` and `main` in `fuzz/shabda.fcyr`.

### Changed

- **`lib/`**: 34 files change, and `simd.cyr` is added because goonj 2.3.1 includes it. shabda
  already vendored `lib/unicode/`, which the 6.7.18 `bayan` now includes.
- **`lib/shabdakosh.cyr` was copied by hand from the 3.1.1 tag**, per ADR-0001, after checking that
  the local tag is the commit GitHub has (`7642b9b`). The 6.7.18 resolver now notes on every resolve
  that this file is outside the declared footprint, because shabdakosh is `optional`. It keeps the
  file and its lock line, so the note is expected.
- **`dist/shabda.deps` is gone**, and `dist/shabda.cyr` no longer opens with the
  `include "lib/unicode/*.cyr"` lines. The 6.7.18 compile-verify finds no stdlib leaf left to name:
  the `unicode` tables now arrive with `bayan`, which svara's sidecar requires and which includes
  `lib/unicode/` at 6.7.18. A consumer gets them through svara, which shabda needs anyway.
  Otherwise the bundle changes only in its version header and the new doc comments.
- **`build/shabda` grew from 4,010,864 to 4,842,184 bytes.** The default build keeps unreachable
  functions, and the unreachable share grew from 3.02 MB (6,920 fns) to 3.62 MB (7,715 fns).
- **`cyrius tests` was removed in cyrius 6.7.12.** CLAUDE.md, README and `state.md` now give
  `cyrius test tests` for running every suite. CI already runs the bare `cyrius test`, which finds
  the same 10 suites.

### Checked — the 6.7.10–6.7.18 meaning changes do not reach shabda

varna 2.5.1 lists what 6.7.10–6.7.18 changed in meaning: struct values, `Str`, multi-value returns,
declaration zones, `f64`, `**T`, closures, `impl`, narrow scalar annotations, duplicate globals and
fns, and include spellings. With comments and strings stripped, `src/`, `tests/` and `fuzz/` have
no `struct`, `enum`, `#derive`, `Str` annotation, `ret2`, `: stack`, `**T`, closure, `impl` or
narrow integer annotation. They have no duplicate global or fn, no fn that shadows a `lib/` one,
and one spelling per include. `f64` goes only through the `f64_*` calls on i64 bit patterns.

The one natively typed spot is the `f32` multiply behind `<break time="Xs">`
(`_shabda_f32_mul` in `src/ssml.cyr`), and 6.7.x changed how an `f32` local is stored. A
differential over 404,075 inputs to `_shabda_secs_to_ms_u32` was built under 3.1.0 (cyrius 6.7.9)
and 3.1.1, and the two outputs are byte-identical. The inputs cover every millisecond from −2 s to
200 s, the same range in sevenths of a second, 64 powers of two divided by three, NaN, ±inf, ±0,
the smallest subnormal and the u32 boundary. The 10,647-input conversion differential from 3.1.0
is byte-identical too.

varna 2.5.2's breaking change is to its `-D MCP` tool surface, and `allophone_realize` now compares
by value. shabda calls neither, and all 12 varna functions it does call exist in 2.5.2.

### Benchmarks

`tests/shabda.bcyr`, x86_64. Medians of **8** runs, with 3.1.0 (cyrius 6.7.9, its `lib/`) and
3.1.1 alternated in one session. Each delta is toolchain and dependencies together. Every row's
range overlaps between versions, so none of these is a measured change. The first three rounds
alone made `dict_lookup_miss` look 27 % slower. The stdlib hashmap seeds itself per process, so a
lookup row moves from run to run, and five more rounds put its medians 3 ns apart.

| Benchmark | 3.1.0 | 3.1.1 | Δ (all overlap) |
|-----------|-------|-------|-----------------|
| g2p_hello_world | 15.29 µs | 15.32 µs | +0.2% |
| g2p_sentence | 60.58 µs | 60.34 µs | −0.4% |
| speak_hello | 5.30 ms | 5.18 ms | −2.4% |
| speak_sentence | 17.39 ms | 16.95 ms | −2.6% |
| dict_english_construction | 7.53 ms | 7.41 ms | −1.6% |
| dict_lookup_hit | 136 ns | 136 ns | noise |
| dict_lookup_miss | 152 ns | 155 ns | noise (3.1.1 ranged 148–525 ns) |

## [3.1.0] — 2026-11-07

Toolchain and dependency release, moving to the cyrius 6.7 language minor. `cyrius` 6.6.20 ->
**6.7.9**, `shabdakosh` 3.0.8 -> **3.1.0**, `svara` 3.5.6 -> **3.6.0**, `varna` 2.4.3 -> **2.5.0**,
and transitively `hisab` 3.3.7 -> **3.4.0**, `naad` 2.2.5 -> **2.3.0**, `goonj` 2.0.6 -> **2.1.0**,
`sakshi` 2.5.7 -> **2.5.8**. This is the 6.7.9 batch. svara 3.6.0, varna 2.5.0, hisab 3.4.0, naad
2.3.0 and goonj 2.1.0 are the tags pinned to exactly 6.7.9, and the next tag of each needs 6.7.18.
shabdakosh 3.1.0 ships the same set, and its lock carries the same six commit pins. `lib/` and
`cyrius.lock` were regenerated from scratch. Every vendored dep is byte-identical to its tag's
`dist/` bundle, every stdlib leaf is byte-identical to the 6.7.9 snapshot, `cyrius deps --verify`
passes 52 of 52, and `cyrius deps --locked` confirms that `lib/` and the lock are exactly what the
tags resolve.

It is a minor because the floor for consumers moves: svara 3.6.0 needs cyrius 6.7.2 or later.
Output changes for one word, which comes from shabdakosh (see *Fixed*).

No source change, no API change. **727 assertions / 10 suites**, the same as 3.0.6; `cyrius fuzz`
passes. `src/` stays fmt- and lint-clean, and `cyrius vet` and `cyrius deny` are clean.

### Fixed

- **`portuguese` now comes from the dictionary.** shabdakosh 3.0.8's generator stored the
  one mixed-case CMUdict entry as `Portuguese`, which a lowercasing lookup never reaches, so shabda
  fell back to the English letter-to-sound rules. Rust's `build.rs` lowercases, so the oracle
  always took the dictionary path. shabdakosh 3.1.0 fixes the generator. A differential over the
  10,617 words of shabdakosh's `data/cmudict-5k.txt`, plus 30 sentences and other-language inputs,
  was built under 3.0.6 (cyrius 6.6.20) and under 3.1.0. Only the 3 `portuguese` rows differ: the
  9 rule-derived events become the dictionary's 8. Every other row is byte-identical, durations
  included.

### Changed

- **`lib/`**: 31 of its 52 files change and the file set does not: nothing is added or dropped.
- **`lib/shabdakosh.cyr` was copied by hand from the 3.1.0 tag**
  (`git -C ../shabdakosh show 3.1.0:dist/shabdakosh.cyr`), per ADR-0001, after checking that the
  local tag is the commit GitHub has (`88cea94`).
- **`cyrius.lock`**: the 6 commit pins move to the 6.7.9 batch, and the toolchain row is
  `cyrius 6.7.9`. Re-resolving into the existing lock appends the new commit rows beside the old
  ones (it held 12), so the lock was regenerated from an empty file.
- **`lib/sakshi.cyr` is 2.5.8**, the 6.7.9 snapshot's copy, byte-identical to the sakshi 2.5.8
  artifact the dep chain pins, so nothing is split. The `refusing to overwrite stdlib leaf 'sakshi'`
  note still prints on every resolve, as expected.
- `dist/shabda.cyr` changes only in its version header. `dist/shabda.deps` still names only
  `unicode`.

### Checked — the 6.7.x meaning changes do not reach shabda

6.7.x changes what some existing code *means*, not just what compiles. As varna 2.5.0 did,
`src/`, `tests/` and `fuzz/` were swept with comments and string literals stripped. They contain no
lone `!` (a real operator since 6.7.3; before that `if (!ok)` meant `if (ok)`), no unary `-` or `~`
(the 6.7.9 binding change), no `for` loops (the 6.7.8–6.7.9 step rule), no `*T` derefs or slices
(6.7.8's element width), and none of the newly reserved words (`trait`, `const`, `true`, `false`,
`do`, `dyn`). shabda negates as `0 - x` and loops with `while`.

### Note — `cyrius distlib` lists 5 `shbdk_*` names for the consumer to supply

Unchanged from 3.0.6: `shbdk_dict_new`, `shbdk_dict_english`, `shbdk_dict_lookup`,
`shbdk_dict_lookup_all` and `shbdk_pronunciation_phonemes`, supplied by the consumer's
`… → varna → shabdakosh → shabda` chain. Expected, and not a defect.

### Known issue — `cyrius audit` exits 1

Unchanged from 3.0.6, leg for leg: **fmt** (`tests/engine.tcyr`, `tests/syllable.tcyr`), **lint**
(14 warnings, all in `tests/`), **docs** (40 undocumented public fns). CI does not run
`cyrius audit`.

### Benchmarks

`tests/shabda.bcyr`, x86_64. Medians of 3 runs, with 3.0.6 (cyrius 6.6.20, its committed `lib/`)
and 3.1.0 alternated in one session. Each delta is toolchain and dependencies together, not either
one alone. `g2p_hello_world` and `speak_hello` were slower in every 3.1.0 run than in every 3.0.6
run. The other rows overlap between versions, and the lookup rows are sub-µs noise (3.0.6's
`dict_lookup_miss` alone ranged 141–219 ns).

| Benchmark | 3.0.6 | 3.1.0 | Δ |
|-----------|-------|-------|---|
| g2p_hello_world | 15.05 µs | 15.18 µs | +0.9% |
| g2p_sentence | 58.84 µs | 59.72 µs | +1.5% (overlaps) |
| speak_hello | 5.17 ms | 5.32 ms | +2.8% |
| speak_sentence | 16.94 ms | 17.16 ms | +1.3% (overlaps) |
| dict_english_construction | 7.35 ms | 7.47 ms | +1.7% (overlaps) |
| dict_lookup_hit | 135 ns | 133 ns | noise |
| dict_lookup_miss | 147 ns | 133 ns | noise |

## [3.0.6] — 2026-11-07

Toolchain and dependency release. `cyrius` 6.6.3 -> **6.6.20**, `shabdakosh` 3.0.5 -> **3.0.8**,
`svara` 3.5.5 -> **3.5.6**, `varna` 2.4.2 -> **2.4.3**, and transitively `hisab` 2.22.1 -> **3.3.7**,
`naad` 2.2.2 -> **2.2.5**, `goonj` 2.0.4 -> **2.0.6**, `sakshi` 2.5.2 -> **2.5.7**. This is the set
matched to 6.6.20, the same one shabdakosh 3.0.8 ships: svara 3.5.6 and varna 2.4.3 are the last
tags on 6.6.x, and their next tags need 6.7.x. svara's manifest supplies the hisab / naad / goonj
versions. `lib/` and `cyrius.lock` were regenerated from scratch (`cyrius deps` into an empty
`lib/`). Every vendored dep is byte-identical to its tag's `dist/` bundle, every stdlib leaf is
byte-identical to the 6.6.20 snapshot, and `cyrius deps --verify` passes 52 of 52.

No source change, no API change. **727 assertions / 10 suites**, the same as 3.0.5; `cyrius fuzz`
passes.

### Fixed

- **The svara and varna tag pins were inert.** `[deps.svara]` and `[deps.varna]` set `path`
  beside `git` and `tag`, and an existing `path` makes both inert: `cyrius deps` copies from the
  sibling checkout with no clone and no commit pin, and every `cyrius build` / `test` / `tests`
  resolves again. So 3.0.5 vendored svara 3.5.5 and varna 2.4.2 while the manifest said 3.5.4
  and 2.4.1, and the lock had no svara or varna row. The siblings now sit at 3.6.3 and 2.5.2 on
  cyrius 6.7.18, so running the suites would have pulled the 6.7.x chain into `lib/`.
  `path` is dropped from all three deps, and svara and varna now resolve from their tags with a
  commit pin. shabdakosh is `optional` and never resolved through its `path`; that line goes for
  consistency.
- **3.0.4's sakshi split is gone.** The 6.6.20 resolver keeps the stdlib snapshot's
  `lib/sakshi.cyr` and skips the dep artifact. Both copies are 2.5.7 here, so nothing is split.
  `cyrius build` / `test` / `deps` print a `refusing to overwrite stdlib leaf 'sakshi'` note on
  every resolve. That note is expected, and shabdakosh 3.0.8 prints it too.
- **Assertion count.** README and `state.md` said 737, but the per-suite counts they list add up
  to 727, which is what the suites report. Both now say 727.

### Changed

- **`dist/shabda.deps`: 22 leaves -> 1 (`unicode`).** Since cyrius 6.6.18 a sidecar lists only
  what the compile-verify proves the bundle needs beyond its named deps. `dist/shabda.cyr` now
  opens with the `include "lib/unicode/*.cyr"` lines for that leaf.
- **`lib/`**: `hashmap_fast.cyr` dropped (not declared; nothing includes it). `alloc_cx.cyr`,
  `chrono.cyr` and `poison.cyr` added, because the 6.6.x stdlib leaves now include them. The lock
  gains the svara and varna commit pins and a `cyrius 6.6.20` toolchain row.
- **`lib/shabdakosh.cyr` was copied by hand from the 3.0.8 tag**
  (`git -C ../shabdakosh show 3.0.8:dist/shabdakosh.cyr`), per ADR-0001. It was not copied from
  `../shabdakosh/dist/`: that checkout is on 3.1.1 / cyrius 6.7.18, so the `cp` that ADR-0001
  documents would vendor the wrong version.
- svara's bundle no longer prints `#inline ignored: body exceeds 32 tokens` warnings.

### Note — `cyrius distlib` lists 5 `shbdk_*` names for the consumer to supply

`shbdk_dict_new`, `shbdk_dict_english`, `shbdk_dict_lookup`, `shbdk_dict_lookup_all` and
`shbdk_pronunciation_phonemes` are shabdakosh's dictionary surface, which `src/engine.cyr` and
`src/heteronym.cyr` call (dictionary-first lookup, heteronym pronunciations). `dist/shabda.cyr`
does not carry shabdakosh. A consumer supplies it from the chain it declares and includes in
dependency order, `… → svara → varna → shabdakosh → shabda`, as dhvani does. This is expected
and not a defect.

### Known issue — `cyrius audit` exits 1

This predates the bump: 3.0.5 on 6.6.3 already failed three legs. **fmt**: `tests/engine.tcyr`
and `tests/syllable.tcyr` need reformatting. **lint**: 14 warnings, all in `tests/`, mostly lines
over 120 columns. **docs**: 3 undocumented public fns on 6.6.3, now **40** on 6.6.20, because
cyrdoc stopped counting `#must_use` as documentation (shabdakosh 3.0.8 hit the same thing). On
both, `src/` is fmt- and lint-clean. CI does not run `cyrius audit`.

### Benchmarks

`tests/shabda.bcyr`, x86_64. Medians of 3 runs, with 3.0.5 (cyrius 6.6.3, its committed `lib/`)
and 3.0.6 alternated in one session:

| Benchmark | 3.0.5 | 3.0.6 | Δ |
|-----------|-------|-------|---|
| g2p_hello_world | 16.51 µs | 15.42 µs | −6.6% |
| g2p_sentence | 64.35 µs | 61.50 µs | −4.4% |
| speak_hello | 5.51 ms | 5.38 ms | −2.4% |
| speak_sentence | 17.94 ms | 17.64 ms | −1.7% |
| dict_english_construction | 7.84 ms | 7.64 ms | −2.6% |
| dict_lookup_hit | 138 ns | 141 ns | noise |
| dict_lookup_miss | 167 ns | 185 ns | noise (3.0.5 alone ranged 147–298 ns) |

## [3.0.5] - Unblocked by cyrius 6.6.3

Toolchain-pin release. `cyrius` 6.6.2 -> **6.6.3**; no source change.

6.6.2 could not compile this project. `#inline` became a real directive at cyrius
v6.5.63, which taught the compiler's PASS 2 to arm the directive — but PASS 1, the
declaration-collection scan, was never taught to CONSUME the token. An unconsumed
directive there falls through to the catchall and **terminates the scan**, so every
declaration after the first `#inline` went unregistered and the next `#derive`d
struct reached the parser as an unknown top-level token. The error named the struct,
which was innocent, hundreds of lines from the real trigger.

cyrius 6.6.3 closes both halves in all seven per-target compiler forks. Re-vendored
and rebuilt against it; tests green.

## [3.0.4] — 2026-09-01

Toolchain pin to cyrius **6.5.37**, a sidecar correction that unblocks consumers, and a repair
to this repo's own release tooling.

### Fixed — `dist/shabda.deps` named three VENDORED deps as stdlib leaves

The sidecar listed `hisab`, `goonj` and `naad` among its stdlib requirements. None is a cyrius
stdlib module — they are vendored dependencies reached through `src/main.cyr`, the umbrella
`cyrius distlib` scans for `include "lib/X.cyr"` lines. The scan correctly captured "these must
be in scope" but tagged them as *stdlib*, so a consumer's `cyrius deps` hunted for them in the
stdlib and failed outright:

```
error: dep shabda requires 'hisab' but it is not in the cyrius stdlib
```

⭐ **Over-reporting is the fatal direction.** An under-reported sidecar fails silently at the
consumer's compile step; an over-reported one is a HARD resolver error *before* compilation is
reached — the same shape that left `crab` and `puka` unable to resolve `dhancha` at all.

Fixed upstream in cyrius 6.5.37, which validates every captured leaf against the stdlib snapshot
at generation time, so the names are dropped by construction. Regenerating picks that up: the
sidecar is now **23 leaves, all of which resolve**.

⚠ No source change was needed or made. `src/main.cyr`'s includes are correct — they carry the
dependency chain the bundled modules require. Only the published metadata was wrong.

### Fixed — `scripts/version-bump.sh` had been BROKEN since the Rust port

It was still the Rust-era script: after writing `VERSION` it ran `sed -i … "$REPO_ROOT/Cargo.toml"`
followed by `cargo generate-lockfile`. `Cargo.toml` now lives under `rust-old/`, not the repo
root, so under `set -euo pipefail` the sed failed and the script **died at that line** — writing
`VERSION` and never regenerating the bundle. Any bump run through it left `dist/shabda.cyr`
stamped with the *previous* version while `VERSION` claimed the new one.

It failed loudly, which is the only reason this was caught; a `|| true` on that line would have
made it silent. Rewritten cyrius-native (write `VERSION`, then `cyrius distlib --all`), matching
shabdakosh's. `--all` rather than bare `distlib` so profile bundles cannot be left behind — the
sankoch 2.7.6 shape.

### Changed

- `cyrius` pin **6.5.36 → 6.5.37**.
- `lib/` re-synced from the 6.5.37 snapshot (42 files).
- `dist/shabda.cyr` + `dist/shabda.deps` regenerated.

### Known issue — `lib/sakshi.cyr` stays at 2.4.11

The pinned 6.5.37 snapshot ships sakshi **2.4.12** (fixing a negative-depth buffer underflow in
`sakshi_span_enter`), and `cyrius lib sync` writes it correctly — but the next `cyrius deps`
reverts it to **2.4.11**, byte-identical to a named dep's own vendored copy. The resolver pulls a
declared *stdlib* leaf from the dep's `lib/` instead of the pinned snapshot, so the two commands
fight and `deps` wins.

A cyrius resolver defect, not a shabda one, filed upstream as
`cyrius/docs/development/issues/2026-09-01-deps-pulls-stdlib-leaf-from-dep-lib.md`. shabda is the
**second** independent repo confirmed hitting it (shabdakosh 3.0.6 is the first). It does not
affect this release: the build is clean and all 198 tests pass.


All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [3.0.3] — 2026-09-01

Priority-1 audit sweep: correctness, hardening, security, parity and performance,
plus the two `cyrius audit` gaps 3.0.2 shipped with. **13 confirmed defects fixed**,
each found by reading against `rust-old/`, adversarially re-verified, and pinned by a
regression test. **737 assertions across 10 suites** (was 653), fuzz corpus rebuilt,
`cyrius audit` exits 0. No API removals; three public functions changed behaviour and
are called out below.

### Fixed — CRITICAL: a 6 KB word allocated 858 MB, permanently

`_shabda_max_legal_onset` (`src/syllable.cyr`) walked `start` from 0 to `len` and
materialised a **fresh vec on every iteration** where the oracle borrows a sub-slice:

```
rust-old/src/syllable.rs:165   for start in 0..cluster.len() {
rust-old/src/syllable.rs:166       let candidate = &cluster[start..];      // O(1), no alloc
src/syllable.cyr:171                var candidate = _shabda_slice(cluster, start, len);
```

The bump allocator never frees, so that is ~21.6·L² bytes retained for a consonant run
of length L. Measured end to end on `shabda_convert(eng, "a" + "b"×6000 + "a")` — one
6,002-byte argument, no length ceiling anywhere between the public entry and here:

| consonant run | 3.0.2 | 3.0.3 |
|---|---|---|
| 500 | 5,461,304 B | 150,600 B |
| 1,000 | 21,702,424 B | 291,944 B |
| 2,000 | 86,559,424 B | 574,608 B |
| 4,000 | 345,772,560 B | 1,139,936 B |
| **6,000** | **858,241,376 B** | **2,188,592 B** |

Quadratic (4× per doubling) → linear (2× per doubling); **392× less at 6,000**. Output
is unchanged — 6,002 events either way. The scan is now range-based
(`_shabda_max_legal_onset_range` / `_shabda_is_legal_onset_range` take bounds instead of
a copy), which also removes the two `_shabda_slice` copies in `shabda_syllabify` where
Rust borrows. Reachable from `convert` / `convert_with` / `convert_ssml` /
`convert_streaming` / `speak*` and from `shabda_syllabify` directly. Streaming does not
help: the blow-up is *within* one whitespace token.

### Fixed — HIGH: nested SSML took the process down with SIGSEGV

`_shabda_parse_until` ↔ `_shabda_parse_tag` (`src/ssml.cyr`) are mutually recursive once
per element, with no ceiling — depth is whatever the caller's string says. The
unknown-tag arm means any tag name recurses, not just the four known ones.

```
shabda_ssml_parse(<emphasis> ×19,437 + "x" + </emphasis> ×19,437)   # ~408 KB
  -> SIGSEGV (exit 139) on the default 8 MB stack, ~432 B of frame per level
```

Bisected to the byte: depth 19,375 survives, 19,437 does not. `shabda_convert_ssml`
reaches it too, and `shabda_ssml_node_clone` / `_shabda_clone_children` recurse over the
same tree. `rust-old/src/ssml.rs:239` has the identical shape and the identical crash,
so **capping is a deliberate divergence from the oracle** rather than a parity fix.
`SHABDA_SSML_MAX_DEPTH = 256` — what libxml2 defaults to, and ~50× deeper than any real
SSML document. Past it the parse fails as `SHABDA_SSML_PARSE_ERR`, which
`shabda_ssml_parse` reports as 0 and `shabda_convert_ssml` as `InvalidInput`. A document
nested exactly 256 deep parses; 257 does not. A 4.2 MB, 200,000-deep document now
returns an error instead of killing the host.

### Fixed — the port was ASCII-only everywhere Rust was Unicode

Five hand-rolled predicates approximated Unicode with ASCII (or ASCII + Latin-1). Each
is now the real property, and the three byte-identical whitespace copies collapsed into
one L0 definition (`shabda_is_ws_cp` / `shabda_ws_len_at` / `shabda_ws_len_before` in
`src/error.cyr`).

- **`char::is_whitespace`** — was `{space, tab, LF, CR, FF, VT}` in `normalize.cyr`,
  `ssml.cyr` and `engine.cyr`; now the full White_Space property (25 code points,
  written out — no table needed). Consequences, all measured:
  - `shabda_normalize("hello\u{00A0}world")` returned `hello\u{00A0}world` as **one
    token** and sent it to the letter-to-sound rules whole. Now `hello world`.
  - `shabda_convert(eng, "\u{3000}")` returned **Ok with 0 events** where Rust's
    `if text.trim().is_empty()` returns `Err(InvalidInput)`. Same for `convert_with`,
    `convert_ssml`, `convert_streaming` and the `render_ssml_nodes` text guard.
  - `shabda_detect_intonation("really?\u{00A0}")` read as a **Statement** — the
    ASCII-only `trim_end` left `0xA0` as the last byte and never saw the `?`.
  - An SSML tag whose name was separated from its attributes by U+00A0 was parsed as a
    tag named `break\u{00A0}time=...` and silently dropped.
- **`char::is_alphabetic`** — was five hard-coded ranges (ASCII, Latin-1 Supplement +
  Extended-A/B, Devanagari, two Arabic blocks). Every other script hit **no arm**, so
  `shabda_normalize` **deleted it**: Greek, Cyrillic, Hebrew, Thai, Han and Hangul input
  normalized to empty and produced zero phonemes. Now L\* + Nl from the stdlib's Unicode
  17 GeneralCategory table, plus the Devanagari and Arabic mark ranges for the
  Other_Alphabetic segments the table does not carry. The scripts that already worked are
  unchanged — that is asserted, because dropping them is the regression the module header
  warns about.
- **`char::to_lowercase().next()`** — was ASCII-only, so `"CAFÉ"` lowered to `"cafÉ"` and
  every accented capital reached the rules in the wrong case. Now the stdlib casefold
  table.
- **`char::is_uppercase`** — was ASCII + U+00C0..U+00DE, so ALL-CAPS emphasis was
  silently not applied to Greek, Cyrillic, or even Latin Extended-A. Now GeneralCategory
  `Lu`. (Residual: `Uppercase` is `Lu` + Other_Uppercase; the ~130 Other_Uppercase points
  are Roman-numeral forms and circled capitals, none of which shabda pronounces.)

`[deps].stdlib` gains **`unicode`** for the two tables. It does not grow
`dist/shabda.cyr` — stdlib leaves are named in the sidecar, not bundled — but consumers
must have the leaf in scope.

### Fixed — `<break time=...>` left Rust's `u32` domain

`_shabda_parse_break_duration` (`src/ssml.cyr`) returns a duration through the same
channel as its failure sentinel, and neither bound nor saturated it.

- **`Nms` had no `u32` ceiling.** Rust does `ms_str.parse::<u32>()`; the port accumulated
  into an i64 with no limit, so `<break time="5000000000ms">` returned 5,000,000,000
  where Rust returns `Err`, and a 20-digit run **wrapped the i64**. Now
  `_shabda_atoi_u32`, which rejects empty, non-digit and `> u32::MAX`.
- **`Xs` did not saturate.** Rust's `(secs * 1000.0) as u32` is a saturating cast:
  NaN → 0, negative → 0, above `u32::MAX` → `u32::MAX`. The port used a bare `f64_to`,
  so `time="-5s"` produced −5000 — and `time="-0.001s"` produced exactly **−1, which IS
  `SHABDA_SSML_PARSE_ERR`**, making a legal parse indistinguishable from a failure.
- **The multiply is now native single precision**, because Rust's is (`parse::<f32>()`
  then an f32 multiply). Typing the params `f32` puts the patterns in xmm lanes and emits
  `mulss`; the result returns as its bit pattern through an `i64`. Same shape
  [ganita 1.2.0](https://github.com/MacCracken/ganita) uses — see the note at the end.

### Fixed — the UTF-8 decoder in `rules.cyr` read past the terminator

`_shabda_decode_chars` loaded continuation bytes at `word+i+1..+3` with **no bound**, so
a lead byte at the end of a string ran up to 3 bytes past the NUL into whatever the bump
allocator held there — a heap over-read whose phoneme output depended on neighbouring
allocations. Its sibling `_shabda_utf8_next` (`normalize.cyr`) always bailed at the NUL;
the crate's two decoders disagreed. Both now degrade a truncated sequence to its lead
byte and resume at the next, and `tests/shabda.tcyr` asserts they agree byte for byte
across ASCII, Devanagari, Arabic and all three truncation shapes.

### Fixed — `ConvertOptions` builders aliased the caller's block

Rust's `with_emphasis` / `with_speaking_rate` / `with_timing` take `mut self` and return
`Self` **by value**, so the receiver is moved and cannot be reused. The port mutated the
caller's block and returned the same pointer:

```
var base = shabda_convert_options_new();
var fast = shabda_convert_options_with_speaking_rate(base, wpm200);
var slow = shabda_convert_options_with_speaking_rate(base, wpm100);
# base, fast and slow were ONE block, all at 100 wpm
```

That program does not compile in Rust, so there was no oracle behaviour to preserve.
Each builder now clones first. All three are `#must_use`, so discarding the returned
handle was already an error. **Behaviour change for any consumer that relied on the
in-place mutation and ignored the return value.** The engine's internal
`_shabda_engine_opts_with_emphasis` / `_with_rate` already cloned, so nested SSML was
never affected.

### Fixed — five loops allocated where the oracle mutates in place

`shabda_apply_emphasis`, `shabda_apply_rate`, `shabda_apply_timing` (`prosody.cyr`) and
`_shabda_engine_apply_emphasis_range`, `_shabda_engine_destress_range` (`engine.cyr`)
minted a replacement `SvPhonemeEvent` per mutated element, out of a bump allocator that
never frees, where Rust does `event.stress = …; event.duration *= …` over a
`&mut [PhonemeEvent]`. A comment claimed this was forced ("immutable-by-handle"); svara
has had `SvPhonemeEvent_set_duration` / `_set_stress` all along.

It is worst under nested `<emphasis>`, where every level re-walks the whole event tail
its children produced — so the allocations are depth × events. Measured on a 26-event
body:

| nesting depth | 3.0.2 | 3.0.3 |
|---|---|---|
| 16 | 35,320 B | 31,224 B |
| 32 | 44,152 B | 35,960 B |
| 64 | 61,816 B | 45,432 B |
| 128 | 97,144 B | 64,376 B |

### Fixed — hardening

- **Every input-proportional `alloc()` is checked for the 0 sentinel** (8 sites across
  `engine.cyr`, `normalize.cyr`, `ssml.cyr`). `alloc` returns 0 for a request over
  `ALLOC_MAX` (2 GiB) or on exhaustion; each was written through immediately afterwards.
- **`shabda_select_phonemes` no longer kills the host process on an empty pronunciation
  vec.** Rust's fallback arm is `pronunciations[0]`, which panics; here `vec_get(v, 0)`
  reaches `lib/vec.cyr`'s `_vec_die()` → `syscall(SYS_EXIT, 1)`, which does not unwind and
  does not return a code — it terminates the **host**, so a consumer like dhvani would
  lose its audio daemon. Reachable: `shbdk_dict_lookup_all` returns
  `shbdk_dict_entry_all(e)` verbatim and a caller-built entry can hold zero
  pronunciations. It now returns an empty vec, which the caller's rules fallback already
  handles. A deliberate divergence, in the direction of the crate's zero-panic contract.

### Fixed — the test suites could not report failure on AGNOS

Every `.tcyr`, the `.bcyr` and the `.fcyr` exited with a raw `syscall(60, rc)`. #60 is
`exit` only on x86-64 Linux/macOS; on AGNOS it is a different call, so
`assert_summary()`'s pass/fail result was **discarded** and CI would read the suite green
while assertions failed. All twelve entry files now use the target-resolved `SYS_EXIT`,
matching `src/main.cyr`. Verified by breaking an assertion on purpose: the suite exits 1.

### Changed — the fuzz harness now fuzzes

`fuzz/shabda.fcyr`'s entry point was fine; its driver was **two string literals**, and
could not reach a single defect in this release. It now runs three things: a fixed
**corpus** of every shape that produced a defect here (kept as permanent regressions —
truncated UTF-8, Unicode whitespace, the `<break>` domain edges, malformed markup, deep
nesting on both sides of the cap, long single tokens, i64-boundary numbers), 400 rounds
of **random bytes** including invalid UTF-8, and 400 rounds of **random markup**
assembled from SSML fragments so the parser meets half-formed and mismatched structure
rather than only noise. The generator is a seeded xorshift, so a CI failure is
reproducible from its iteration number.

### Changed — tests

**737 assertions, 10 suites** (was 653): ssml 45 → 66, engine 133 → 151, normalize
40 → 60, rules 126 → 133, syllable 47 → 55, shabda 29 → 37, heteronym 21 → 23. Every fix
above has a regression that fails on 3.0.2, including the two that previously killed the
process — reaching those assertions at all is the test.

### Changed — `cyrius audit` exits 0

3.0.2 shipped with two gaps. `src/ssml.cyr` and `src/heteronym.cyr` needed
`cyrius fmt` (continuation indent), and `main` in `src/main.cyr` was undocumented.

### Performance

Flat. `cyrius bench tests/shabda.bcyr`: `g2p_hello_world` 15.75 µs, `g2p_sentence`
61.4 µs, `speak_hello` 7.40 ms, `speak_sentence` 23.7 ms, `dict_english_construction`
7.62 ms, `dict_lookup_hit` 123 ns, `dict_lookup_miss` 258 ns — all within the ±3-4%
run-to-run band established in 3.0.2. The wins above are in *allocation*, which the
benchmark set does not probe; they were measured with `alloc_used()` deltas instead.

### Note — the f32→f64 widening is svara's to undo, not shabda's

`docs/development/state.md` has listed "f32 prosody/rate scalars widened to f64" as a
port decision since 3.0.0. It was not a decision — it was a toolchain limitation, and
that limitation is gone. This release restores native f32 where shabda **owns** the whole
computation (the `<break time="Xs">` multiply above), and files the rest honestly:

`svara`'s `SvPhonemeEvent.duration` is `f64` and its base durations are f64 constants
(`0.08`, `0.12`, `0.18`), where the Rust svara used `f32` throughout. So the widening is
**chain-wide and owned by svara**. shabda rounding its own prosody arithmetic to f32
would produce a third set of numbers — matching neither the oracle nor the current stack
— so it is deliberately NOT done here. The public encoding (`with_speaking_rate` and
`shabda_timing_new` take f64 bit patterns) is unchanged; restoring f32 end to end is a
svara-first change and a breaking one for this surface.

The adjacent gap **was** fixed upstream: `ganita_f32_add` / `_sub` / `_mul` / `_div`
shipped in **ganita 1.2.0**, and the cyrius issue that recorded "no f32 tier can be
written in native single-precision arithmetic today" has been amended — a function
parameter can simply be typed `f32`, which is what the original filing missed.

## [3.0.2] — 2026-09-01

Dependency + toolchain pin refresh over 3.0.1. No shabda source change — the
distlib bundle is byte-identical apart from its version line, and the
653-assertion suite (10 `.tcyr` files) plus the fuzz harness pass unchanged.

- **Changed**: toolchain pin 6.4.12 → **6.5.36**, aligning shabda with the rest of
  the chain (shabdakosh and varna are already there; svara is on 6.5.35).
- **Changed**: shabdakosh 3.0.2 → **3.0.5** (a HIGH-severity corrupt-text-export
  fix, a `prefix_search` stack-overflow DoS, deep-cloning `merge`, O(n log n)
  dictionary sorts, and two `rust-old/` parity gaps), svara 3.1.0 → **3.5.4** (two
  memory-safety repairs that made every sample rate in (1000, 7500] abort), varna
  2.0.0 → **2.4.1** (tone/features/numerals modules upstream; four data-accuracy
  fixes). The transitive folds move with them: hisab 2.6.7 → **2.11.2**, naad
  2.1.1 → **2.2.1**, goonj 2.0.0 → **2.0.4**, sakshi 2.4.2 → **2.4.11**.
- **Changed**: `[deps].stdlib` gains `slice` + `result` (varna 2.4.1's sidecar) and
  `callback` (new at hisab 2.11.2 / naad 2.2.1). None is reached from `src/` — they
  are leaves the folds need in scope.
- **Changed**: `[deps.shabdakosh]` is declared `optional = true`. shabdakosh's
  `dist/shabdakosh.deps` names hisab / goonj / naad, which `cyrius deps` resolves
  only against the stdlib snapshot; at 6.4.12 the three errors were non-fatal, at
  6.5.36 they exit 3 and take down `deps` / `build` / `test` / `bench` / `distlib`.
  shabdakosh is still mandatory and `lib/shabdakosh.cyr` is still committed —
  refreshing it is now a manual copy. Rationale, the five workarounds that do *not*
  work, and the upstream fix that retires this:
  [ADR-0001](docs/adr/0001-shabdakosh-declared-optional.md).
- **Fixed**: `cyrius.lock` regained its commit pins. The resolver had been erroring
  out before writing them since at least 3.0.1, so the committed lock still named
  hisab 2.6.7 / naad 2.1.1 / goonj 2.0.0 / sakshi 2.4.2 — svara **3.1.0**'s pins,
  left behind by the 3.0.0-era resolve. It now pins the four commits actually in
  `lib/`, and `cyrius deps --verify` passes 41/41.
- **Changed**: `dist/shabda.deps` carries the full 22-leaf stdlib set instead of
  just the three folds — a 6.5.36 distlib-generator change, the same one shabdakosh
  saw at 3.0.3. hisab/goonj/naad still lead the list, so consumers on 6.5.36
  (dhvani, vansh) need the ADR-0001 workaround against shabda in turn.
- **Performance**: flat to slightly better, measured on one machine in one session
  (3.0.1 on cycc 6.4.12, 3.0.2 on 6.5.36 — each delta is toolchain and dependencies
  together). `dict_lookup_hit` 135 → **123 ns**, `dict_lookup_miss` 258 → **237 ns**,
  `dict_english_construction` 7.47 → **7.17 ms**; `g2p_hello_world` 15.79 → 15.37 µs,
  `g2p_sentence` 61.5 → 60.0 µs, `speak_hello` 7.29 → 7.28 ms, `speak_sentence`
  23.69 → 23.44 ms. Only the dictionary rows are clearly outside run-to-run variance,
  which a repeat run put at ±3-4% on the g2p/speak rows.
  `docs/benchmarks-rust-vs-cyrius.md` keeps its 2026-07-06 numbers
  and now says so — its ratios are a ceiling until the Rust half is re-run.

## [3.0.1] — 2026-07-06

Dependency + toolchain pin refresh over the 3.0.0 port. No shabda API or behavior
change; the 653-assertion suite passes unchanged.

- **Changed**: svara pin 3.0.1 → **3.1.0** (control-rate glide synthesis) and
  shabdakosh 3.0.1 → **3.0.2** (which re-pins svara 3.1.0). shabda picks up svara's
  faster diphthong synthesis end to end — **`speak` of a diphthong word (e.g.
  "hello") drops ~3× (22.4 ms → 7.65 ms)**; convert/g2p is unchanged (render-independent).
- **Changed**: toolchain pin 6.4.11 → **6.4.12** (current release; aligns the whole
  shabda/shabdakosh/svara chain on one toolchain, removes drift).

## [3.0.0] — 2026-07-06

Complete port of shabda from Rust to the **CYRIUS** language (AGNOS toolchain). A
full-parity port: every Rust module reproduced against the preserved `rust-old/`
oracle and cross-checked by a 653-assertion suite across 11 test suites, plus a
consumer-verified distlib bundle (`dist/shabda.cyr`).

- **Breaking**: Language change — shabda is now a CYRIUS (`.cyr`) library, not a Rust crate. The API is flat, `shabda_`-prefixed C-style functions (`shabda_g2p_new`, `shabda_convert`, `shabda_speak`, …) rather than Rust methods/traits/generics. Consumers pull `dist/shabda.cyr`.
- **Breaking**: Errors are **sakshi** packed-i64 codes (`0 == ok`) instead of `thiserror` enums; the Rust `ShabdaError` variants became `SHABDA_ERR_*` codes with `shabda_err_name()` diagnostics, and the `From<ShabdakoshError>` map became a code translation. Fallible functions return a packed error (test with `shabda_is_err`) and write their result to an out-param pointer.
- **Breaking**: Traits → dispatch. `Option<T>` → sentinels; `Result<T>` → pointer-or-0 / out-param; `convert_streaming` takes an `fnptr` callback instead of a Rust closure; `f32` prosody scalars widened to `f64`. No serde derive — serialization is hand-written where needed.
- **Breaking**: Cargo feature flags (`std`, `varna`, `json`, `logging`, `full`) **collapse** — CYRIUS has no feature flags, so varna phoneme-inventory validation, language detection, and every other capability are always compiled into the one bundle.
- **Feature**: full G2P pipeline — normalize → dictionary lookup (shabdakosh) / letter-to-sound rules fallback → syllabify (Maximal Onset Principle) → prosody (stress / rate / timing) → svara `PhonemeEvent`s. `shabda_convert` / `convert_with` / `convert_ssml` / `convert_streaming`, plus `shabda_speak` / `speak_with` (renders audio via svara).
- **Feature**: languages — English (dictionary + letter-to-sound rules), Spanish / German / Hindi / Arabic / Sanskrit (rules; native scripts + romanized fallback). `shabda_detect_language` picks a language from script over varna's ranges.
- **Feature**: normalization — abbreviation expansion (Dr.→doctor), acronym pronounceability heuristic (NASA→word, FBI→spelled out), number-to-words, foreign-word diacritic detection, and emphasis markers (ALL-CAPS / `*asterisk*`).
- **Feature**: prosody & SSML — syllable-weight stress, emphasis, speaking-rate clamp (50–300 WPM), a `TimingProfile` for independent vowel/consonant/pause scaling, intonation mapping, and an SSML subset parser (`<break>`, `<emphasis>`, `<prosody>`).
- **Feature**: heteronym disambiguation with context triggers, and varna-backed phoneme-inventory + phonotactics validation (`shabda_g2p_phoneme_inventory`, per-language IPA mapping) — always compiled, not feature-gated.
- **Changed**: dependencies are CYRIUS distlib bundles pulled via path (local dev) + git+tag (CI) — shabdakosh 3.0.2 (`shbdk_*`), svara 3.1.0 (`SVARA_PH_*`), varna 2.0.0. The transitive stack folds hisab/goonj/naad (shabdakosh) + hashmap/bayan (svara).
- **Removed**: the Rust `cli`/examples binaries, the criterion harness, and `no_std`/serde plumbing — replaced by `.tcyr` test suites and `tests/shabda.bcyr` benchmarks.

## [2.0.0] — 2026-04-01

Multi-language G2P, prosody control, SSML, accuracy, and varna integration.

- **Breaking**: `Language` enum expanded with `Spanish`, `German`, `Hindi`, `Arabic`, `Sanskrit` variants
- **Feature**: Spanish G2P rules — Castilian orthography (ch, ll, rr, qu, gu, c/g/z context)
- **Feature**: German G2P rules — sch, ch (ich/ach-Laut), ei/ie/eu digraphs, umlauts, final devoicing, ß
- **Feature**: Hindi G2P rules — Devanagari with inherent schwa deletion, virama/matra, romanized fallback
- **Feature**: Arabic G2P rules — 28 consonants, diacritics, shadda, tanween, hamza, romanized fallback
- **Feature**: Sanskrit G2P rules — perfectly regular Devanagari (no schwa deletion), 36C + 14V, romanized fallback
- **Feature**: `ConvertOptions` with builder pattern — `convert_with()` and `speak_with()` for emphasis, rate, timing
- **Feature**: Emphasis markers — ALL-CAPS and `*asterisk*` words receive emphatic stress
- **Feature**: Speaking rate control — `with_speaking_rate(wpm)`, clamped 50–300 WPM
- **Feature**: `TimingProfile` — independent vowel/consonant/pause duration scaling
- **Feature**: SSML subset parser — `<speak>`, `<break>`, `<emphasis>`, `<prosody>` tags
- **Feature**: `convert_ssml()` — applies SSML markup to G2P pipeline
- **Feature**: `convert_streaming()` — word-by-word callback API for real-time G2P
- **Feature**: Abbreviation expansion — 25-entry table (Dr.→doctor, Mr.→mister, etc.)
- **Feature**: Acronym handling — pronounceability heuristic (NASA→word, FBI→spelled out)
- **Feature**: Foreign word detection — diacritic detection with `strip_diacritics()` fallback
- **Feature**: Heteronym disambiguation — 20-entry context table (read, lead, live, wind, etc.)
- **Feature**: Optional `varna` feature — phoneme inventory validation, `detect_language()`, `phoneme_inventory()`
- **Feature**: Phonotactic constraint validation via varna (debug assertions)
- **Feature**: Language-aware IPA mapping — `validate_phonemes_for()` and `phoneme_to_ipa_for()`
- **Performance**: Rule compilation — static pattern slices eliminate per-match Vec allocations

## [1.0.0] — 2026-03-28

Full English G2P pipeline with dictionary, rules, syllabification, and prosody.

- **Feature**: Dictionary extracted to standalone `shabdakosh` crate (10,000+ entries)
- **Feature**: Context-sensitive vowel rules (magic-e, r-colored vowels)
- **Feature**: Silent letter handling (kn, gn, wr, ps, pn, mn, mb, bt, igh, eigh, ough)
- **Feature**: Morphological decomposition (-tion/-sion/-cian, -ed, un-/re-/dis-/pre-/mis-)
- **Feature**: Syllabification (Maximal Onset Principle with sonority constraints)
- **Feature**: Syllable-weight stress (heavy penult rule, antepenult fallback)
- **Feature**: Number-to-words expansion (0–999,999,999, decimals, negatives)
- **Feature**: Phrase-level prosody (commas 150ms, periods 300ms)
- **Changed**: Engine pipeline uses syllabification for stress instead of simple first-vowel rule
- **Changed**: `normalize()` expands numbers before text normalization

## [0.1.0] — 2026-03-27

- **Feature**: Initial scaffold — G2PEngine, dictionary lookup, rule-based fallback
- **Feature**: Built-in English dictionary (~30 entries), extensible
- **Feature**: Text normalization, sentence type detection, stress assignment
- **Feature**: `speak()` one-call text-to-audio via svara
- **Feature**: Integration tests, benchmarks, no_std, serde
